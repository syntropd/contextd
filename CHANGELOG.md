# Changelog

All notable changes to contextd are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-09-24

### Changed

- **Breaking:** `contextctl`'s socket flag moved from `-s`/`--socket`
  to `-S`/`--socket` to disambiguate from `record --source`/`-s`.
  Scripts that used `-s` must switch to `-S`.

### Fixed

- `sd_notify` no longer silently fails on the abstract namespace path
  that stock systemd uses. Implementation now builds the `sockaddr_un`
  via `rustix::net::SocketAddrUnix::new_abstract_name` so the kernel
  receives the correct address for both abstract and filesystem
  targets. Embedded newlines in `STATUS` are stripped to prevent
  early termination of the variable and injection of fake keys.
- `systemd socket activation` now logs `warn!` when an adopted
  FD cannot be converted into a tokio `UnixListener`, instead of
  silently dropping the listener and leaving clients to see
  `connection refused`.
- `FileTracker::set_baseline` now logs at `error` level when the
  snapshot mutex is poisoned, instead of silently failing.
- `varlink GetInfo` reports `env!("CARGO_PKG_VERSION")` instead
  of the hardcoded `0.1.0`.
- `contextctl` CLI no longer collides between the global `-s`
  socket flag and `record -s` source flag (socket moved to `-S`).
- `contextctl` Varlink error replies now include the daemon's
  parameters payload (e.g. `InvalidParameter` reason, `NotFound`
  id) so operators can see why a call was rejected without
  enabling daemon-side debug logging.

### Added

- Unit tests for `notify_address` validation (filesystem path,
  abstract namespace, empty, interior-NUL rejection) and
  `STATUS` sanitization.
- `release-plz.toml`, CI workflow, and PR title lint for future
  automated releases.

## [0.1.0] - 2026-09-23

### Added

- Initial release: configuration drift watcher, JSONL event store,
  pure-Rust sd_notify, Varlink IPC (`io.syntrop.Context1`), and
  `contextctl` operator CLI.
