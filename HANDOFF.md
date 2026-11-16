# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-16 by the lane. Worktree C:\Projects\webservant-wt-lane (branch lane/webservant-0411 for PR #22; lane/webservant is a stale local branch). Repo nwo ThinkersJournal/webservant. Primary checkout stays on main. Published: 0.4.4 only (tag v0.4.4).

## State
Everything the roadmap names is built and merged (#1 to #21, 0.4.10 on main). Static files, HTTPS from PEM, ACME HTTP-01 via instant-acme (Pebble in Linux CI), renewal, reverse proxy, check_reachable / `webservant check`, Settings, adapters tested per web library, QUICKSTART, patent sweep. Board 402 ruled A (publish on our sweep after the staging run, which is done by an AI agent on Let's Encrypt staging, Debian 13).

## In flight
- #22 MERGED (0.4.11: heading fix, cargo update). PM will publish ONE release covering 0.4.5-0.4.x, then tag; he allocates the number.
- PR for 0.4.12 (branch lane/webservant-0412, docs only): all 2 major, 8 minor, 3 nit findings of the httpant lane's Part 2 read (PM task quickstart-part2-findings) fixed in QUICKSTART. [READY] sent to the PM once the PR was open; the release waits on it. If new findings arrive, fix in the same PR before merge.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455, PM raising). Staging runs were by an AI agent. Never run: production, failed renewal on a real CA, reboot with the systemd unit, phone on mobile data.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, the PM allocates; one PR per day per project (a PM-requested follow-up is the exception).
- Squash-merge: rebase with `git rebase --onto origin/main <old-base-tip> <branch>`.
- Shell on this box eats backslashes/apostrophes in heredocs and python; use Write/Edit; files are CRLF so Python `\n` matches fail. Read back before claiming.
- Sign every PR body/comment/merge note with a last line "— lane: webservant". Sonnet only for subagents. Never `gh auth switch`. Untracked `supertool` is not ours: `git add -A . ':!supertool'`.

## Next
1. `git fetch`; `lane-restart waits`; synapse fetch; `gh pr list -R ThinkersJournal/webservant --head lane/webservant-0412` (checks, threads).
2. Fix any capant/PM findings; on merge do branch and worktree cleanup per CLAUDE.md section 3.

WAIT: 0.4.12 PR merge/CI, new reader findings, or PM publish | check: git fetch -q && git log --oneline origin/main -3; gh pr list -R ThinkersJournal/webservant --head lane/webservant-0412 --json state,statusCheckRollup | wake: 2026-11-16T22:30:00Z
