# contextd

System Chronology, Configuration Drift, and Causality Graph Daemon for the Syntropd OS Suite.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](Cargo.toml)

`contextd` monitors Linux system state changes, configuration drift across `/etc`, package transactions across package managers (`dnf`, `dpkg`, `pacman`), and correlates historical events to enable root cause analysis for supervisor daemons like `sentry`.

---

## Features

- **Configuration Drift Tracking**: Monitors tracked paths and records unified diff snapshots in `/var/lib/contextd/diffs/`.
- **Package Transaction Log Parsing**: Zero-dependency parsing of DNF, DPKG, and Pacman logs.
- **Append-Only Event Store**: Microsecond-indexed JSONL event persistence in `/var/lib/contextd/events.jsonl`.
- **Causality Timeline Correlation**: Correlates configuration changes and package upgrades with target unit incidents.
- **Pure Rust Varlink IPC**: Native implementation of `io.syntrop.Context1` over Unix domain sockets with socket activation.
- **Zero Dynamic C Dependencies**: Compiles directly against standard Linux syscalls without `libsystemd.so` or `libdbus-1.so`.
- **Minimal Resource Footprint**: Fixed-size streaming buffers and strict cgroups constraints (<15 MiB RSS).

---

## Directory Structure

```
contextd/
├── Cargo.toml
├── crates/
│   ├── contextd-core/       # Core engines: diffs, package parser, event store
│   ├── contextd-daemon/     # Daemon binary: socket activation, Varlink server
│   └── contextctl/          # Admin CLI tool for querying and controlling contextd
├── qa/
│   ├── unit/                # 1:1 unit tests for every core and daemon function
│   └── edge/                # Edge cases, corrupt logs, high-scale diffs
├── systemd/                 # contextd.service and contextd.socket units
├── sysusers.d/              # User and group definitions
├── tmpfiles.d/              # Directory lifecycle and permissions
├── install/                 # install.sh and uninstall.sh scripts
└── docs/                    # Architecture, Varlink spec, CLI reference
```

---

## Quickstart

### Build and Test

```bash
cargo build --release
cargo test --workspace
```

### Run Daemon Locally

```bash
cargo run --bin contextd
```

### Query via contextctl

```bash
# Query unit context
cargo run --bin contextctl -- unit nginx.service

# List recent configuration diffs
cargo run --bin contextctl -- diffs --since 3600

# Query events
cargo run --bin contextctl -- events --limit 20
```

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
