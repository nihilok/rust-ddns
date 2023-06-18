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
```

The above example config would make 6 calls to the same server, one for each method for each record type. You can provide between 1 and 3 methods, either PUT, POST, or DELETE.
