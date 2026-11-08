# What WebHostAnt's local serving gets from WebServAnt (consumer contract)

Written 2026-11-08 by the WebServAnt lane at `origin/main` of this repository after #3 and with #4 (reverse proxy, 0.2.1) in review. Derived from
WebHostAnt's own `docs/CONSUMER-CONTRACT.md`, `docs/TLS-NEED-LIST.md`, `docs/EDGE-AND-AGNOSTICISM.md` and `docs/DESIGN.md` (read at WebHostAnt
`origin/main`, 2026-11-08), through `docs/DESIGN.md` section 2 here. **The WebHostAnt lane has not reviewed this** (it is suspended); it is written so it
can be reviewed either way. "Built" means built and tested in this repository; the **Gap** column is what WebHostAnt would otherwise find out late.

## Hello world (what WebHostAnt's local adapter writes)

```rust
// Built (0.2.x). The name, contact and store come from the settings WebHostAnt was configured with.
let site = webservant::Site::new("school.example.org")
    .files("./public")                                   // static files at the root
    .proxy("/app", "http://127.0.0.1:3000")              // the consumer's own app (a prefix is required; "/" is not allowed yet)
    .acme(webservant::Acme::new("it@school.example.org").staging().store("/var/lib/webservant"))
    .start().await?;                                     // returns when a certificate is in service
let certs = site.certificates().unwrap();                // status(), renew_now()
```

## The twelve needs, what exists, what does not

| # | WebHostAnt need (source) | State | Gap, stated plainly |
|---|---|---|---|
| C1 | Certificate for a list of names; `ensure_certificate(names, challenge)`, `status()`, `renew_now()`, an event stream (DESIGN 5.3) | `status()` and `renew_now()` built; names fixed at start (`Site::new` + `Acme::also_name`) | No way to add a name to a running site (`ensure` for new names); events are a callback (`Acme::on_event`), not a stream |
| C2 | A staging switch and one test issuance that answers "is port 80/443 reachable" (DESIGN 5.4) | `.staging()` built | `check_reachable()` is **not built**; the only reachability test today is starting with `.staging()` and reading the error |
| C3 | HTTP-01 and TLS-ALPN-01 (DESIGN 5.2) | HTTP-01 built and tested against Pebble | TLS-ALPN-01 waits on HTTPAnt (its `ServerCert` rejects the RFC 8737 validation certificate; the fix is with the HTTPAnt lane). DNS-01 is not v1 |
| C4 | Renewal with no restart; a failed renewal keeps the current certificate (TLS-NEED-LIST 15, 16) | Built and tested (fake CA; failure keeps the old certificate being served) | The 12-hour check interval and the one-third-of-lifetime rule are not settings you can read back |
| C5 | Events for AuditAnt: issued, renewed, renewal failed, challenge served, certificate swapped, handshake refused; no key material (TLS-NEED-LIST 23) | `AcmeEvent::{Loaded, Issued, RenewalFailed}` built | Challenge served, certificate swapped and handshake refused are **not** surfaced; HTTPAnt's `TlsAudit` is not passed through |
| C6 | Read back what the live listener really serves, not what the store says (TLS-NEED-LIST 22; CONSUMER-CONTRACT 2) | **Not built.** `status()` reports the certificate we put in service | HTTPAnt has the read-back (0.5.4); WebServAnt does not expose it. Until it does, WebHostAnt must not treat `status()` as proof |
| C7 | No secret in any log, plan, event or command line; credential in memory for one run (CONSUMER-CONTRACT 3) | Private keys: redacted `Debug`, never in an error (tested for the certificate-file path); store files are mode 0600 on Unix | No canary test over the ACME paths yet; **the store holds private keys on disk by design** (the account key and the certificate key), and on Windows relies on the default location being under the user's profile (explicit ACL is not built); SecretAnt backend is row 7 |
| C8 | A server-side probe refuses private/loopback/link-local addresses unless opted in (CONSUMER-CONTRACT 3 rule 4) | The proxy upstream is a configured address, not one taken from a request, so there is nothing to refuse | Nothing in WebServAnt probes a URL a person typed; if WebHostAnt wants a probe it uses HTTPAnt's client `refuse_private_addresses(true)` |
| C9 | Port 80 redirects to 443 and leaves the challenge path alone (TLS-NEED-LIST 18) | Built (`308`, `421` for a host that is not ours) | Redirect target port follows the HTTPS listener's actual port; not configurable separately |
| C10 | Configured by settings (ConfigurAnt), not code (DESIGN 0, 6) | CLI flags and the builder only | **No serde settings struct yet**, so ConfigurAnt cannot fill it by dotted key; HTTPAnt did this in 0.5.34 (`ServerSettings`) and the same shape is the plan |
| C11 | Runs as a long-lived service: logs to stdout, non-zero exit with a sentence, graceful stop, no prompts (DESIGN 3.1 row 3) | Exit codes and sentences built; graceful stop on Ctrl-C | **SIGTERM is not handled** (`tokio::signal::ctrl_c` only), so a systemd stop is an abrupt kill; Windows service control is not handled either. No logging beyond the one start line |
| C12 | An app of the consumer's can sit behind it (CONSUMER-CONTRACT 1, 4) | Reverse proxy built (#4): prefix, streaming, forwarded headers written by us | The proxy needs a **prefix**: a consumer app at `/` is not possible next to the files yet. No WebSocket. Humboldt's authenticated configuration endpoint passes through unchanged because the proxy adds nothing to it |

## What WebHostAnt must not assume

- That a name is covered because `start()` returned: it returned because *a* certificate was put in service for the names listed, and C6 says that is not a read-back.
- That a renewal worked because no event arrived: `RenewalFailed` is delivered only if an `on_event` callback was registered, and `status().last_result` is the other source.
- That HTTPS works when only 443 is open: TLS-ALPN-01 is not offered yet.
- That a Let's Encrypt run has happened: **nothing has been run against Let's Encrypt, staging or production**, only against Pebble.

## Order I would do the gaps in (a proposal for the PM)

1. C6 (expose HTTPAnt's read-back) and C11 (SIGTERM): both small, both change what a consumer can trust.
2. C10 (settings struct) so ConfigurAnt can drive it.
3. C5 (pass `TlsAudit` through) and C2 (`check_reachable()`).
4. C1 (add a name to a running site) only if WebHostAnt shows it needs it.
