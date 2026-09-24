# contextd: Architecture and System Design

`contextd` is the system chronology, configuration drift, and causality graph daemon in the Syntropd OS architecture. It provides historical context to supervisor daemons (`sentry`) and inference brokers (`inferenced`).

## 1. Problem Statement

When a Linux daemon or service fails, the immediate error is often a consequence of changes made earlier:
1. Configuration edits in `/etc` (e.g., malformed syntax, wrong ports, altered certificates).
2. Package upgrades (e.g., dynamic library changes, ABI breaks, updated configurations).
3. Cascade events from other daemons (e.g., GPU memory exhaustion, network state changes).

Standard system logs (`journald`) record runtime output, but do not record filesystem drift or package history alongside unit telemetry. `contextd` fills this void with minimal resource utilization (<15 MiB RSS).

## 2. Core Architecture

```
                  +-----------------------------------+
                  |             contextd              |
                  |                                   |
  /etc Files  --> | [FileTracker] -> [DiffStore]      |
  Package Logs--> | [PackageParser]                   |
                  | [EventStore] (events.jsonl)       |
                  +-----------------+-----------------+
                                    | Varlink IPC
                                    v
                     /run/syntrop/io.syntrop.Context1
                                    |
          +-------------------------+-------------------------+
          |                         |                         |
          v                         v                         v
       sentry                  inferenced                 contextctl
 (Crash Root Cause)       (Device Allocation)         (Admin CLI Tool)
```

## 3. Subsystem Overview

### 3.1 Configuration Drift Engine (`contextd-core::watcher`)
- **Baseline Tracking**: Captures initial file content snapshots on service startup.
- **Unified Diff Generation**: Computes standard unified diffs (`--- a/path` / `+++ b/path`) upon content changes.
- **Diff Storage**: Persists incremental unified diff records in `/var/lib/contextd/diffs/` indexed by microsecond timestamp.

### 3.2 Package Manager Transaction Correlator (`contextd-core::correlator`)
- Parses standard distribution transaction logs without C library dependencies:
  - DNF / RPM: `/var/log/dnf.log`
  - DPKG / APT: `/var/log/dpkg.log`
  - Pacman / Arch: `/var/log/pacman.log`
- Normalizes records into chronological `PackageTransactionRecord` structures.

### 3.3 Causality Event Index (`contextd-core::index`)
- Append-only JSON Lines log in `/var/lib/contextd/events.jsonl`.
- Microsecond timestamps (`timestamp_us`) prevent clock jitter ambiguities.
- Jaccard token similarity (`compute_jaccard_similarity`) allows fast root-cause cluster matching.

### 3.4 Varlink IPC Protocol (`contextd-daemon::varlink`)
- Conforms to standard Varlink framing (NUL-terminated JSON over Unix domain socket).
- Implements `org.varlink.service` introspection (`GetInfo`, `GetInterfaceDescription`).
- Implements `io.syntrop.Context1` (`GetUnitContext`, `ListRecentDiffs`, `ListEvents`, `RecordEvent`).

## 4. Resource Budgets and Constraints

- **Dynamic Dependencies**: Zero (`libc` and `rustix` syscall bindings only; no `libsystemd.so` or `libdbus-1.so`).
- **Memory Footprint**: Target RSS < 15 MiB.
- **I/O Overhead**: Streaming reads using fixed-size buffers (`[u8; 32768]`); no unbounded allocations.
- **Sandboxing**: `ProtectSystem=strict`, `MemoryDenyWriteExecute=yes`, `SystemCallFilter=@system-service`.
