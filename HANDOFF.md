# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #10 (0.3.4: ACME store private; Windows CI found explicit-ACE gap, fixed in the same PR). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (`webservant check` on his VPS; needs his public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR for slice9-proxy-limits (0.4.0 proposed, breaking: public structs gained fields): per-route proxy timeouts (`ProxyOptions`, `proxy_with`, `*_timeout_secs`) and the site-wide `max_body_bytes` / `--max-body`, in Settings and the ConfigurAnt schema. Three mutants caught (idle, response timeout, body limit).
- Remaining: connect timeout is validated but not exercised by a test (no portable blackhole address); ConfigurAnt env/flag layers not exercised; `--config FILE` (TOML dependency); C1 add a name to a running site (only if asked); proxy at "/" next to files; no run against Let's Encrypt staging; per-route body limit (not possible with HTTPAnt's site-wide Limits).
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-9 PR (CI, threads).
2. Then proxy at "/" next to files (files as fallback), or ask the PM what is wanted next: the roadmap is otherwise done up to what needs CireSnave (staging run, sweep acceptance, publish).

WAIT: slice-9 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice9-proxy-limits --json number,mergedAt | wake: 2026-11-08T16:00:00Z
