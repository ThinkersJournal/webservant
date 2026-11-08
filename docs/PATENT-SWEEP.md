# Patent sweep: WebServAnt (webservant, line 0.2.x) @ main `ddc8ad3` (0.2.1)

Date: 2026-11-08; Run by: WebServAnt lane, Claude Sonnet 5.5; Method: AI-assisted search and read (web search limited to patents.google.com, then claims read with `C:\Projects\tools\patent-claims.py`); **NOT legal advice; NOT a freedom-to-operate opinion.**

Covers the code on `main` at `ddc8ad3` (0.2.1: static files, TLS from a certificate pair, ACME HTTP-01 certificates, reverse proxy). Later commits since are docs only. It does **not** cover HTTPAnt's own mechanisms (swept in HTTPAnt's `docs/PATENT-SWEEP.md`: interchangeable adapters, request limits, the `CertSource` per-handshake certificate lookup, trusted-proxy address) except where WebServAnt adds to them. A new family of capability (TLS-ALPN-01, DNS-01, WebSocket forwarding, a SecretAnt-backed store) needs a re-sweep (PATENT-SWEEP-PROCESS.md).

## Ideas swept

1. A static-file server resolves the request path to a canonical filesystem path and serves it only if it lies under the canonical root; a path that is forbidden (traversal, symlink out of the root, a drive or alternate-stream marker, a device name, a trailing dot or space, a dotfile) and a path that is missing get the identical `404`, so a probe learns nothing about what exists.
2. File responses carry a weak entity tag built from the file's size and modification time to nanosecond precision, `If-None-Match` taking precedence over `If-Modified-Since`, and a single byte range (`a-b`, `a-`, `-n`) answered `206`, an unsatisfiable one `416`, anything not understood the whole file.
3. A web server obtains its own TLS certificate in process over ACME (RFC 8555): it answers the HTTP-01 challenge from the same server that serves the site (the challenge path is answered before any route), redirects plain HTTP to HTTPS on the same machine, and serves HTTPS from the result, with no outside program.
4. A certificate source that holds the issued certificate in memory keyed by lower-cased server name and refuses (never substitutes another name's certificate) a handshake whose name it does not hold or that sends no name. *(The per-handshake lookup itself is HTTPAnt's idea 7; the keyed in-memory source is ours.)*
5. Renewal is due when a third of the certificate's own lifetime or less remains (read from the certificate's dates, not a constant), checked on a timer; a failed renewal leaves the current certificate in service and is retried with a back-off from one minute up to an hour, without stopping.
6. A certificate store that is separated by certificate authority directory (staging and production accounts and certificates never mix), writes each file atomically, and writes the list of names last so that a crash leaves an unused certificate and never one for the wrong names; the stored certificate is reused at start unless the names changed or renewal is due.
7. A reverse proxy for a path prefix that removes the prefix, drops every forwarding header the client supplied (`Forwarded`, `X-Forwarded-*`), writes its own from the client address and scheme the server itself established, removes hop-by-hop headers and every header named by `Connection` in both directions, refuses dot segments (raw or percent-encoded) before forwarding, refuses an `Upgrade` request out loud, and answers an upstream failure `502` or `504` without naming the upstream.
8. A TLS-ALPN-01 validation certificate carrying the `acmeIdentifier` extension built from the key authorization's digest (code is in the tree; **not reachable from the public API** until HTTPAnt accepts such a certificate).

## Queries (source, query, date)

All 2026-11-08, web search (US), standard mode, limited to `patents.google.com`. Search snippets are not claims; only the claims read below were relied on. This is a general web index restricted to one domain, not Google Patents' own query interface: no boolean operators, classification codes, or date and assignee filters.

| # | idea | query |
|---|---|---|
| Q1 | 3, 5 | patent web server automatically obtain TLS certificate ACME challenge response same server renew before expiry without restart |
| Q2 | 1 | patent static file server path traversal prevention canonical path root directory symbolic link resolve verify within document root |
| Q3 | 7 | patent reverse proxy remove hop-by-hop headers replace client supplied forwarded-for header with determined client address |
| Q4 | 5 | patent certificate renewal when remaining validity falls below fraction of lifetime retain current certificate on failure retry backoff |
| Q5 | 3 | patent ACME protocol http-01 challenge token key authorization served by web server automated certificate management environment |
| Q6 | 4 | patent server selects TLS certificate by server name indication from in-memory store updated at runtime new certificate used for subsequent connections |
| Q7 | 2 | patent HTTP static file serving conditional request entity tag derived from file size and modification time byte range partial content |
| Q8 | 3 | patent web server single process integrated automatic HTTPS certificate provisioning redirect port 80 to 443 challenge path |

Ideas 6 and 8 got **no query of their own**: 6 (atomic writes, a names file last) is ordinary engineering with no distinguishing mechanism, and 8 is not reachable from the shipped API. That is a gap, not a clearance.

## Hits

Claims read with `patent-claims.py`. Status is Google Patents', not the official record. "Read" says how much of claim 1 was read.

| # | patent | title | assignee | status | claim(s) read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|
| H1 | US20050081028A1 | Method to automate the renewal of digital certificates | Imcentric (now CyberArk) | Expired (priority 2003) | 1 in full | Automatic renewal of server certificates | A separate managing device is notified, makes the managed server produce a CSR, sends it to a CA and installs the result; our server asks the CA itself and no managing device exists | clear (expired; no overlap on claim 1) |
| H2 | US7650497B2 | Automated digital certificate renewer | Venafi | Active (est. expiry 2027-01) | 1 through element (iv) of its managing-device system | Automatic renewal | Claim 1 is a managing device that receives notifications **from a certificate authority**, identifies a managed server and makes it generate a CSR; we have no managing device and take no CA notifications. Elements after (iv) not read: they further limit the same architecture | clear on the elements read; expires about 2027-01 on Google Patents' estimate |
| H3 | US20050078830A1 | Method for automated installation of digital certificates to network servers | Imcentric | Abandoned | 1 (first elements) | Installing a certificate on servers | Centralised installer that detects server type and runs type-specific install steps; we write to our own in-memory source and our own store | clear |
| H4 | US7120929B2 | Automated authentication, processing and issuance of digital certificates | GeoTrust | Expired | 1 in full | Domain control vetting for a certificate | Claim 1: contact an approver by e-mail address tied to the domain so they can approve; ACME HTTP-01 contacts nobody, it serves a token | clear (expired; no overlap) |
| H5 | US20080086633A1 | Method to handle SSL certificate expiration and renewal | Individual | Expired (fee) | 1, 9 in full | Replace a current certificate; roll back to the old one | Claims require sending the new certificate to a **second device**, verifying that it installed there, archiving the old one and activating the new; ours replaces in-process, with no second device and no archive step (the old one simply stays in service if the new one fails) | clear (expired; and no overlap on every independent claim) |
| H6 | US9215232B2 | Certificate renewal | Red Hat | Active (est. 2030-12) | 1, 11, 17 in full | Automatic renewal on an expiration condition | Claims search a **CA database** for certificates meeting the condition and renew by **reusing the key** and setting a new expiry; we are not a CA, search no database and generate a new key for each certificate | clear; **re-check if we ever reuse a key across renewals or add CA-side software** |
| H7 | US8738901B2 | Automatic certificate renewal | Red Hat | Active (est. 2030-12) | 1, 12, 18 in full | Same family as H6 | Same CA-database search and batch renewal requests approved by the system; absent from our design | clear (same note as H6) |
| H8 | US8321662B2 | Certificate renewal using secure handshake | IBM | Expired (fee) | 1 in full | Expiry detected, new certificate requested | Claim 1 detects expiry in a **certificate received from a holder during a handshake**, notifies the holder and requests a new certificate on its behalf; we check our own certificate on a timer, not a peer's during a handshake | clear (expired; no overlap) |
| H9 | US9762571B2 | Securing connections to unsecure internet resources | Team Internet AG | Active (est. 2036-02) | 1 in full | Obtains a certificate from a CA for a domain automatically, places an ID code on the domain, and serves the certificate in later connections | Claim 1 requires **intercepting data packets** from client terminals, identifying a secure connection request to an unsecure domain, determining the client is legitimate, and locating the domain name **in a ClientHello** to trigger obtaining the certificate. WebServAnt obtains certificates for names the operator configured, at start and on a timer; **nothing is triggered by an incoming ClientHello and no packets are intercepted** | clear; **re-check if on-demand issuance (obtain on the first handshake for an unknown name) is ever added** |
| H10 | US11218329B2 | Certificate generation with fallback certificates | Arris | Active (est. 2040-07) | 1 (first elements) | Serves something while a certificate is not ready | Claim 1: a client device presents a fallback certificate from read-only memory until its generated certificate is complete; our server refuses the handshake until a certificate exists and has no fallback certificate (self-signed serving is off) | clear |
| H11 | US20120203927A1 | Symbolic-link identifying | IBM | Expired (fee) | 1, 2, 3 | A symlink in a path is detected on a file server | Claim 1 traverses the absolute path **using a file descriptor of each file** to identify a symlink; we canonicalise the path and check that the result lies under the canonical root, with no per-component descriptor traversal | clear (expired; different mechanism) |
| H12 | US20050262357A1 | Network access using reverse proxy | AEP Networks | Abandoned | 1 | A reverse proxy forwards translated requests | Claim 1 requires authenticating the user and validating the request before translating and forwarding; our proxy authenticates nobody | clear |
| H13 | US12388652B2 | Header for conveying trustful client address | IBM | Active (est. 2042-12) | 1, 6, 9 in full | Conveying the client address to a web server through a proxy | Every independent claim requires a header carrying the address, a UUID, a timestamp, a lifetime, URL and a hash, with a UUID cache that invalidates replays; we send a plain `X-Forwarded-For` and keep no cache | clear |
| H14 | US6199107B1 | Partial file caching and read range resume | Microsoft | Expired (fee) | 1 in full | Byte ranges and `ETag` / `Last-Modified` are mentioned in its description | Claim 1 is a **client** cache merging a partial cached file with a range fetched from a server; we are the server answering a range | clear |

## Not read (first pass)

Surfaced by the queries and judged out of scope from their titles and snippets only, so **`unread`, not `clear`:** US7707406B2 and CA2502480C (certificate renewal in a CA infrastructure), US9130758B2 (renewal of expired certificates), US20110126001A1, US20050074124A1 (management of SSL/TLS certificates), US8898457B2, US10277632B2, US7395424B2, US20110161662A1 (updating a digital certificate automatically), US20180077121A1 (reverse proxy for a local network), US20050078668A1, US20050076201A1, the "System and method for media content streaming" family (US8966106, US8677005, US10432683) and US20070162571A1, US8019885B2 (media serving by byte range). Q5 (an ACME HTTP-01 patent) surfaced **only unrelated challenge-response authentication results**; ACME's own standardisation and any IPR disclosures to the IETF were **not** searched, and that is the most obvious thing a reviewer would ask about.

## Needs review (to the PM)

No hit has an unresolved overlap on the claims read. Two live patents deserve the PM's eye because they are close in subject and their expiry is far off: H9 (Team Internet, to about 2036) is avoided only because we do not issue on demand from a ClientHello, and H6/H7 (Red Hat, to about 2030) only because we are not CA-side software and never reuse keys. Both are design facts that a future change could cross.

## Limitations

- AI-assisted search and read; **not legal advice, not a freedom-to-operate opinion**, and no attorney has reviewed it.
- Eight queries over a domain-limited general web index, one pass. No boolean search of USPTO, EPO or WIPO, no CPC classification (for example H04L 9/32, H04L 63, H04L 67), no inventor or assignee searches (Let's Encrypt/ISRG, Cloudflare, Google, Akamai, F5 and Microsoft were not searched by name), no search of the ACME working group's IPR disclosures, no non-English patents.
- Idea 2 (entity tags and ranges) and the standard header handling in idea 7 are specified in public RFCs (9110, 9111, 7233 lineage) and long deployed in open-source servers; only one query touched each, and an RFC being public does not clear a patent.
- Statuses and expiry dates are Google Patents' estimates. Claim text is machine-converted. Several granted claims were read in full; where only the first elements were read the table says so.
- WebServAnt depends on `instant-acme` for the protocol; its licence (Apache-2.0) was read, its patent position was not, and the Apache licence's patent grant covers the contributors' own patents only.
- Coverage of anything added after `ddc8ad3` is none.
