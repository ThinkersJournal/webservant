# Patent sweep

No code exists beyond an empty placeholder crate. A sweep (web servers, TLS certificate automation, reverse proxying) is a precondition of the first release that contains functionality. Not legal advice; no attorney exists.

## Licence reads (2026-11-08)

Not a patent sweep. Dependency licences read from the GitHub and crates.io APIs: `instant-acme` 0.8.5, Apache-2.0 (repo licence field; the crate's own `license` field still to be read before adoption); `rustls-acme` 0.15.4, repo Apache-2.0, not adopted; Caddy v2.11.9 Apache-2.0 and Traefik v3.7.17 MIT, not dependencies. Patents sweep for the shipped techniques remains a precondition of the first release with functionality (ROADMAP row 5).

## Techniques in the tree, not yet swept (2026-11-08)

Slice 1 (0.1.0, not published) contains: static file serving with conditional requests and byte ranges (RFC 9110), path confinement under a root, and TLS termination through HTTPAnt. All are long-standing and specified in public standards, but **no sweep has been done**; it is ROADMAP row 5 and a precondition of publishing.
