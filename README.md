# rust-ddns (dynamic DNS client)

### Requirements:

- cargo
- dnsutils

### Installation

`./install.sh` or `RUST_DDNS_INTERVAL=30min ./install.sh`

(default interval is `5min`)

```sh
sudo systemctl enable rust-ddns.timer
sudo systemctl start rust-ddns.timer
```

## Debugging

Set log level to debug:

`export DDNS_LOG_LEVEL=debug`

## Configuration

Default config file (not created automatically): `$HOME/.ddns.conf`

Custom config path with option `-c`/`--config-file` e.g.

```sh
DDNS_LOG_LEVEL=debug rust-ddns --config-file ./my.conf
```

Config file is in `yaml` format, and must include the following properties:

```yaml
server: my.dns.provider.com
domain: ddns.domain.com
methods:
    - DELETE
    - POST
    - PUT
records:                    # optional, defaults to A
    - A
    - AAAA
username: api-key/username
password: api-secret/password
---                         # multiple configs split by line break
server: my.other.dns.provider
domain: my.other.domain.com
...  
```

The above example config would make 6 calls to the same server, one for each method for each record type. You can provide between 1 and 3 methods, either PUT, POST, or DELETE.

### Secure Credential Storage

Passwords (and usernames) can be read from environment variables at runtime using the `env:` prefix:

```yaml
password: env:MY_SECRET_VAR
```

- `password: env:MY_VAR` resolves the value from environment variable `MY_VAR` at runtime
- If the variable is unset or empty, the app exits with an error
- Plain text passwords still work unchanged
- Recommended: use a systemd `EnvironmentFile` to supply secrets without writing them to disk

Example systemd service drop-in (`/etc/systemd/system/rust-ddns.service.d/secrets.conf`):

```ini
[Service]
EnvironmentFile=/etc/rust-ddns/secrets
```

Where `/etc/rust-ddns/secrets` contains:

```
MY_SECRET_VAR=your-api-key-here
```
