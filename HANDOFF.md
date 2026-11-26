# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-26 by the lane. Worktree C:\Projects\webservant-wt-lane (branch lane/webservant-acme-audit). Repo nwo ThinkersJournal/webservant. Primary checkout stays on main. Publishing is the PM's.

## State
Everything the roadmap names is built and merged (#1 to #25). PR #26 open (in the gate, pm-ants/pm-assist): ACME + `check` audit docs from the registry 0.4.13 binary against Pebble, plus CI pin of the portfolio workflow to coderipper 467fd259 (CodeRipper 0.48.1, pm-ants TASK 2026-11-26). Version number is the gate's.

## WAIT
Waiting on merge of #26 (`gh pr view 26 --json state`). After merge: delete remote branch + `git worktree` cleanup per CLAUDE.md s3, start a fresh branch from origin/main. Read the advisory `portfolio` job by check name at the pin; a new Medium is dispositioned, never reverted.

## Audit rows
- Done: static, TLS-from-PEM, proxy, junction escape, ACME, `check`.
- SIGTERM: DONE 2026-11-26 in docker rust:latest, registry 0.4.13 binary: `webservant serve --files ./public` answered 200, `kill -TERM` -> clean exit 0. NOT yet in README: next PR must drop "SIGTERM" from the not-exercised list (README line ~76) and say Linux/container only (not systemd, not Windows). Held out of #26 so as not to move a gated head.
- Remaining: non-default adapters, Settings, file-symlink escape on Windows. Next PR window is a new UTC day after #26 merges; one PR per day.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Staging runs were by an AI agent. Never run: production, failed renewal on a real CA, reboot with the systemd unit, phone on mobile data.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, the PM allocates; one PR per day per project.
- Squash-merge: rebase with `git rebase --onto origin/main <old-base-tip> <branch>`.
- Shell eats backslashes/apostrophes in heredocs; files are CRLF. Sign every PR body/comment last line "— lane: webservant". Sonnet only for subagents. Never `gh auth switch`. Untracked `supertool` is not ours: `git add -A . ':!supertool'`.
- CLI flags: `--bind` (not `--addr`); see `webservant --help`.
