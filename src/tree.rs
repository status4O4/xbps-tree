use crate::dep::Dep;
use crate::error::XbpsError;
use crate::provider::PackageProvider;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct Node {
    pub name: String,
    pub version: Option<String>,
    pub children: Vec<Node>,
    pub cycle: bool,
}

impl Node {
    fn new(name: &str, version: Option<String>) -> Self {
        Node {
            name: name.to_string(),
            version,
            children: Vec::new(),
            cycle: false,
        }
    }
}

pub fn build_tree_from_map(
    pkg: &str,
    version: Option<String>,
    depth: usize,
    max_depth: usize,
    visited: &mut HashSet<String>,
    map: &HashMap<String, Vec<Dep>>,
) -> Node {
    let mut node = Node::new(pkg, version);

    if depth >= max_depth {
        return node;
    }

    if visited.contains(pkg) {
        node.cycle = true;
        return node;
    }

    visited.insert(pkg.to_string());

    if let Some(deps) = map.get(pkg) {
        for dep in deps {
            let child = build_tree_from_map(
                &dep.name,
                dep.version.clone(),
                depth + 1,
                max_depth,
                visited,
                map,
            );
            node.children.push(child);
        }
    }

    node
}

pub fn count_unique(node: &Node) -> usize {
    let mut seen = HashSet::new();
    count_recursive(node, &mut seen);
    seen.len()
}

fn count_recursive(node: &Node, seen: &mut HashSet<String>) {
    seen.insert(node.name.clone());
    for child in &node.children {
        count_recursive(child, seen);
    }
}

pub fn collect_packages(
    pkg: &str,
    depth: usize,
    max_depth: usize,
    visited: &Arc<Mutex<HashSet<String>>>,
    reverse: bool,
    provider: &dyn PackageProvider,
) -> Result<HashMap<String, Vec<Dep>>, XbpsError> {
    let result: Arc<Mutex<HashMap<String, Vec<Dep>>>> = Arc::new(Mutex::new(HashMap::new()));

    if depth >= max_depth {
        return Ok(Arc::try_unwrap(result).unwrap().into_inner().unwrap());
    }

    {
        let mut vis = visited.lock().unwrap();
        if vis.contains(pkg) {
            return Ok(Arc::try_unwrap(result).unwrap().into_inner().unwrap());
        }
        vis.insert(pkg.to_string());
    }

    let deps = if reverse {
        provider.rdeps(pkg)?
    } else {
        provider.deps(pkg)?
    };

    result.lock().unwrap().insert(pkg.to_string(), deps.clone());

    let errors: Mutex<Vec<XbpsError>> = Mutex::new(vec![]);

    deps.par_iter().for_each(|dep| {
        match collect_packages(&dep.name, depth + 1, max_depth, visited, reverse, provider) {
            Ok(child_map) => {
                result.lock().unwrap().extend(child_map);
            }
            Err(e) => {
                errors.lock().unwrap().push(e);
            }
        }
    });

    let errs = errors.into_inner().unwrap();
    if let Some(e) = errs.into_iter().next() {
        return Err(e);
    }

    Ok(Arc::try_unwrap(result).unwrap().into_inner().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::XbpsError;
    use crate::provider::PackageProvider;
    use std::sync::{Arc, Mutex};

    struct FakeProvider {
        deps: Vec<Dep>,
        rdeps: Vec<Dep>,
        version: Option<String>,
    }

    impl FakeProvider {
        fn new(
            deps: Vec<(&str, Option<&str>)>,
            rdeps: Vec<(&str, Option<&str>)>,
            version: Option<&str>,
        ) -> Self {
            FakeProvider {
                deps: deps
                    .into_iter()
                    .map(|(n, v)| Dep::new(n, v.map(|s| s.to_string())))
                    .collect(),
                rdeps: rdeps
                    .into_iter()
                    .map(|(n, v)| Dep::new(n, v.map(|s| s.to_string())))
                    .collect(),
                version: version.map(|s| s.to_string()),
            }
        }
    }

    impl PackageProvider for FakeProvider {
        fn deps(&self, _pkg: &str) -> Result<Vec<Dep>, XbpsError> {
            Ok(self.deps.clone())
        }
        fn rdeps(&self, _pkg: &str) -> Result<Vec<Dep>, XbpsError> {
            Ok(self.rdeps.clone())
        }
        fn version(&self, _pkg: &str) -> Result<Option<String>, XbpsError> {
            Ok(self.version.clone())
        }
    }

    fn build(pkg: &str, provider: &dyn PackageProvider) -> Node {
        let visited = Arc::new(Mutex::new(HashSet::new()));
        let map = collect_packages(pkg, 0, 99, &visited, false, provider).unwrap();
        let mut visited = HashSet::new();
        build_tree_from_map(pkg, None, 0, 99, &mut visited, &map)
    }

    #[test]
    fn test_leaf_node_has_no_children() {
        let provider = FakeProvider::new(vec![], vec![], None);
        let node = build("curl", &provider);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_tree_builds_children() {
        let provider = FakeProvider::new(
            vec![("glibc", Some("2.41")), ("zlib", Some("1.2.3"))],
            vec![],
            None,
        );
        let node = build("curl", &provider);
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].name, "glibc");
        assert_eq!(node.children[1].name, "zlib");
    }

    #[test]
    fn test_version_is_stored() {
        let provider = FakeProvider::new(vec![("glibc", Some("2.41"))], vec![], None);
        let node = build("curl", &provider);
        assert_eq!(node.children[0].version, Some("2.41".to_string()));
    }

    #[test]
    fn test_max_depth_limits_tree() {
        let provider = FakeProvider::new(vec![("glibc", None)], vec![], None);
        let visited = Arc::new(Mutex::new(HashSet::new()));
        let map = collect_packages("curl", 0, 1, &visited, false, &provider).unwrap();
        let mut visited = HashSet::new();
        let node = build_tree_from_map("curl", None, 0, 1, &mut visited, &map);
        assert_eq!(node.children.len(), 1);
        assert!(node.children[0].children.is_empty());
    }

    #[test]
    fn test_reverse_uses_rdeps() {
        let provider = FakeProvider::new(
            vec![("glibc", None)],
            vec![("firefox", None), ("chromium", None)],
            None,
        );
        let visited = Arc::new(Mutex::new(HashSet::new()));
        let map = collect_packages("curl", 0, 99, &visited, true, &provider).unwrap();
        let mut visited = HashSet::new();
        let node = build_tree_from_map("curl", None, 0, 99, &mut visited, &map);
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].name, "firefox");
    }

    #[test]
    fn test_cycle_protection() {
        let provider = FakeProvider::new(vec![("glibc", None)], vec![], None);
        let visited = Arc::new(Mutex::new(HashSet::new()));
        let map = collect_packages("curl", 0, 99, &visited, false, &provider).unwrap();
        let mut visited = HashSet::new();
        visited.insert("glibc".to_string());
        let node = build_tree_from_map("curl", None, 0, 99, &mut visited, &map);
        assert!(node.children[0].cycle);
        assert!(node.children[0].children.is_empty());
    }

    #[test]
    fn test_non_cycle_node_is_not_marked() {
        let provider = FakeProvider::new(vec![("glibc", None)], vec![], None);
        let node = build("curl", &provider);
        assert!(!node.children[0].cycle);
    }
}
