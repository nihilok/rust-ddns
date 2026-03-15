# FR0005 — Namecheap DDNS Support

## Status
Draft

## Summary
Add support for Namecheap's Dynamic DNS API as a new protocol variant.

## Background
Namecheap provides a dedicated DDNS endpoint separate from their full XML API. It uses a simple HTTP GET with query parameters and returns XML. The protocol is meaningfully different from both Mail-in-a-Box and Cloudflare (FR0004) in two ways: the domain is split into `host` (subdomain) and `domain` (apex), and the response must be parsed as XML to detect errors.

## API Details

- **Endpoint:** `GET https://dynamicdns.park-your-domain.com/update`
- **Auth:** Per-domain DDNS password passed as a query parameter (not the account password — generated under Domain List → Advanced DNS → Enable Dynamic DNS)
- **Query parameters:** `host`, `domain`, `password`, `ip`
- **Response:** XML; success is indicated by `<ErrCount>0</ErrCount>`

Example request:
```
GET https://dynamicdns.park-your-domain.com/update?host=ddns&domain=example.com&password=abc123&ip=1.2.3.4
```

## Config Shape

The `domain` field in the existing config format is the full hostname (e.g. `ddns.example.com`). For Namecheap, the application should split this automatically into the apex domain and host label, so the user does not need to supply them separately.

```yaml
server: namecheap
domain: ddns.example.com
password: env:NAMECHEAP_DDNS_PASSWORD
```

At runtime, `ddns.example.com` is split into `host=ddns` and `domain=example.com`. The apex domain (root record) can be expressed as `domain: example.com`, which maps to `host=@`.

The existing `username` field is not used — Namecheap DDNS has no username concept.

## Protocol Detection

`Protocol::from_server` maps `"namecheap"` → `Protocol::Namecheap`.

## Quirks and Limitations

- **A records only.** The Namecheap DDNS endpoint does not support AAAA records. If `records: [AAAA]` is specified, the application should exit with a clear error.
- **XML response parsing.** Unlike the existing protocols which log the raw response text, the Namecheap handler must parse the XML and surface any errors from `<errors>` when `<ErrCount>` is non-zero.
- **Password in query string.** The password is transmitted as a URL query parameter — HTTPS is mandatory (reqwest defaults to this).
- **IP parameter.** The application should always pass the `ip` parameter explicitly (using the resolved actual IP from `ip_checker.rs`) rather than relying on Namecheap's auto-detection, which is unreliable behind NAT.

## Acceptance Criteria

- `server: namecheap` successfully updates an A record via the Namecheap DDNS endpoint.
- The full domain is automatically split into host and apex — no extra config fields required.
- Apex domain (`domain: example.com` with no subdomain) correctly sends `host=@`.
- Specifying `records: [AAAA]` produces a clear unsupported error.
- XML error responses surface a human-readable message rather than silently succeeding.
- `password` supports the `env:` prefix from FR0001.
- README documents the config format and where to find the DDNS password in the Namecheap dashboard.

## Out of Scope
- AAAA record support (not available via this endpoint).
- Wildcard records (`host=*`).
- Namecheap's full XML API (`api.namecheap.com`).
