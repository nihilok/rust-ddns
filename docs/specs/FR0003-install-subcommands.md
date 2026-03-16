# FR0003 — Install/Uninstall Subcommands

## Status
Implemented

## Dependencies
- FR0002 — Cross-Platform Installation (defines the per-platform install behaviour this FR bakes in)

## Summary
Replace the external `install.sh` / `uninstall.sh` scripts with `rust-ddns install` and `rust-ddns uninstall` subcommands built into the binary, providing a single-artefact distribution story.

## Background
FR0002 requires maintaining three platform-specific install scripts. Baking this logic into the binary keeps install behaviour in sync with the binary version, eliminates the need to retain the source directory post-build, and gives a consistent CLI across platforms.

## CLI Design

```
rust-ddns install   [--interval <duration>] [--log-file <path>] [--config-file <path>]
rust-ddns uninstall [--purge]
```

- `--interval` — scheduler interval, default `5min`. Accepted formats: `5min`, `30min`, `1h`. The binary converts to the platform-native unit internally.
- `--log-file` — override the default log path (`~/.rust-ddns.log` / `%USERPROFILE%\.rust-ddns.log`).
- `--config-file` — path to the config file to bake into the scheduled task/service definition.
- `--purge` (uninstall only) — also remove the config file and log file.

## Behaviour

### Install
1. Detect the current executable path (`std::env::current_exe()`).
2. Copy the binary to the user-local bin directory (`~/.local/bin/` on Linux/macOS, `%LOCALAPPDATA%\rust-ddns\` on Windows) if not already there.
3. Register the scheduler entry for the current platform (see FR0002 for per-platform detail), pointing at the installed binary path.
4. On Linux/macOS: write a small log-rotation wrapper script equivalent to `ddnsd` (keeps last 200 lines).
5. Print next steps (e.g. `sudo systemctl enable --now rust-ddns.timer`).

### Uninstall
1. Stop and remove the scheduler entry/service for the current platform.
2. Remove the installed binary and wrapper script.
3. If `--purge`: remove config file and log file.

### Privilege escalation
- On Linux/macOS, steps that require `sudo` (writing to `/etc/systemd/system/`, `launchctl bootstrap`) should be clearly called out; the binary invokes them with `sudo` via `std::process::Command` and fails with a clear message if permission is denied.
- On Windows, the binary should request elevation via a manifest or detect if not running as Administrator and re-launch elevated.

## Acceptance Criteria
- `rust-ddns install` produces an equivalent result to the current `install.sh` on Linux.
- `rust-ddns install` works correctly on macOS (launchd) and Windows (Task Scheduler) per FR0002.
- `rust-ddns uninstall` cleanly reverses all install steps on all three platforms.
- `install.sh`, `uninstall.sh`, and `ddnsd` are removed from the repository.
- The `--interval` flag accepts the same duration strings as the current `RUST_DDNS_INTERVAL` env var; the env var remains supported as a fallback for scripted use.
- README is updated to document the new subcommands.

## Implementation Notes
- Extend `arg_parser.rs` with a `clap` subcommand enum (`Subcommand::Install`, `Subcommand::Uninstall`) alongside the existing flat `--ip` flag.
- Platform-specific logic lives in a new `installer` module, gated by `#[cfg(target_os = ...)]`.
- Duration parsing for `--interval` can be a small shared utility (feeds into FR0002's interval conversion requirement).
- The log-rotation wrapper on Linux/macOS can be written as a static string template — no need for a separate shell script file in the repo.

## Out of Scope
- Running as a system-level (non-user) service.
- Package manager integration (brew, winget, apt).
- Auto-update functionality.
