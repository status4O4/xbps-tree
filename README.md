# xbps-tree

A fast CLI utility for visualizing dependency trees of [xbps](https://github.com/void-linux/xbps) packages, written in Rust.

## Features

- Show full dependency tree for any installed package
- Show reverse dependencies (who depends on a package)
- Display package versions
- Detect and mark already-seen packages with `[*]`
- Limit tree depth
- Parallel dependency resolution via rayon
- Colored terminal output

## Installation

```bash
git clone https://github.com/yourname/xbps-tree
cd xbps-tree
cargo install --path .
```

## Usage

```bash
# Show dependency tree
xbps-tree curl

# Limit depth
xbps-tree curl --depth 2

# Show reverse dependencies
xbps-tree curl --reverse

# Hide already-seen packages
xbps-tree curl --no-cycles
```

## Output

```
curl 8.20.0_1
├── ca-certificates
│   ├── openssl
│   │   ├── glibc 2.41_1
│   │   ├── libcrypto3 3.6.2_1
│   │   │   └── glibc 2.41_1 [*]
│   │   └── libssl3 3.6.2_1
│   └── run-parts
├── glibc 2.41_1 [*]
├── libcurl 8.20.0_1
│   └── ...
└── zlib 1.2.3_1 [*]

24 unique packages
```

| Symbol | Meaning |
|--------|---------|
| `[*]` | Package already shown above, dependencies not expanded |

## Options

| Flag | Description |
|------|-------------|
| `--depth <N>` / `-d <N>` | Limit tree depth (default: 99) |
| `--reverse` / `-r` | Show reverse dependencies |
| `--no-cycles` | Hide already-seen packages |

## Project structure

```
src/
├── main.rs       # CLI interface, argument parsing
├── dep.rs        # Dep struct (name + version)
├── provider.rs   # PackageProvider trait
├── xbps.rs       # XbpsProvider — runs xbps-query, parses output
├── tree.rs       # Tree building, parallel dependency collection
├── output.rs     # Terminal output formatting
└── error.rs      # Error types
```

## Architecture

The project is built around a `PackageProvider` trait:

```rust
pub trait PackageProvider: Sync {
    fn deps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError>;
    fn rdeps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError>;
    fn version(&self, pkg: &str) -> Result<Option<String>, XbpsError>;
}
```

Tree building is split into two phases:

1. **Collect** — parallel traversal via `rayon`, builds a `HashMap<String, Vec<Dep>>`
2. **Build** — constructs the tree from the collected map

This separation makes the code testable (via `FakeProvider`) and enables parallelism in phase 1.

## Performance

```
xbps-tree firefox  9.73s user 0.92s system 644% cpu 1.652s total
```

`644% CPU` — rayon distributes work across all available cores. Without parallelism this would take ~10 seconds.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `anyhow` | Error handling in application layer |
| `thiserror` | Error types |
| `colored` | Terminal colors |
| `rayon` | Parallel dependency resolution |

## Requirements

- Void Linux with xbps installed
- `xbps-query` available in PATH

## License

MIT
