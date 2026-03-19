#!/usr/bin/env bash
# Manual test script for zsh-autocomplete-rs popup
# Run this directly in your terminal (not piped or in a subshell)
#
# Usage:
#   cargo build && bash test-popup.sh
#
# Keys:
#   Up/Down   - navigate
#   Tab/Enter - confirm selection
#   Space     - dismiss with space
#   Esc/Ctrl-C- cancel
#   Type      - fuzzy filter
#   Backspace - delete filter char

set -euo pipefail

BIN="${1:-$(dirname "$0")/target/debug/zsh-autocomplete-rs}"
# Try cargo target dir if direct path doesn't exist
if [[ ! -x "$BIN" ]]; then
    BIN="${CARGO_TARGET_DIR:-$HOME/.local/share/cargo/target}/debug/zsh-autocomplete-rs"
fi

if [[ ! -x "$BIN" ]]; then
    echo "Binary not found. Run 'cargo build' first."
    exit 1
fi

# Get current cursor position for realistic placement
echo ""
echo "=== zsh-autocomplete-rs popup test ==="
echo "Simulating prefix: 'gi' at current cursor position"
echo -n "gi"

# Generate test candidates
CANDIDATES="$(cat <<'EOF'
git	command
gitk	command
git-lfs	command
gist	alias
gimp	command
git-absorb	command
git-branchless	command
git-delta	command
ginkgo	file
give-permissions.sh	file
EOF
)"

output=$(printf '%s' "$CANDIDATES" | "$BIN" complete --prefix "gi" --cursor-row 5 --cursor-col 2)
exit_code=$?

echo ""
echo "---"
echo "Exit code: $exit_code"
echo "Output: '$output'"

case $exit_code in
    0) echo "Action: Confirmed selection" ;;
    1) echo "Action: Cancelled" ;;
    2) echo "Action: Dismissed with space" ;;
esac
