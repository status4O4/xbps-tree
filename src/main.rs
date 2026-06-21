mod dep;
mod error;
mod output;
mod provider;
mod tree;
mod xbps;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::{provider::PackageProvider, xbps::XbpsProvider};

#[derive(Parser)]
#[command(
    name = "xbps-tree",
    about = "Show dependency tree for xbps packages",
    arg_required_else_help = true
)]
struct Cli {
    package: String,

    #[arg(short, long, default_value = "99")]
    depth: usize,

    #[arg(short, long)]
    reverse: bool,

    #[arg(long, help = "Hide already seen packages")]
    no_cycles: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format!("Error: {:#}", e).red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let provider = XbpsProvider;

    let root_version = provider.version(&cli.package)?;

    let visited = Arc::new(Mutex::new(HashSet::new()));
    let map = tree::collect_packages(&cli.package, 0, cli.depth, &visited, cli.reverse, &provider)?;

    let mut visited = HashSet::new();
    let root =
        tree::build_tree_from_map(&cli.package, root_version, 0, cli.depth, &mut visited, &map);

    output::print_tree(&root, !cli.no_cycles);
    Ok(())
}
