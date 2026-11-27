# Coding while you wait for the next session: your PC runs it on a schedule

**English** · [Srpski](SCHEDULED-CODING.sr.md) · 27 November 2026

Goal: your Threadripper PRO 3945WX (Windows) runs Claude Code on a schedule. Each run does **one step** from the queue in [HANDOFF.md](HANDOFF.md), checks the boot tests and opens a pull request. You review in the morning. Nothing goes to `main`.

## What you need

- A Claude account with Claude Code access: Pro, Max, Team, Enterprise or Console. The free claude.ai plan does **not** include Claude Code ([setup docs](https://code.claude.com/docs/en/setup.md)).
- WSL2 with Ubuntu. Claude Code also runs natively on Windows, but WSL2 is recommended here: the Rust, QEMU and GRUB tools are Linux tools, and WSL2 supports sandboxing ([setup docs](https://code.claude.com/docs/en/setup.md)).
- A GitHub account with write access to the repo.
- The PC on (or woken by the task) and signed in at run time.

## Steps

1. **WSL2** (PowerShell as administrator, once): `wsl --install -d Ubuntu`, reboot, create a user.
2. **Inside Ubuntu:** `git clone https://github.com/RasVoja/Neo-OS-Beta-vibecoded ~/nbpos`, then `bash ~/nbpos/scripts/setup-wsl.sh` (installs QEMU, GRUB, xorriso, OVMF, pandoc, Rust with targets, `gh` and Claude Code).
3. **Logins, once, by hand:** `gh auth login`, then `claude` (browser login), then `/exit`.
4. **Manual test:** `cd ~/nbpos && ./scripts/test-boot-x86_64.sh`. Do not automate until this passes.
5. **Dry run:** `bash scripts/nightly-step.sh --dry-run`.
6. **First real step, with you watching:** `bash scripts/nightly-step.sh`. Look at the PR.
7. **Scheduling:** PowerShell: `powershell -ExecutionPolicy Bypass -File \\wsl$\Ubuntu\home\<you>\nbpos\scripts\register-task.ps1` (time and days are parameters). Try it with `Start-ScheduledTask -TaskName 'NBPOS nightly step'`.

## How it is limited

`scripts/nightly-step.sh` calls `claude -p` (non-interactive, [docs](https://code.claude.com/docs/en/headless.md)) with:

| Option | Why |
| --- | --- |
| `--permission-mode acceptEdits` | edits files without asking |
| `--allowedTools "…"` | only the listed tools: git (push only to `claude/*`), `gh pr create`, `cargo`, repo scripts. Anything else that would need approval is denied |
| `--permission-prompts none` | nobody is there to answer; needs Claude Code 2.1.259 or newer |
| `--max-budget-usd` | spending cap per run (default 15 USD, set with `NEO_NIGHT_BUDGET_USD`) |
| `timeout 4h` | wall-clock cap |
| `flock` | never two runs at once |

The rules for the step itself are in [scripts/nightly-prompt.md](../scripts/nightly-prompt.md): one branch `claude/neo-lite-stepNN-…`, boot tests before and after, no force-push, never `main`, PROGRESS and STATUS updated, a PR. If a test fails, the step is reverted and nothing is pushed.

**What the script does not guarantee by itself:** the pattern `git push origin claude/*` does not explicitly forbid `--force`. Turn on branch protection for `main` on GitHub (no direct pushes, no force-push) and merge PRs only yourself.

## Cost and safety

- Runs use your Claude limits or Console credit; watch the first few. `last-summary.txt` in `~/nbpos-night-logs/` has the cost (a client-side estimate, not a bill).
- Claude gets write access to your repo through the `gh` login. Use an account or token limited to this repo.
- Code Claude writes runs on your machine (QEMU, cargo build). Keep it inside WSL2, in Ubuntu's home folder.
- Review every PR before merging.

## If it fails

- `claude: command not found` in the task: Task Scheduler uses `bash -lc`; make sure `~/.local/bin` is on PATH (the script adds it).
- No `/dev/kvm`: QEMU is slower and tests take longer; raise `NEO_NIGHT_MAX_HOURS`.
- Login expired: run `claude` by hand again. For a fully headless setup you can use a Console API key (`ANTHROPIC_API_KEY`), billed per use.
- PC asleep: `-WakeToRun` is on, but BIOS and power plan must allow it.

## What I could not test

Neither the scripts nor the task were run on your machine (no WSL, QEMU or Rust targets here). The options were checked against the documentation, but the first run should be supervised.
