# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- PR #1 (design) merged. Branch slice1-static in C:\Projects\webservant-wt-lane: slice 1 built (src/lib.rs Site, files.rs, main.rs CLI, tests/files.rs + tls.rs, examples/hello.rs run in CI, README, features axum/actix-web/poem/salvo/warp). Version 0.1.0 is a PROPOSAL; PM allocates at the gate.
- Local: fmt, clippy -D warnings, 13 tests, example, doc, cargo deny all green. Mutants caught: root check, component check, device names, If-None-Match.
- Not verified: other adapters (only hyper tested); Linux (only this Windows box; CI covers both); HTTP/2 not offered yet (no `http2` feature).
- Untracked `supertool` file is not ours.

## Next
1. Open the slice-1 PR, [READY] to PM, check threads/CI.
2. Then ROADMAP row 2 (ACME via instant-acme behind AcmeClient; staging run recorded in the PR before anything touches Let's Encrypt production) and row 3 (reverse proxy); read instant-acme's own `license` field first.

WAIT: slice-1 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice1-static --json number,mergedAt | wake: 2026-11-08T08:00:00Z
