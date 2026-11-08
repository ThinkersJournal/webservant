# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 design, #2 (0.1.0), #3 ACME HTTP-01 (0.2.0), #4 proxy (0.2.1), #5 consumer contract + patent sweep (0.2.2). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (CireSnave's public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- #6 merged (0.3.0). PR for slice5-settings (0.3.1 proposed, C10 `Settings`/`Site::from_settings`) is open; not tried with ConfigurAnt itself.
- Remaining contract gaps: C2 check_reachable(), C1 add a name to a running site (only if asked), Windows ACL on the store, proxy timeouts as settings, proxy at "/" next to files, `--config FILE` (TOML dependency).
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-5 PR (CI, threads).
2. Then C2 `check_reachable()` (one staging issuance that reports which step failed) and trying Settings with ConfigurAnt.

WAIT: slice-5 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice5-settings --json number,mergedAt | wake: 2026-11-08T12:00:00Z
