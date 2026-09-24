# io.syntrop.Context1: Varlink Interface Specification

The `io.syntrop.Context1` interface provides programmatic access to system chronology, configuration drift diffs, and causality timelines.

Socket Endpoint: `/run/syntrop/io.syntrop.Context1`

## 1. Interface Definition

```varlink
interface io.syntrop.Context1

type ConfigDiff (
  file_path: string,
  timestamp_us: int,
  diff_content: string
)

type PackageTransaction (
  timestamp_us: int,
  action: string,
  package_name: string,
  version: string
)

type UnitContext (
  unit_name: string,
  config_diffs: []ConfigDiff,
  package_upgrades: []PackageTransaction,
  summary: string
)

type Event (
  id: string,
  timestamp_us: int,
  source: string,
  unit: ?string,
  summary: string,
  details: ?string
)

method GetUnitContext(unit: string, since_seconds: int) -> (context: UnitContext)
method ListRecentDiffs(since_seconds: int) -> (diffs: []ConfigDiff)
method ListEvents(unit: ?string, since_seconds: int, limit: int) -> (events: []Event)
method RecordEvent(source: string, unit: ?string, summary: string, details: ?string) -> (event_id: string)

error InvalidParameter(parameter: string)
error OperationFailed(reason: string)
```

## 2. Methods

### 2.1 `GetUnitContext`
Aggregates recent configuration drift and package updates relevant to the target unit name.
- Parameters:
  - `unit` (string): Target systemd unit name (e.g., `nginx.service`).
  - `since_seconds` (int): Number of seconds into the past to correlate.
- Returns:
  - `context` (`UnitContext`): Aggregated incident timeline and summary.

### 2.2 `ListRecentDiffs`
Retrieves all recorded configuration diff snapshots within the specified time window.
- Parameters:
  - `since_seconds` (int): Time window in seconds.
- Returns:
  - `diffs` (`[]ConfigDiff`): List of recorded unified diffs.

### 2.3 `ListEvents`
Queries recorded causality events with optional unit filtering.
- Parameters:
  - `unit` (?string): Optional unit name filter.
  - `since_seconds` (int): Time window in seconds.
  - `limit` (int): Maximum records to return.
- Returns:
  - `events` (`[]Event`): List of matching event records.

### 2.4 `RecordEvent`
Appends a new event into the append-only causality event store.
- Parameters:
  - `source` (string): Subsystem recording the event (e.g., `sentry`, `inferenced`).
  - `unit` (?string): Associated systemd unit name.
  - `summary` (string): Short human-readable summary.
  - `details` (?string): Diagnostic snippet or error trace.
- Returns:
  - `event_id` (string): Generated event identifier.
