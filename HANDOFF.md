# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-26 by the lane. Worktree C:\Projects\webservant-wt-lane. Repo ThinkersJournal/webservant. Primary checkout stays on main. Publishing is the PM's.

## State
Roadmap built and merged (#1 to #26; #26 = d24cdf73, 0.4.15 next, remote branch gone). Lane branch `lane/webservant-sigterm-audit` (from origin/main) holds one local commit, not pushed: README drops SIGTERM from the not-exercised list (Linux/container only). Open the PR on the next UTC day (one PR per day), batching more audit rows first.

## Next
- Audit rows left: non-default adapters, Settings, file-symlink escape on Windows. Add each to the same branch, then ONE PR; send [READY] to pm-assist cc pm-ants.
- Check `gh pr list` / `git ls-remote origin main` yourself; never wait to be told.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Never run: production, failed renewal on a real CA, systemd reboot, phone on mobile data.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, PM allocates. Squash-merge: `git rebase --onto origin/main <old-base-tip> <branch>`.
- Files are CRLF; sign PR text "— lane: webservant". Sonnet only. Never `gh auth switch`. Untracked `supertool` is not ours: never `git add -A`/`commit -a`; add by path.
- CLI flags: `--bind` (not `--addr`).
