#!/bin/sh
# Focused native regression tests; this is not an isolated manual environment.
set -eu
script_dir=${0%/*}
repo_root=$(CDPATH='' cd -- "$script_dir/../.." && pwd -P)
test_root=$(mktemp -d "${TMPDIR:-/tmp}/zacrs-shell-test.XXXXXX")
trap 'rm -rf -- "$test_root"' 0
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$test_root/home" "$test_root/config" "$test_root/cache" \
    "$test_root/state" "$test_root/data" "$test_root/run" "$test_root/tmp"
chmod 700 "$test_root/run"
cd "$repo_root"
for test_file in shell/tests/*.zsh; do
    printf '==> %s\n' "$test_file"
    env -i PATH="$PATH" HOME="$test_root/home" ZDOTDIR="$test_root/home" \
        XDG_CONFIG_HOME="$test_root/config" XDG_CACHE_HOME="$test_root/cache" \
        XDG_STATE_HOME="$test_root/state" XDG_DATA_HOME="$test_root/data" \
        XDG_RUNTIME_DIR="$test_root/run" \
        TMPDIR="$test_root/tmp" TMP="$test_root/tmp" TEMP="$test_root/tmp" \
        TERM="${TERM:-xterm-256color}" zsh -d "$test_file"
done
