# FR0004 — Squarespace Migration (Google Domains Sunset)

## Status
Draft

## Summary
Google Domains was sold to Squarespace in 2023. Squarespace does not support the DynDNS `/nic/update` protocol that the `GoogleDomains` protocol variant currently targets, so that variant is broken for all migrated users. This FR removes the dead protocol and adds Cloudflare as a replacement, as it is the most common migration target.

## Background
The `GoogleDomains` protocol variant in `api_client.rs` issues:

```
GET https://domains.google.com/nic/update?hostname=<domain>&myip=<ip>
```

with HTTP Basic Auth. Squarespace has explicitly dropped DDNS support — there is no equivalent API. Users who migrated from Google Domains to Squarespace Domains have no path forward with the current codebase.

Squarespace does not provide any public DNS management API. The practical migration target is **Cloudflare**, which has a well-documented REST API and is the most widely recommended alternative in the DDNS community.

## Proposed Changes

### 1. Deprecate and remove `Protocol::GoogleDomains`
- Remove the `GoogleDomains` variant from the `Protocol` enum.
- If `server: domains.google.com` is found in a config file, exit with a clear error message directing users to this change.

### 2. Add `Protocol::Cloudflare`
Cloudflare DNS updates use a REST API, which is meaningfully different from the existing DynDNS and Mail-in-a-Box patterns:

- **Endpoint:** `PUT https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}`
- **Auth:** Bearer token (`Authorization: Bearer <token>`) — no username/password pair
- **Body:** JSON `{ "type": "A", "name": "<domain>", "content": "<ip>", "ttl": 1 }`

This requires two new config fields: `zone_id` and `record_id`. The `username`/`password` fields are replaced by a single `api_token` field (compatible with FR0001's `env:` prefix).

#### New config shape for Cloudflare
```yaml
server: cloudflare
domain: ddns.example.com
zone_id: abc123
record_id: def456
api_token: env:CF_API_TOKEN
records:
  - A
```

### 3. Protocol detection
`Protocol::from_server` maps `"cloudflare"` → `Protocol::Cloudflare`, and emits a deprecation error for `"domains.google.com"`.

## Acceptance Criteria
- Configs using `server: domains.google.com` produce a clear error explaining the sunset and pointing to docs.
- `server: cloudflare` successfully updates A and AAAA records via the Cloudflare API.
- `api_token` supports the `env:` prefix from FR0001.
- README documents the new Cloudflare config format and migration steps from the old Google Domains config.

## Migration Guide (to be added to README)
1. Transfer domain DNS to Cloudflare (or any provider supported by existing `MailInABox` protocol).
2. Create a scoped Cloudflare API token with `Zone / DNS / Edit` permission.
3. Obtain `zone_id` (Cloudflare dashboard → domain → Overview) and `record_id` (`GET /zones/{zone_id}/dns_records`).
4. Update config to use `server: cloudflare` with the new fields.

## Out of Scope
- Squarespace DNS API support (no public API exists).
- Automatic lookup of `record_id` from the domain name (could be a future FR).
- Other Cloudflare API operations (proxying, page rules, etc.).
