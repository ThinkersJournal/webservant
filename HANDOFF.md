# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #10 (0.3.4: ACME store private; Windows CI found explicit-ACE gap, fixed in the same PR). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (`webservant check` on his VPS; needs his public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- #11 merged (0.4.0: per-route proxy timeouts, site-wide body limit as settings). PR for slice10-root-proxy (0.4.1 proposed): a proxy at "/" with files FIRST, then the app (my reading of the PM's "files as fallback": say so in the [READY]; the other reading, app first, cannot work because the proxy cannot know the app would 404). 5 mutants checked.
- NEXT ROW (PM): docs/QUICKSTART.md = the ten-line hello world and the Let's Encrypt staging run as a numbered list a non-expert can follow (what to point DNS at, port 80, the exact `webservant check` command, what success looks like, what to do if each step fails), then a FRESH-EYES TRIAL of it (an agent given only the README reaches the result; method: HTTPAnt docs/FRESH-EYES-TRIAL.md at origin/main). Release gate (EXPECTATIONS 6.24).
- Remaining: connect timeout is validated but not exercised by a test (no portable blackhole address); ConfigurAnt env/flag layers not exercised; `--config FILE` (TOML dependency); C1 add a name to a running site (only if asked); no run against Let's Encrypt staging; per-route body limit (not possible with HTTPAnt's site-wide Limits).
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-10 PR (CI, threads).
2. docs/QUICKSTART.md, then the fresh-eyes trial (see NEXT ROW above), then idle (restart rule) unless the PM has more.

WAIT: slice-10 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice10-root-proxy --json number,mergedAt | wake: 2026-11-08T17:00:00Z
