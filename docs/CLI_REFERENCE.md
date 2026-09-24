# contextctl: Command-Line Reference

`contextctl` is the companion CLI client for interacting with the `contextd` daemon over Varlink.

## Global Flags

- `-s, --socket <PATH>`: Override Varlink socket path (default: `/run/syntrop/io.syntrop.Context1`).
- `--json`: Output raw JSON replies instead of formatted text tables.
- `-h, --help`: Display help information.
- `-V, --version`: Display version.

## Subcommands

### 1. `unit`
Inspect causality timeline and recent configuration drift for a target systemd unit.

```bash
contextctl unit <UNIT_NAME> [--since <SECONDS>] [--json]
```

Example:
```bash
contextctl unit nginx.service --since 7200
```

### 2. `diffs`
List all configuration changes recorded across tracked system directories.

```bash
contextctl diffs [--since <SECONDS>] [--json]
```

Example:
```bash
contextctl diffs --since 3600
```

### 3. `events`
Query the chronological system causality event log.

```bash
contextctl events [--unit <UNIT_NAME>] [--since <SECONDS>] [--limit <COUNT>] [--json]
```

Example:
```bash
contextctl events --unit sshd.service --limit 20
```

### 4. `record`
Insert an arbitrary system event into the contextd event store.

```bash
contextctl record --source <SOURCE> [--unit <UNIT>] --summary <SUMMARY> [--details <DETAILS>]
```

Example:
```bash
contextctl record --source sentry --unit postgresql.service \
  --summary "OOM killer triggered on worker" \
  --details "cgroups v2 memory.events oom=1"
```

### 5. `info`
Introspect `contextd` daemon version, vendor details, and supported Varlink interfaces.

```bash
contextctl info [--json]
```

### 6. `completions`
Generate shell tab completion script for bash, zsh, fish, or powershell.

```bash
contextctl completions <SHELL>
```

Example:
```bash
source <(contextctl completions bash)
```
