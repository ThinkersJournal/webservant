# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-16 by the lane. Worktree C:\Projects\webservant-wt-lane (branch lane/webservant-next, tracks origin/main). Repo nwo ThinkersJournal/webservant. Primary checkout stays on main. PUBLISHED: 0.4.12 on crates.io (PM, tag v0.4.12). 0.4.13 PUBLISHED by PM 2026-11-19 18:38Z (tag v0.4.13). Part 1 re-run from the registry on 0.4.13 (CLI + Rust way, curl only) and reported to pm-ants.

## State
Everything the roadmap names is built and merged (#1 to #24). Static files, HTTPS from PEM, ACME HTTP-01 (Pebble in Linux CI), renewal, reverse proxy, check_reachable, Settings, adapters, QUICKSTART, patent sweep. Remote branch lane/webservant-0.4.13 deleted (verified 404, main 200 as control); local branch deleted.

## In flight
Nothing. No WAIT lines.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Staging runs were by an AI agent. Never run: production, failed renewal on a real CA, reboot with the systemd unit, phone on mobile data. Part 2 not re-run from the registry.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, the PM allocates; one PR per day per project.
- Squash-merge: rebase with `git rebase --onto origin/main <old-base-tip> <branch>`; a stale sequencer can block `git switch` (`git cherry-pick --quit`).
- Shell eats backslashes/apostrophes in heredocs; files are CRLF. Sign every PR body/comment last line "— lane: webservant". Sonnet only for subagents. Never `gh auth switch`. Untracked `supertool` is not ours: `git add -A . ':!supertool'`.

## Next
1. Nothing unblocked, nothing owed. 0.4.13 published and Part 1 re-run reported to pm-ants. Stray mysite.exe/webservant.exe from that run may hold port 8080 (taskkill denied; kill by PID). Await a [TASK] or re-sweep the out-of-first-version list.
