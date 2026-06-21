use crate::tree::Node;
use colored::Colorize;

const BRANCH: &str = "├── ";
const LAST: &str = "└── ";
const PIPE: &str = "│   ";
const EMPTY: &str = "    ";

fn format_name(node: &Node) -> String {
    let version = match &node.version {
        Some(v) => format!(" {}", v.dimmed()),
        None => String::new(),
    };
    if node.cycle {
        format!(
            "{}{} {}",
            node.name.dimmed(),
            version,
            "[*]".dimmed().italic()
        )
    } else {
        format!("{}{}", node.name.bold(), version)
    }
}

pub fn print_tree(node: &Node, show_cycles: bool) {
    println!(
        "{} {}",
        node.name.bold().cyan(),
        node.version.as_deref().unwrap_or("").dimmed()
    );
    print_children(&node.children, "", show_cycles);

    let total = crate::tree::count_unique(node);
    println!("\n{}", format!("{} unique packages", total).dimmed());
}

fn print_children(children: &[Node], prefix: &str, show_cycles: bool) {
    let visible: Vec<&Node> = children
        .iter()
        .filter(|n| show_cycles || !n.cycle)
        .collect();

    for (i, child) in visible.iter().enumerate() {
        let is_last = i == visible.len() - 1;
        let connector = if is_last { LAST } else { BRANCH };
        println!("{}{}{}", prefix, connector, format_name(child));
        let new_prefix = format!("{}{}", prefix, if is_last { EMPTY } else { PIPE });
        print_children(&child.children, &new_prefix, show_cycles);
    }
}
