# WebServAnt roadmap

The repo's own work list (a HANDOFF is current state and is rewritten; work that must not be lost lives here). Build order set by CireSnave
2026-10-28 (verbatim): *"I'm thinking WebHostAnt's local serving should wait until WebServAnt is available. Similarly, WebServAnt should wait until
HTTPAnt is available."* HTTPAnt is published (`httpant` 0.5.22), so this is the next tool. Design: [docs/DESIGN.md](docs/DESIGN.md) (draft).
Status: `next` (do now, in this order), `later`, `blocked` (needs a decision), `done`.

**What WebServAnt is:** a web server program, and the library under it, that works with any Rust web library because it is built on HTTPAnt: static
files, a reverse proxy, and in-process ACME certificates. Release gate (EXPECTATIONS 6.24, easy is the product): a ten-line hello world, compiled and
run in CI, and a fresh-eyes trial of the published package that reaches a working `https://` site from the README alone.

| # | Row | Status | Notes |
|---|---|---|---|
| 0 | Design note, this roadmap, open decisions | **next** (draft written 2026-11-08) | `docs/DESIGN.md`. Decisions D1, D2, D5, D8 block row 1; each has a recommendation there. |
| 1 | Slice 1: the crate, hello world, static files over HTTP, and over TLS from a certificate file pair | **done** (0.1.0, #2) | `Site`, `webservant serve`, traversal/symlink/range/304 tests, mutants. |
| 2 | Slice 2: in-process ACME | **HTTP-01 done (0.2.0)**; TLS-ALPN-01 code removed in 0.3.0 (PM 2026-11-08: keep it out of the first version) | `instant-acme` 0.8.5 behind our `AcmeClient` trait; store, renewal that keeps the last good certificate, events, status, `renew_now`. Tested against a fake CA (7 tests) and against Pebble in Docker (the real protocol, with controls; required in Linux CI). **Not done: a run against Let's Encrypt staging** (needs a public name and port 80; PM and CireSnave's call) and the Windows ACL on the store (the default location is under the user's profile). **TLS-ALPN-01:** HTTPAnt's `ServerCert::from_pem`/`from_der` reject a certificate whose `acmeIdentifier` extension is critical (RFC 8737 requires it critical), because `keys_match` fails; measured, finding sent to the PM. Until HTTPAnt accepts it the option is not offered. |
| 3 | Slice 3: reverse proxy | **done in this PR (0.2.1)**; WebSocket upgrade refused with `501` (D3: no in v1) | `Site::proxy`, `--proxy`; prefix stripped; forwarded headers written by us, client's dropped; hop-by-hop and `Connection`-named headers stripped; streaming both ways; `502`/`504` without the upstream's address; dot segments refused (HTTPAnt's router already answers `400` for them, ours is a second layer covered by unit tests only). Not done: per-route body limit (HTTPAnt's 2 MiB request limit applies), HTTP/2 upstream, load balancing, the less common methods (`PROPFIND`...), a per-route timeout setting (connect 5 s, head 60 s, idle 60 s are fixed). |
| 4 | Consumer contract for WebHostAnt local serving | **written, unreviewed** (`docs/CONSUMER-CONTRACT.md`; the WebHostAnt lane is suspended) | Twelve needs from WebHostAnt's own docs, each marked built / gap. Largest gaps: C6 live read-back not exposed, C10 no settings struct, C11 no SIGTERM, C5 audit events partial, C2 no `check_reachable()`. |
| 5 | Patent sweep for the shipped techniques | **done for 0.2.x, awaiting PM acceptance** (`docs/PATENT-SWEEP.md`) | 14 hits read (claims from Google Patents), 8 queries, no unresolved overlap; thin by its own Limitations (one domain-limited web index, no boolean/CPC search, ACME IPR disclosures not searched). Closest live patents: Team Internet US9762571B2 (avoided because we never issue on demand from a ClientHello) and Red Hat US9215232B2/US8738901B2 (we are not CA-side and never reuse keys). Re-sweep before TLS-ALPN-01, DNS-01, on-demand issuance or WebSocket ships. |
| 6 | Fresh-eyes trial, hub card, first-release gate table | later (needs 1 to 4) | Same method as HTTPAnt's `docs/FRESH-EYES-TRIAL.md`. |
| 8 | Settings as data for ConfigurAnt (contract C10) | **done in this PR (0.3.1)**; not yet tried with ConfigurAnt | `Settings` + `Site::from_settings`, CLI goes through it. Next: run it with ConfigurAnt; `--config FILE` needs a TOML dependency. |
| 7 | SecretAnt backend for the account and certificate keys | blocked on SecretAnt's interface | Until then, files with restricted permissions. |

## Not in v1

DNS-01 and wildcard certificates; load balancing, caching, compression (compression is a later row), HTTP/3; any authentication of visitors; an admin
API or web UI; config reload on signal; installing itself as a service (WebHostAnt writes the service definition); IP-address certificates; OCSP
stapling; rendering Caddy or nginx configuration (WebHostAnt's edge adapters). Reasons are in `docs/DESIGN.md` section 6.
