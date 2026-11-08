# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 design, #2 (0.1.0), #3 ACME HTTP-01 (0.2.0), #4 proxy (0.2.1), #5 consumer contract + patent sweep (0.2.2). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (CireSnave's public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR #6 (slice4-contract-gaps, 0.3.0): all CI green incl. Linux SIGTERM test and Pebble; waits on the PM merge.
- Branch slice5-settings (0.3.1 proposed), committed LOCALLY, stacked on slice4, NOT pushed: C10 `Settings`/`Site::from_settings` (src/settings.rs, tests/settings.rs, CLI goes through it). After #6 merges: `git fetch`, `git rebase --onto origin/main eddb5b8 slice5-settings`, push, open the PR, [READY]. Not tried with ConfigurAnt itself.
- Remaining contract gaps: C2 check_reachable(), C1 add a name to a running site (only if asked), Windows ACL on the store, proxy timeouts as settings, proxy at "/" next to files, `--config FILE` (TOML dependency).
- Untracked `supertool` file is not ours.

## Next
1. When #6 merges, do the slice5 rebase above and open its PR.
2. Then C2 `check_reachable()` (one staging issuance that reports which step failed) and trying Settings with ConfigurAnt.

WAIT: PR #6 merge (then rebase and push slice5-settings) | check: gh pr view 6 --repo ThinkersJournal/webservant --json state,mergedAt | wake: 2026-11-08T11:00:00Z
