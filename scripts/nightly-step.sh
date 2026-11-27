#!/usr/bin/env bash
# Run ONE unattended coding step of NBPOS with Claude Code (headless, `claude -p`).
# Run inside WSL2 from the repo clone:  bash scripts/nightly-step.sh [--dry-run]
# Pokrece JEDAN korak kodiranja bez nadzora.
set -euo pipefail

REPO="${NEO_REPO:-$HOME/nbpos}"
LOG_DIR="${NEO_NIGHT_LOG_DIR:-$HOME/nbpos-night-logs}"
BUDGET_USD="${NEO_NIGHT_BUDGET_USD:-15}"     # hard cap per run (Claude Code --max-budget-usd)
MAX_HOURS="${NEO_NIGHT_MAX_HOURS:-4}"        # wall clock cap
export NEO_NIGHT_LOG_DIR="$LOG_DIR"

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
mkdir -p "$LOG_DIR"
STAMP="$(date +%Y%m%d-%H%M%S)"
LOG="$LOG_DIR/run-$STAMP.json"
LOCK="$LOG_DIR/.lock"

# Only one run at a time.
exec 9>"$LOCK"
flock -n 9 || { echo "another run is active"; exit 0; }

cd "$REPO"
git fetch --all --prune
git status --porcelain | grep -q . && { echo "working tree not clean, refusing"; exit 1; }

# Tools the run may use without asking. Everything else that would prompt is denied.
ALLOWED='Read,Edit,Write,Glob,Grep,Bash(git status *),Bash(git diff *),Bash(git log *),Bash(git add *),Bash(git commit *),Bash(git checkout *),Bash(git switch *),Bash(git branch *),Bash(git fetch *),Bash(git push origin claude/*),Bash(gh pr create *),Bash(gh pr list *),Bash(cargo *),Bash(rustup *),Bash(./scripts/*),Bash(bash scripts/*),Bash(ls *),Bash(cat *),Bash(mkdir *),Bash(make *),Bash(qemu-*),Bash(pandoc *)'

CMD=(timeout "${MAX_HOURS}h" claude -p "$(cat scripts/nightly-prompt.md)"
  --permission-mode acceptEdits
  --permission-prompts none
  --allowedTools "$ALLOWED"
  --max-budget-usd "$BUDGET_USD"
  --output-format json)

if [ "${1:-}" = "--dry-run" ]; then
  printf 'Would run in %s:\n' "$REPO"; printf ' %q' "${CMD[@]:0:3}"; echo " <prompt> ..."
  claude --version; exit 0
fi

set +e
"${CMD[@]}" >"$LOG" 2>"$LOG_DIR/run-$STAMP.err"
RC=$?
set -e
echo "exit code $RC, log $LOG"
jq -r '.result // empty' "$LOG" 2>/dev/null | tail -n 40 > "$LOG_DIR/last-summary.txt" || true
jq -r '.total_cost_usd // empty' "$LOG" 2>/dev/null | sed 's/^/cost USD: /' >> "$LOG_DIR/last-summary.txt" || true
exit $RC
