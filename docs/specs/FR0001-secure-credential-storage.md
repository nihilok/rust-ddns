# FR0001 — Secure Credential Storage

## Status
Accepted

## Summary
Passwords (API secrets) must not be stored in plain text in the config file (`~/.ddns.conf`).

## Background
The current config format requires a `password` field as a plain text string:

```yaml
username: api-key
password: my-secret
```

This means anyone with read access to the file can immediately obtain the credential.

## Requirement
The application must support supplying the password via an environment variable reference in the config file, so that the secret never needs to be written to disk.

## Chosen Approach — Environment Variable Reference

A `password` value prefixed with `env:` is treated as an environment variable name. The application resolves it at runtime:

```yaml
username: api-key
password: env:MY_DNS_API_SECRET
```

At startup, if the resolved env var is unset or empty, the application exits with a clear error message indicating which variable is missing.

Plain text values (no `env:` prefix) continue to work unchanged for backwards compatibility.

### Systemd integration

The recommended way to supply the secret without it appearing in shell history or the config file is via a systemd `EnvironmentFile`:

```ini
# /etc/systemd/system/rust-ddns.service  (or a drop-in override)
[Service]
EnvironmentFile=/etc/rust-ddns/secrets.env
```

```sh
# /etc/rust-ddns/secrets.env  (mode 0600, owned by root or the service user)
MY_DNS_API_SECRET=my-secret
```

## Acceptance Criteria
- `password: env:VAR_NAME` resolves the value from the environment at runtime.
- If the referenced variable is unset or empty, the application exits with an informative error (e.g. `"password env var 'VAR_NAME' is not set"`).
- Plain text `password` values continue to work without any changes.
- README documents the `env:` syntax and the systemd `EnvironmentFile` pattern.

## Implementation Notes
- Resolution should happen in `parse_yaml` in `api_client.rs`, after reading the raw string from the YAML doc.
- A small helper function (e.g. `resolve_secret(value: &str) -> Result<String, ...>`) keeps the logic contained and testable.
- `username` may use the same `env:` prefix for consistency, but is not required by this FR.

## Out of Scope
- System keyring integration.
- Credential helper / external command execution.
- Encrypting the entire config file at rest.
- Multi-user or role-based credential access.
