use crate::dep::Dep;
use crate::error::XbpsError;
use crate::provider::PackageProvider;
use std::process::Command;

pub struct XbpsProvider;

fn parse_dep(line: &str) -> Dep {
    if let Some(idx) = line.find(">=") {
        let name = line[..idx].trim().to_string();
        let version = line[idx + 2..].trim().to_string();
        let version = if version == "0" || version.is_empty() {
            None
        } else {
            Some(version)
        };
        return Dep::new(name, version);
    }
    Dep::new(line.trim(), None)
}

fn parse_rdep(line: &str) -> Dep {
    let parts: Vec<&str> = line.rsplitn(2, '-').collect();
    if parts.len() == 2
        && parts[0]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        return Dep::new(parts[1], Some(parts[0].to_string()));
    }
    Dep::new(line, None)
}

impl PackageProvider for XbpsProvider {
    fn deps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError> {
        get_deps(pkg)
    }
    fn rdeps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError> {
        get_rdeps(pkg)
    }
    fn version(&self, pkg: &str) -> Result<Option<String>, XbpsError> {
        get_version(pkg)
    }
}

fn run_xbps_query(args: &[&str]) -> Result<Vec<String>, XbpsError> {
    let output = Command::new("xbps-query")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                XbpsError::NotFound
            } else {
                XbpsError::Io(e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

pub fn get_version(pkg: &str) -> Result<Option<String>, XbpsError> {
    let lines = run_xbps_query(&["-S", pkg])?;
    for line in &lines {
        if let Some(rest) = line.strip_prefix("pkgver: ") {
            let parts: Vec<&str> = rest.rsplitn(2, '-').collect();
            if parts.len() == 2
                && parts[0]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                return Ok(Some(parts[0].to_string()));
            }
        }
    }
    Ok(None)
}

pub fn get_deps(pkg: &str) -> Result<Vec<Dep>, XbpsError> {
    let lines = run_xbps_query(&["-x", pkg])?;
    if lines.is_empty() {
        let check = run_xbps_query(&["-S", pkg])?;
        if check.is_empty() {
            return Err(XbpsError::PackageNotFound(pkg.to_string()));
        }
    }
    Ok(lines.into_iter().map(|l| parse_dep(&l)).collect())
}

pub fn get_rdeps(pkg: &str) -> Result<Vec<Dep>, XbpsError> {
    let lines = run_xbps_query(&["-X", pkg])?;
    Ok(lines.into_iter().map(|l| parse_rdep(&l)).collect())
}
