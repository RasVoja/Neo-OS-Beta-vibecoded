You are continuing NBPOS (Neo Boing Platinum OS), Lite first, unattended: nobody can answer questions, so decide, note the decision in PROGRESS.md, and carry on.

Read README.sr.md, docs/STATUS.sr.md, CONTRIBUTING.sr.md and docs/HANDOFF.sr.md. Read only the last section of docs/PROGRESS.md.

Do exactly ONE step: the first unticked item of the queue in docs/HANDOFF.sr.md that you can do and test in this environment (WSL2 Ubuntu, QEMU, no real PC). Items that need the owner's hands (first boot on real PCs, installer beside Windows) are skipped; if only those remain, stop and say so.

Rules:
- Create a branch `claude/neo-lite-stepNN-short-name` from the newest claude/neo-lite-step* branch (or main if none). Never commit to main. Never force-push. Push only that branch.
- Run ./scripts/test-boot-x86_64.sh, ./scripts/test-boot-aarch64.sh and ./scripts/test-usb-x86_64.sh before and after. If a test that passed before fails after and you cannot fix it in this run, revert your change, push nothing, and write the failure in the file named by $NEO_NIGHT_LOG_DIR/FAILED.md.
- cargo fmt in the root and in user/.
- Docs in English and Serbian Latin (X.md and X.sr.md). All user-visible text comes from language catalogs.
- Own names, icons and drawings only. Never claim something works unless a test showed it. Say what you did not test.
- At the end: add an entry to docs/PROGRESS.md, tick the line in docs/STATUS.md, commit with a clear message, push the branch, and open a pull request with `gh pr create` describing the step, tests run, and what is untested.
- One step only. Then stop.
