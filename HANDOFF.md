# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane. Worktree C:\Projects\webservant-wt-lane on branch lane/webservant (= origin/main
at 6874883, 0.4.3). The primary checkout C:\Projects\webservant stays on main. This file's rewrite is merged (#14, 0.4.3, docs only).

## Why (CireSnave 2026-10-28, ROADMAP.md)
Build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published and depended on as `~0.5.22`.

## State: everything the roadmap names is built and merged (#1 to #13, 0.4.2), nothing is published
Built: static files (traversal/symlink/range/304 tests), HTTPS from a PEM pair, ACME HTTP-01 certificates via `instant-acme` behind our `AcmeClient`
(fake CA + Pebble in Docker, required in Linux CI), renewal that keeps the last good certificate, private store (Unix 0700, Windows icacls), reverse
proxy (prefix and `/` with files first, per-route timeouts, site-wide body limit), `check_reachable` / `webservant check`, `Settings` + ConfigurAnt
schema (file layer tried with ConfigurAnt 0.6.3), live read-back, TLS events, SIGTERM, QUICKSTART, three fresh-eyes runs, patent sweep.

Waiting on people, not on code:
- **CireSnave** (via the PM): board 402, whether the thin patent sweep (`docs/PATENT-SWEEP.md`, 14 hits read, 8 queries, thin by its own
  Limitations) meets the first-publish bar; and the Let's Encrypt STAGING run on his VPS (`webservant check`; needs his public name and port 80),
  where the PM will ask him to follow `docs/QUICKSTART.md` Part 2 verbatim and send every failure. **Every failure he sends goes into QUICKSTART and
  the trial record.** Nothing may touch Let's Encrypt production before a staging run is recorded in a PR.
- **The PM**: version allocation (always: I only propose), merge, publish. The PM publishes only after the sweep decision and the staging run.
- **HTTPAnt lane**: the fix for `ServerCert` rejecting a critical `acmeIdentifier` extension (TLS-ALPN-01 is OUT of the first version anyway).

Keep OUT of the first version (each needs a patent re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Honest gaps (do not claim more than this)
- No agent or person has RUN QUICKSTART Part 2 on a real Linux server. The three fresh-eyes runs only read it (Windows; they were forbidden from
  touching a real CA). The last round of fixes after run 3 was not re-tried.
- `webservant check` passed against Let's Encrypt STAGING on a real Debian 13 VPS (2026-11-09, run by the PM's agent over SSH, trial run 4). Never run: `serve` with a real certificate and renewal, production, the systemd unit, the phone-on-mobile-data step.
- `cargo install webservant` from crates.io is untested (not published); QUICKSTART has a "until the first release" block to DELETE at publish.
- The connect timeout is validated but untested; ConfigurAnt's environment and flag layers are not exercised; no `--config FILE` (needs a TOML dep).
- Not built, only if asked: add a name to a running site (C1); per-route body limit (HTTPAnt's limit is site-wide).

## Rules that bit here
- Version: I propose, the PM allocates; every pushed change bumps (pre-1.0 the second number is "major"). Docs-only took 0.x.1+ too.
- Squash-merge: rebase the next branch with `git rebase --onto origin/main <old-base-tip> <branch>`.
- Shell quoting on this box: heredocs and python strings eat backslashes and apostrophes; write scripts with the Write tool, edit with Edit, and
  anchor Python replacements on text without backslashes. A shell command that fails to PARSE runs none of its lines; I once recorded an edit as
  done that had never run. Read the file back before claiming a fix.
- CI flakes seen: cargo-deny action's docker `apk update`, and CodeRipper's gitleaks download (HTTP 504) fail the portfolio job by design when a rule
  cannot run. Re-run only the failed jobs; read the log first.
- End every GitHub PR body, comment and squash-merge note with a last line "— lane: webservant" (CireSnave 2026-11-09). Sonnet only for subagents; short messages; headers per MESSAGE-FORMATS.md. Never `gh auth switch`. An untracked file `supertool` in the worktree
  is not ours: never add it (`git add -A . ':!supertool'`).

## Next
1. On start: `git fetch`; `lane-restart waits`; check synapse for the PM's relay of CireSnave's results.
2. If he sent QUICKSTART failures: fix the page, add them to docs/FRESH-EYES-TRIAL.md (as a person's run, not an agent's), PR, [READY].
3. Otherwise idle. Do not invent work: the roadmap is done up to what needs a person.

UPDATE 2026-11-08: board 402 RULED (CireSnave: "402: A"): publish the first version on our sweep once the staging run is done and httpant 0.7.1 is on
crates.io (PM publishes it next; our `~0.5.22` dep will need bumping then, PM allocates the version). Four gated features stay out until a re-sweep.

WAIT: PM word to move httpant ~0.5.22 -> 0.7.1 (0.7.1 is on crates.io; I move only on the PM word); PM merge of the --renew-check-every PR (0.4.5), then I send the step-C-variant steps (plant a cert due in 3 min, watch a running server swap it) for the PM to run; PM take of the first-publish question to CireSnave | check: git fetch -q && git log --oneline origin/main -3; cargo search httpant | wake: 2026-11-10T17:00:00Z
