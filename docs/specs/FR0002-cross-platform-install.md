# FR0002 — Cross-Platform Installation

## Status
Draft

## Summary
The install and uninstall scripts are Linux/systemd-only. The application should be installable and run as a scheduled task on macOS and Windows as well.

## Background
`install.sh` and `uninstall.sh` hardcode systemd units under `/etc/systemd/system/`, making them incompatible with macOS (which uses launchd) and Windows (which uses Task Scheduler or a service manager). The Rust binary itself already compiles cross-platform (`build_config_path` has a Windows variant), but there is no install tooling to match.

## Requirement
Provide installation and uninstall support for Linux (systemd), macOS (launchd), and Windows (Task Scheduler), with consistent behaviour across all three: periodic execution, logging, and clean removal.

## Per-Platform Design

### Linux — systemd (existing)
No changes to current behaviour. Timer/service units installed to `/etc/systemd/system/`.

### macOS — launchd
- Install a `LaunchAgent` plist to `~/Library/LaunchAgents/com.rust-ddns.plist`.
- Use `StartInterval` (seconds) for the run interval, defaulting to 300 (5 min).
- Load with `launchctl bootstrap gui/$UID`.
- Log stdout/stderr via `StandardOutPath` / `StandardErrorPath` to `~/.rust-ddns.log`.
- Uninstall: `launchctl bootout`, remove plist.

### Windows — Task Scheduler
- Register a scheduled task via `schtasks` or a PowerShell script.
- Trigger: repeat every 5 minutes indefinitely.
- Action: run the compiled binary with the user's config file.
- Log: redirect output to `%USERPROFILE%\.rust-ddns.log` via a wrapper `.cmd` script (analogous to `ddnsd`).
- Uninstall: `schtasks /Delete`.

## Acceptance Criteria
- Running the appropriate install script on each platform results in the binary executing on a configurable schedule without manual intervention after reboot.
- The install interval is configurable via `RUST_DDNS_INTERVAL` on all platforms (seconds or a duration string as appropriate per platform).
- Logs are written to a consistent default location (`~/.rust-ddns.log` / `%USERPROFILE%\.rust-ddns.log`) and rotated to the last 200 lines, matching current Linux behaviour.
- A corresponding uninstall script/command exists for each platform.
- README documents platform-specific install steps.

## Implementation Notes
- Consider a single `install.sh` (bash, covers Linux + macOS via Homebrew bash) and a separate `install.ps1` for Windows.
- The `RUST_DDNS_INTERVAL` env var may need unit conversion per platform (systemd accepts `5min`, launchd requires integer seconds, schtasks uses `HH:MM:SS`).
- The `ddnsd` log-rotation wrapper should be replicated as a `.cmd` or PowerShell script on Windows.

## Out of Scope
- Package manager distribution (brew formula, winget manifest, apt package).
- Running as a system-level service (non-user session) on any platform.
