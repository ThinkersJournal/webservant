# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #7 (design; 0.1.0 files/TLS; 0.2.0 ACME HTTP-01; 0.2.1 proxy; 0.2.2 contract + patent sweep; 0.3.0 read-back/TLS events/SIGTERM/ALPN code removed; 0.3.1 Settings). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (needs his public name + port 80; `webservant check` is the tool for it). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR for slice6-check (0.3.2 proposed): `Site::check_reachable()` / `Reachability` / `webservant check` (contract C2). Fake-CA tests (mutants: production not replaced, throwaway store not used: both caught) + Pebble test with a REAL failed validation.
- Remaining contract gaps: Settings not tried with ConfigurAnt; C1 add a name to a running site (only if asked); Windows ACL on the store; proxy timeouts as settings; proxy at "/" next to files; `--config FILE` (TOML dependency); no run against Let's Encrypt staging.
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-6 PR (CI, threads).
2. Then try Settings with ConfigurAnt (read its API at origin/main first), Windows ACL on the store, proxy settings.

WAIT: slice-6 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice6-check --json number,mergedAt | wake: 2026-11-08T13:00:00Z
