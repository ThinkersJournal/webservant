# WebServAnt lane HANDOFF

Written 2026-11-08 by the PM. Current state only; history is in git.

## Why this lane exists (CireSnave, 2026-11-08)
"If HTTPAnt is published and ready for use, we need to get to work on WebServAnt. It will need to exist before local serving can be built
into WebHostAnt."
Build order is CireSnave's (2026-10-28, verbatim in ROADMAP.md): HTTPAnt -> WebServAnt -> WebHostAnt local serving. The Humboldt installer
needs WebHostAnt to host locally and to install to remote hosts; its local half waits on you.

## State
- Repo ThinkersJournal/webservant, main = a 0.0.0 placeholder crate (name claimed), CI and licences, ROADMAP.md (one paragraph). No code.
  Your worktree: C:\Projects\webservant-wt-lane on branch lane/webservant. The primary checkout C:\Projects\webservant stays on main; never switch it.
- HTTPAnt is published: `httpant` 0.5.22 on crates.io (server, client, TLS: edge mode, `CertSource` trait and `tls` feature, ACME challenge prefix,
  client `https` with bundled roots). Depend on crates.io versions only (no paths, no git deps), pin with `~`. 0.6.x on HTTPAnt main is not published yet.
- Consumers: WebHostAnt (ThinkersJournal/WebHostAnt: `docs/CONSUMER-CONTRACT.md`, `docs/TLS-NEED-LIST.md`, `docs/EDGE-AND-AGNOSTICISM.md`,
  `docs/DESIGN.md`) and, through it, the Humboldt installer (`docs/INSTALLER-DESIGN.md` on origin/docs/installer-design in the Humboldt repo).

## First task (do not wait for a [TASK])
1. Read HTTPAnt's `README.md`, `docs/` and ROADMAP (on origin/main of ThinkersJournal/httpant) and WebHostAnt's four docs above. Read, do not assume.
2. Write `docs/DESIGN.md` (draft, docs only) and replace ROADMAP.md with rows: what WebServAnt is (a web server program that works with any Rust
   web library, built on HTTPAnt: static files, reverse proxy, in-process ACME certificates, a ten-line hello world), what WebHostAnt's local
   serving needs from it (consumer contract, written from WebHostAnt's own needs, not guessed), build-or-adopt (Caddy, other servers, ACME
   libraries: name what we would adopt and why not), what is NOT in v1. "Easy is the product" (EXPECTATIONS 6.24) is a release gate.
3. List every open decision with your recommendation in full plain sentences and send the PM an [ASK] with those that block row 1. The PM
   puts them on CireSnave's board; never ask him directly.
4. Then row 1, red-first, one PR per slice; send [READY] after each.

## Rules that bite here
- Secrets, and anything that touches them, via SecretAnt's interface when it exists (SecretAnt lane runs); never print a value.
- Patent sweep: `PATENT-SWEEP.md` exists; add rows for any technique before it ships.
- Cost: Sonnet only; short messages; headers per MESSAGE-FORMATS.md. You are on Synapse.
- Open issues and PRs as the lane account. Never `gh auth switch`.

## Waiting on
Nothing. The PM gates and publishes; WebHostAnt local serving waits for you.
