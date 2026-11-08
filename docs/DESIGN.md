# WebServAnt design (draft)

**Design only. Nothing here is built.** Written 2026-11-08 by the WebServAnt lane. "Read" means read this session: HTTPAnt at `origin/main`
(`README.md`, `ROADMAP.md`, `crates/httpant-server/src/tls.rs`, the `Server` surface in `lib.rs`; published as `httpant` 0.5.22), and WebHostAnt at
`origin/main` (`docs/CONSUMER-CONTRACT.md`, `TLS-NEED-LIST.md`, `EDGE-AND-AGNOSTICISM.md`, `DESIGN.md` sections 0 to 8.1 only). Crate facts (versions,
licences, dependencies, release dates) were read from the crates.io and GitHub APIs on 2026-11-08. **UNVERIFIED** marks anything recalled and not read,
and nothing was run: no ACME exchange has been attempted against any CA.

## 0. Why this exists

CireSnave, 2026-10-28, verbatim: *"I'm thinking WebHostAnt's local serving should wait until WebServAnt is available. Similarly, WebServAnt should wait
until HTTPAnt is available. If we work it that way then we build the tools before we build what sits on top of them."* HTTPAnt is published, so this is
the next tool. The first consumer is WebHostAnt's local serving, and through it the Humboldt installer.

## 1. What WebServAnt is

A web server **program** (and a library under it) that runs on whichever Rust web library the person chooses, because it is built on HTTPAnt's `Server`.
It does the three things a person who wants to put a site on the internet from their own machine needs, and nothing else:

1. **Serves static files** from a directory.
2. **Reverse-proxies** a path or host to an app that is already running.
3. **Gets and renews its own TLS certificates** (ACME, in process), so `https://` works with no second program.

### The hello world (a release gate: ten lines or fewer, compiled and run in CI)

```rust
// Intended, not built.
#[tokio::main]
async fn main() -> webservant::Result<()> {
    webservant::Site::new("example.org")
        .contact("me@example.org")      // the ACME account address
        .files("./public")               // static files
        .serve().await                   // certificate, port 80 and 443, renewal: all of it
}
```

and the same thing with no code, from a settings file or the command line (`webservant serve --name example.org --files ./public`). **Easy is the
product** (EXPECTATIONS 6.24): the gate is that a stranger who has only the README reaches a working `https://` site, shown by the fresh-eyes trial
HTTPAnt used (`docs/FRESH-EYES-TRIAL.md` there), not by us saying so. A name that cannot get a certificate (port 80 not reachable, no public DNS) fails
with a sentence that says which and what to do; it never serves plain HTTP in its place without the person asking (section 5).

### Where it sits

```
 person's settings (ConfigurAnt)
          |
   WebServAnt  -- static files, reverse proxy, ACME orchestration, certificate store, renewal
          |         (CertSource impl, ChallengeResponder impl)
      HTTPAnt   -- Server, TLS acceptor (rustls), CertSource trait, redirect, edge mode, read-back, audit events
          |
 hyper | axum | actix-web | poem | salvo | warp   (chosen by id)
```

HTTPAnt's own README says it does no ACME, storage or renewal: *"that is the source's job."* WebServAnt is that source. If WebServAnt ever needs
HTTPAnt to do one of those, the seam has moved (WebHostAnt `TLS-NEED-LIST.md`, "What stays outside HTTPAnt").

## 2. What WebHostAnt's local serving needs from it (the consumer contract)

Written from WebHostAnt's own documents (read), not guessed. WebHostAnt's rule: it does not build the in-process certificate path; WebServAnt does
(`EDGE-AND-AGNOSTICISM.md`, "Ruled: yes, and it comes first"). WebHostAnt's `plan -> apply -> read_back` shape, its secrets rule and its detection rule
all bind what WebServAnt must expose.

| # | WebHostAnt needs | Source (read) | WebServAnt provides |
|---|---|---|---|
| C1 | A certificate for a list of names, automatically, with a status: not-after date, issuer, last renewal result | DESIGN 5.3 trait: `ensure_certificate(names, challenge)`, `status()`, `renew_now()`, event stream | A `Certificates` handle with those four, Rust-typed. The in-process provider in WebHostAnt's capability table is this crate |
| C2 | A **staging** switch, so one test issuance can answer "is port 80/443 really reachable from outside" | DESIGN 5.4 ("asks the CA's staging environment to issue once") | `.staging(true)` (the directory URL is the only difference), and `check_reachable()` that does exactly one staging issuance and reports which step failed |
| C3 | Challenge types, with the person's network deciding: HTTP-01 and TLS-ALPN-01 | DESIGN 5.2, 5.4 | Both, using HTTPAnt's `ChallengeResponder` (HTTP-01 on port 80) and `acme_tls_cert` (TLS-ALPN-01 on 443). DNS-01 is not v1 (section 6) |
| C4 | Renewal with no restart; a failed renewal keeps the current certificate | TLS-NEED-LIST 15, 16; DESIGN section 9 | The store keeps the last good certificate until a new one validates; the source is asked per handshake, so the next connection gets the new one |
| C5 | Events for AuditAnt: issued, renewed, renewal failed, challenge served, certificate swapped, handshake refused. No key material | TLS-NEED-LIST 23 | A typed event stream, plus HTTPAnt's own `TlsAudit` passed through |
| C6 | Read-back of what the live listener really serves, not what the store says | TLS-NEED-LIST 22; CONSUMER-CONTRACT 2 | Re-exports HTTPAnt's served-certificate read-back (HTTPAnt 0.5.4) and exposes it next to `status()` |
| C7 | No secret value in a log, plan, audit event or command line; credential in memory for one run | CONSUMER-CONTRACT 3 rules 1 and 5 | Account key and certificate private keys are redacted types (HTTPAnt's `ServerCert` already is); tests use a canary and assert booleans. The account key's **storage** goes through SecretAnt's interface when it exists (section 4) |
| C8 | A server-side probe refuses private, loopback and link-local addresses unless the operator opts in | CONTRACT 3 rule 4; DESIGN 4.2 | The reverse proxy's upstream is the person's own app, so private addresses are allowed **for the configured upstream only**; any address derived from a request (a `Host` header, a redirect) is refused if it is non-public. HTTPAnt's client has `refuse_private_addresses` for the second case |
| C9 | A port-80 redirect to 443 that leaves the challenge path alone | TLS-NEED-LIST 18 | HTTPAnt's `HttpsRedirect` and `acme_challenge`, turned on by default when a certificate is wanted |
| C10 | Configured by settings (ConfigurAnt), not by code; same app, different settings | DESIGN section 0, 6; CONSUMER-CONTRACT hello-world | A serde `ServerSettings`-style struct (HTTPAnt did this in 0.5.34, `Server::apply`) that ConfigurAnt fills by dotted key. No dependency on ConfigurAnt, per HTTPAnt's reading of its API |
| C11 | Can run as a long-lived service started by an OS service definition | DESIGN 3.1 row 3, 7.4 (SupervisorAnt does not exist; WebHostAnt generates the unit) | A binary that logs to stdout, exits non-zero with a sentence on a fatal error, shuts down gracefully on SIGTERM / Ctrl-C using HTTPAnt's grace period, and needs no interactive input. WebHostAnt writes the service file; WebServAnt does not install itself |
| C12 | An app of the consumer's (Humboldt is a container app today) can sit behind it | CONTRACT 1, 4 | The reverse proxy (section 3.2). Humboldt's `/health` and its authenticated configuration endpoint are reached through it unchanged; WebServAnt never answers for the app |

**Not asked for, so not built:** presence detection, deployment, rollback, domain binding at a registrar. Those are WebHostAnt's.

## 3. Pieces

### 3.1 Static files

Serve a directory with: correct `Content-Type`, `ETag` / `Last-Modified` and `304`, `Range`, `index.html` for a directory, `HEAD`, path-traversal refusal
(`..`, encoded forms, symlinks leaving the root, Windows device names and alternate data streams: tested, since this box is Windows), and no directory
listing unless asked. Compression is a later row. **Adopt or build:** `hyper-staticfile` (0.10.1, last release 2024-07-01) and `tower-http`'s `ServeDir`
(0.7.1, 2026-08-31) are hyper/tower-bound, and HTTPAnt's handlers are its own `Handler` type, not tower services (read: `Server::layer` exists, so a
tower layer may fit; **not tried**). Recommendation: build a small file handler on HTTPAnt's types, because the traversal and symlink rules are where
the security lives and we want them tested through all six adapters by HTTPAnt's conformance harness; borrow nothing without reading its licence and
tests. Decision D4.

### 3.2 Reverse proxy

Forward matching requests (by path prefix and, later, by host) to an upstream URL using HTTPAnt's **client** half, streaming bodies both ways. Rules:

- `X-Forwarded-For` / `-Proto` / `-Host` set by us, and any value the client sent for them is overwritten, not appended to, unless the peer is a
  trusted proxy (HTTPAnt's `TrustedProxies`). Hop-by-hop headers are stripped (RFC 9110 section 7.6.1, **UNVERIFIED** against the text).
- Timeouts on connect, first byte and idle, all finite by default (HTTPAnt's client already has finite defaults).
- An upstream that is down gives `502` with a body that says so and nothing about the upstream's address.
- **WebSocket upgrade** is a known gap in HTTPAnt's server surface as read (no upgrade handler in the `Server` API); v1 either proxies it by tunnelling the
  upgraded connection (needs HTTPAnt support, an ask) or says plainly that it does not. Decision D3.
- HTTP/2 to the upstream is not v1 (the HTTPAnt client speaks HTTP/1.1, as far as read).

### 3.3 In-process ACME

The protocol work (RFC 8555) is not ours to write. See section 4. What WebServAnt owns around it: the order of operations, the challenge responders,
the certificate store, the renewal loop, rate-limit behaviour and the error sentences.

- **Certificate source.** Implements HTTPAnt's `CertSource`: `cert_for(server_name)` answers from an in-memory map filled from the store; `Ok(None)` for
  an unknown name (clean refusal, never another site's certificate, which is HTTPAnt's rule and its tested behaviour). `acme_tls_cert` answers the
  pending TLS-ALPN-01 certificate.
- **First issuance at startup** blocks listening on 443 for that name only for a bounded time; until it finishes the listener refuses that name's
  handshake (or serves a self-signed certificate **only if the person turned that on**, D6). The person sees a progress line, not silence.
- **Renewal.** A background task checks twice a day and renews when 30 days or fewer remain, or a third of the lifetime on shorter certificates
  (Let's Encrypt shortened lifetimes are **UNVERIFIED** as of this date; the rule must be read off the certificate, not hard-coded). It retries with
  backoff, keeps the current certificate on any failure, and emits an event; it does not hammer the CA after a rate-limit answer (honours `Retry-After`).
- **Store.** A directory (default under the platform's data directory, overridable), account key and per-name chain + key, written atomically,
  mode 0600 on Unix and an ACL restricted to the current user on Windows (WebHostAnt DESIGN 7.2 notes ConfigurAnt's `file` source checks Unix bits only,
  so Windows is ours to do and test). The store is behind a trait so SecretAnt can be the backend (section 4).
- **Names.** Only names the person listed. Wildcards need DNS-01 and are not v1. IP-address certificates are not v1 (**UNVERIFIED** whether the CA offers them).
- **Staging first.** The documented first run is `--staging`; the real CA is one flag away. Rate limits are the way a first-time user hurts themselves.

### 3.4 Settings and the command line

One settings struct, filled from a TOML file, environment, or arguments (ConfigurAnt fills it for consumers; the binary reads it directly for the
no-code path). Every field has a default or a sentence saying what is missing. Unknown fields are an error naming the field (a typo in `contact` must
not silently mean "no contact").

## 4. Build or adopt

Candidates and what was read (crates.io and GitHub APIs, 2026-11-08):

| Candidate | What it is | Read | Verdict |
|---|---|---|---|
| **`instant-acme`** | An async ACME **client** library (RFC 8555 plus profiles), no listener of its own | 0.8.5 published 2026-02-24; repo pushed 2026-11-02; Apache-2.0; 3.8 M downloads; not archived. Runtime deps: `tokio`, `http`, `rustls-pki-types`, `serde`, `base64`; `ring`, `aws-lc-rs`, `hyper-rustls`, `rcgen` and `x509-parser` are optional | **Adopt** as the protocol layer. It does not own the TLS acceptor, so it fits under HTTPAnt's `CertSource`. Not yet checked: which of its features we turn on, whether its default HTTP client is acceptable or we hand it HTTPAnt's client (it takes a pluggable HTTP client in recent versions, **UNVERIFIED**), and its `rcgen` line against ours (`rcgen` is 0.14.10) |
| **`rustls-acme`** | Resolver + acceptor with ACME built in; TLS-ALPN-01 only | 0.15.4 published 2026-08-11; Apache-2.0 (repo licence; the crate's own `license` field **not read**); depends on `futures-rustls`, `async-io`, `async-web-client`, `rcgen ^0.13` | **Do not adopt.** It brings its own acceptor and runtime pieces (`async-io`, `futures-rustls`) where HTTPAnt already owns the acceptor on `tokio-rustls`; it is TLS-ALPN-01 only (needs 443; useless when only 80 is open); and `rcgen ^0.13` would hold us off the 0.14 line the dependency rule wants. Its ideas (the challenge-certificate flow) are worth reading, not copying |
| **`acme2`**, **`acme-lib`** | Older ACME clients | 0.5.1 last published 2022-05-02; 0.9.1 on 2024-01-24 | **Do not adopt**: stale |
| **Caddy** | A Go web server with automatic HTTPS | v2.11.9 released 2026-11-05; Apache-2.0; 78.9 k stars | **Not a dependency.** It is a separate program in another language, so it cannot be chosen by id inside a Rust app, and WebHostAnt already ruled that Caddy "may be one edge adapter; it must never be the required one". It remains the thing to compare against on ease: WebServAnt's hello world must be about as easy as `caddy file-server --domain example.org` (**UNVERIFIED** flag spelling) |
| **Traefik** | A Go reverse proxy with ACME | v3.7.17 released 2026-10-30; MIT | Same verdict as Caddy: an external edge, not ours to wrap |
| **Own ACME client** | | | **No.** Certificate handling is where subtle mistakes cost the most (WebHostAnt DESIGN section 2). We write orchestration around a maintained protocol library, never the protocol |

A **risk to say plainly**: `instant-acme`'s maintenance is one person's hobby-scale project (the repo is `djc/instant-acme`; contributor count **not read**).
If it stalls, the protocol layer is replaceable behind our own small `AcmeClient` trait, which we should write so that swap is a day's work and so
that our tests can use a fake CA.

**Testing against a CA.** Unit and integration tests run against a local ACME test server (Pebble is the usual one, **UNVERIFIED** whether it runs on
this Windows box; if not, a self-hosted Linux runner per the "tests that cannot run on GitHub" rule). A test against Let's Encrypt **staging** over the
real network is a separate, opt-in job; it is never the only evidence for a behaviour.

**SecretAnt.** The account key and certificate keys are secrets. Until SecretAnt's interface exists, the store holds them in files with the restricted
permissions of section 3.3; when it does, the store trait gets a SecretAnt backend, and no key value passes through any log or plan. Not designed here
(SecretAnt's design is not on its main as of WebHostAnt's note; **not re-read**).

## 5. Safe defaults and the escape hatch

- TLS 1.2 minimum, 1.3 preferred, h2 offered with `http/1.1` (HTTPAnt defaults).
- HTTPAnt's request limits stay on (8 KiB target, 64 headers, 16 KiB of headers, 2 MiB body). **A reverse proxy in front of an upload endpoint needs a larger
  body limit per route**: a per-route limit is a setting, not a global switch.
- Plain HTTP serves only the ACME challenge and a redirect to HTTPS. "Serve plain HTTP" is an explicit setting for a LAN-only site.
- HSTS off by default (a wrong HSTS on a name is hard to undo), one setting to turn on.
- A failure to obtain a certificate **never** falls back to serving the site over HTTP or with a certificate for another name.
- Static files: no listing, no dotfiles, no symlinks out of the root, unless set.

## 6. What is NOT in v1

DNS-01 and wildcard certificates (needs a provider-token design and SecretAnt); a built-in load balancer or health-checked pools; caching; HTTP/3;
compression (a later row); authentication of any kind (basic auth, OAuth, mTLS to clients); a dynamic admin API or web UI (SecretAnt's UI will be hosted
by WebHostAnt on top of this, which is another project's work); config reload on signal (restart is cheap and renewal needs none); running as a Windows
service or systemd unit by itself (WebHostAnt generates the unit); multiple instances sharing one certificate store (a lock on the store directory is
the only promise); IP-address certificates; OCSP stapling (rustls's handling and the CA's end-of-OCSP position are **UNVERIFIED**); Caddy / nginx
config rendering (WebHostAnt's edge adapters).

## 7. Open decisions

Each has a recommendation; those that block row 1 are marked. None is for CireSnave unless the PM says so.

- **D1 (blocks row 1).** *Is the program one crate with a library and a binary, or a library crate plus a separate `webservant-cli`?* I recommend one package
  `webservant` with a `[[bin]]`, so `cargo install webservant` works and `cargo add webservant` gives the library; HTTPAnt's multi-crate layout was driven by a hub rule
  that does not apply to a program.
- **D2 (blocks row 1).** *Which version of HTTPAnt do we depend on?* I recommend `~0.5.22` now (published), and moving to 0.6 when the PM publishes it, since 0.6.0 is
  the breaking h2c change. Pre-1.0 `~` pins to the second number, so each move is a deliberate edit.
- **D3.** *Does v1 proxy WebSocket upgrades?* I recommend no for v1, stated in the README, and an ask to the HTTPAnt lane for an upgrade hook only when WebHostAnt or
  SecretAnt's UI needs one. Not blocking row 1 (static files and TLS come first).
- **D4.** *Build the static-file handler or adopt one?* I recommend build, on HTTPAnt's types, for the reasons in 3.1.
- **D5 (blocks row 1).** *Default CA and directory.* I recommend Let's Encrypt production as the default, `--staging` as the documented first run, and any other CA
  by its directory URL. A different default is the person's choice, not ours to bury in code.
- **D6.** *Serve a self-signed certificate while the first issuance runs, or refuse the handshake?* I recommend refuse (a browser warning teaches people to click
  through); a setting enables self-signed for local testing.
- **D7.** *Windows is a first-class platform for the store's permissions and for the binary.* I recommend yes, since this team's machines and the school installs
  may be Windows; it costs an ACL implementation and a Windows CI job, which HTTPAnt already runs.
- **D8 (blocks row 1).** *What does the first slice contain?* I recommend: the crate with the ten-line hello world serving **static files over plain HTTP and over TLS
  from a certificate file pair** (no ACME yet), so the consumer contract's shape, settings and the fresh-eyes README are proven before the ACME risk is taken on. ACME is slice 2.
- **D9.** *Patent sweep.* `PATENT-SWEEP.md` exists as a stub; the techniques here (automatic certificate issuance and renewal, ACME TLS-ALPN-01 on a shared
  listener, reverse proxy header rewriting) are old and heavily published, but I will sweep before the first release with functionality, as the rule requires.
