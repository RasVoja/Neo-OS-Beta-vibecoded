#!/usr/bin/env bash
# One-time setup of the NBPOS build + unattended-coding environment.
# Run INSIDE WSL2 Ubuntu (not PowerShell):  bash scripts/setup-wsl.sh
# Jednokratna postavka okruzenja. Pokrenuti UNUTAR WSL2 Ubuntu-a.
set -euo pipefail

echo "== 1/5 System packages =="
sudo apt-get update
sudo apt-get install -y build-essential git curl ca-certificates gnupg jq \
  qemu-system-x86 qemu-system-arm qemu-utils \
  grub-pc-bin grub-efi-amd64-bin xorriso mtools ovmf \
  pandoc wkhtmltopdf python3 python3-pip

echo "== 2/5 GitHub CLI =="
if ! command -v gh >/dev/null; then
  sudo apt-get install -y gh || echo "gh not in apt: see https://cli.github.com"
fi

echo "== 3/5 Rust (rustup) =="
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
# The repo's rust-toolchain file (if present) picks the channel; targets below
# are the ones the kernel and user programs need.
rustup target add x86_64-unknown-none aarch64-unknown-none || true
rustup component add rustfmt clippy rust-src llvm-tools-preview || true

echo "== 4/5 Claude Code (native installer, per code.claude.com/docs/en/setup) =="
if ! command -v claude >/dev/null; then
  curl -fsSL https://claude.ai/install.sh | bash
fi
export PATH="$HOME/.local/bin:$PATH"
claude --version || echo "Open a NEW terminal if 'claude' is not found."

echo "== 5/5 Checks =="
for t in qemu-system-x86_64 qemu-system-aarch64 xorriso grub-mkrescue mtools pandoc; do
  command -v "$t" >/dev/null && echo "ok   $t" || echo "MISSING $t"
done
ls /usr/share/OVMF/OVMF_CODE*.fd >/dev/null 2>&1 && echo "ok   OVMF" || echo "MISSING OVMF"
# KVM speeds QEMU up a lot; needs WSL2 with nested virtualization.
[ -e /dev/kvm ] && echo "ok   /dev/kvm" || echo "note: no /dev/kvm, QEMU will use slower emulation"

cat <<'EOF'

Next, by hand (needs you, once):
  1. gh auth login            (GitHub: choose HTTPS, login with browser)
  2. claude                   (log in with your Claude account, then /exit)
  3. git clone https://github.com/RasVoja/Neo-OS-Beta-vibecoded ~/nbpos
  4. cd ~/nbpos && ./scripts/test-boot-x86_64.sh     (must pass before automating)
  5. bash scripts/nightly-step.sh --dry-run          (check the runner)
EOF
