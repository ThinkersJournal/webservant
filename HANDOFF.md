# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-16 by the lane. Worktree C:\Projects\webservant-wt-lane (branch lane/webservant-next, tracks origin/main). Repo nwo ThinkersJournal/webservant. Primary checkout stays on main. 0.4.14 merged (#25, main 8346ad3); publishing is the PM's.

## State
Everything the roadmap names is built and merged (#1 to #24). Static files, HTTPS from PEM, ACME HTTP-01 (Pebble in Linux CI), renewal, reverse proxy, check_reachable, Settings, adapters, QUICKSTART, patent sweep. Remote branch lane/webservant-0.4.13 deleted (verified 404, main 200 as control); local branch deleted.

## In flight
Branch lane/webservant-acme-audit (from main 8346ad3, 0.4.14): README claim-to-command audit continues. Done and merged in #25: static, TLS-from-PEM, proxy, junction escape, LICENSE-APACHE canonical. Remaining audit rows (pm-ants 2026-11-19): ACME, `check`, SIGTERM, adapters, Settings, file-symlink escape. One PR per day; #25 was 11-20's, so next PR 11-21 UTC via [READY] to pm-assist cc pm-ants.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Staging runs were by an AI agent. Never run: production, failed renewal on a real CA, reboot with the systemd unit, phone on mobile data. Part 2 not re-run from the registry.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, the PM allocates; one PR per day per project.
- Squash-merge: rebase with `git rebase --onto origin/main <old-base-tip> <branch>`; a stale sequencer can block `git switch` (`git cherry-pick --quit`).
- Shell eats backslashes/apostrophes in heredocs; files are CRLF. Sign every PR body/comment last line "— lane: webservant". Sonnet only for subagents. Never `gh auth switch`. Untracked `supertool` is not ours: `git add -A . ':!supertool'`.

## Next
1. ACME and `webservant check` audit rows (Pebble via Docker if present; no domain here), then SIGTERM, adapters, Settings. Keep the not-exercised list in README honest.
