# WebServAnt roadmap

The repo's own work list (a HANDOFF is current state and is rewritten; work that must not be lost lives here). Build order set by CireSnave
2026-10-28 (verbatim): *"I'm thinking WebHostAnt's local serving should wait until WebServAnt is available. Similarly, WebServAnt should wait until
HTTPAnt is available."* HTTPAnt is published (`httpant` 0.7.1 from 0.4.6; 0.5.22 before), so this is the next tool. Design: [docs/DESIGN.md](docs/DESIGN.md) (draft).
Status: `next` (do now, in this order), `later`, `blocked` (needs a decision), `done`.

**What WebServAnt is:** a web server program, and the library under it, that works with any Rust web library because it is built on HTTPAnt: static
files, a reverse proxy, and in-process ACME certificates. Release gate (EXPECTATIONS 6.24, easy is the product): a ten-line hello world, compiled and
run in CI, and a fresh-eyes trial of the published package that reaches a working `https://` site from the README alone.

| # | Row | Status | Notes |
|---|---|---|---|
| 0 | Design note, this roadmap, open decisions | **done** (written 2026-11-08; D1, D2, D5, D8 settled by the PM and built in rows 1 to 3) | `docs/DESIGN.md`. Current state is the other rows: everything here is built through 0.4.2 and unpublished; what remains needs a person (staging run on CireSnave's VPS, httpant on crates.io). |
| 1 | Slice 1: the crate, hello world, static files over HTTP, and over TLS from a certificate file pair | **done** (0.1.0, #2) | `Site`, `webservant serve`, traversal/symlink/range/304 tests, mutants. |
| 2 | Slice 2: in-process ACME | **HTTP-01 done (0.2.0)**; TLS-ALPN-01 code removed in 0.3.0 (PM 2026-11-08: keep it out of the first version) | `instant-acme` 0.8.5 behind our `AcmeClient` trait; store, renewal that keeps the last good certificate, events, status, `renew_now`. Tested against a fake CA (7 tests) and against Pebble in Docker (the real protocol, with controls; required in Linux CI). **Let's Encrypt staging: `webservant check` passed on a real Debian 13 VPS 2026-11-09 (PM's run, `docs/FRESH-EYES-TRIAL.md` run 4); not done: serving and renewing a staging certificate there, production** (the Windows ACL on the store was done in 0.3.4). **TLS-ALPN-01:** HTTPAnt's `ServerCert::from_pem`/`from_der` reject a certificate whose `acmeIdentifier` extension is critical (RFC 8737 requires it critical), because `keys_match` fails; measured, finding sent to the PM. Until HTTPAnt accepts it the option is not offered. |
| 3 | Slice 3: reverse proxy | **done (0.2.1); proxy at `/` with files first in 0.4.1**; WebSocket upgrade refused with `501` (D3: no in v1) | `Site::proxy`, `--proxy`; prefix stripped; forwarded headers written by us, client's dropped; hop-by-hop and `Connection`-named headers stripped; streaming both ways; `502`/`504` without the upstream's address; dot segments refused (HTTPAnt's router already answers `400` for them, ours is a second layer covered by unit tests only). Not done: a per-route body limit (the limit is site-wide: `max_body_bytes`, 0.4.0), HTTP/2 upstream, load balancing, the less common methods (`PROPFIND`...). Timeouts per route are settings since 0.4.0. |
| 4 | Consumer contract for WebHostAnt local serving | **written, unreviewed** (`docs/CONSUMER-CONTRACT.md`; the WebHostAnt lane is suspended) | Twelve needs from WebHostAnt's own docs, each marked built / gap. Largest gaps: C6 live read-back not exposed, C10 no settings struct, C11 no SIGTERM, C5 audit events partial, C2 no `check_reachable()`. |
| 5 | Patent sweep for the shipped techniques | **done for 0.2.x; CireSnave ruled 2026-11-08 ("402: A"): publish the first version on this sweep** (`docs/PATENT-SWEEP.md`) | 14 hits read (claims from Google Patents), 8 queries, no unresolved overlap; thin by its own Limitations (one domain-limited web index, no boolean/CPC search, ACME IPR disclosures not searched). Closest live patents: Team Internet US9762571B2 (avoided because we never issue on demand from a ClientHello) and Red Hat US9215232B2/US8738901B2 (we are not CA-side and never reuse keys). Re-sweep before TLS-ALPN-01, DNS-01, on-demand issuance or WebSocket ships. |
| 6 | Fresh-eyes trial, hub card, first-release gate table | **QUICKSTART written (0.4.2); trial: see `docs/FRESH-EYES-TRIAL.md`**; hub card and gate table later | `docs/QUICKSTART.md`; the trial covers Part 1 and reading Part 2 (it cannot run the staging check without a domain). Same method as HTTPAnt's trial. |
| 8 | Settings as data for ConfigurAnt (contract C10) | **done (0.3.1, #7); tried with ConfigurAnt 0.6.3 in 0.3.3 (file layer)** | `Settings` + `Site::from_settings`, CLI goes through it; `configurant.toml` schema. Next: `--config FILE` (needs a TOML dependency), environment/flag layers through ConfigurAnt. |
| 9 | `check_reachable()` / `webservant check` (contract C2) | **done (0.3.2)** | One real issuance, throwaway store, staging forced over production; fake CA and Pebble (success and a real failed validation). Run against Let's Encrypt staging once, 2026-11-09 (passed). |
| 7 | SecretAnt backend for the account and certificate keys | blocked on SecretAnt's interface | Until then, files with restricted permissions. |

## Not in v1

DNS-01 and wildcard certificates; load balancing, caching, compression (compression is a later row), HTTP/3; any authentication of visitors; an admin
API or web UI; config reload on signal; installing itself as a service (WebHostAnt writes the service definition); IP-address certificates; OCSP
stapling; rendering Caddy or nginx configuration (WebHostAnt's edge adapters). Reasons are in `docs/DESIGN.md` section 6.
