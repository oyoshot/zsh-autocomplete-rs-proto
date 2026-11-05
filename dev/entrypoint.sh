#!/bin/sh
set -eu

profile=${1:?expected debug or release}
shift
case "$profile" in
    debug) build_flag= ;;
    release) build_flag=--release ;;
    *) printf 'Unsupported build profile: %s\n' "$profile" >&2; exit 2 ;;
esac

# Install the actual host terminal description into this disposable HOME when
# the launcher could export it. Standard entries already present in the image
# need no host-side infocmp.
if [ -s /opt/zacrs/host-terminfo/entry ]; then
    mkdir -p "$HOME/.terminfo"
    tic -x -o "$HOME/.terminfo" /opt/zacrs/host-terminfo/entry
fi
if ! infocmp "${TERM:-dumb}" >/dev/null 2>&1; then
    printf 'Missing terminfo for TERM=%s; install infocmp on the host or choose a supported TERM.\n' "${TERM:-dumb}" >&2
    exit 1
fi

artifact_log=$(mktemp /tmp/zacrs-build.XXXXXX)
# Keep build + staging together across sessions sharing this build cache.
(
    flock 9
    cd /src
    # An empty build_flag deliberately contributes no argument.
    cargo build --locked --package zsh-autocomplete-rs --bin zsh-autocomplete-rs \
        --message-format=json-render-diagnostics ${build_flag:+"$build_flag"} > "$artifact_log"
    binary=$(jq -r 'select(.reason == "compiler-artifact" and .target.name == "zsh-autocomplete-rs" and .executable != null) | .executable' "$artifact_log")
    test -n "$binary"
    install -m 755 "$binary" "$HOME/.local/bin/zsh-autocomplete-rs"
) 9> "$CARGO_TARGET_DIR/.zacrs-manual-build.lock"

# A writable, disposable project for file and Git completion, separate from /src.
git -C "$HOME/work" init --quiet --initial-branch=main
git -C "$HOME/work" -c user.name='zacrs test' -c user.email='test@example.invalid' \
    add .
git -C "$HOME/work" -c user.name='zacrs test' -c user.email='test@example.invalid' \
    commit --quiet -m 'Initial completion fixtures'
git -C "$HOME/work" branch feature/example

printf '\nLinux/zsh manual session: %s build, fresh HOME and runtime state.\n' "$profile"
printf 'Source: /src (read-only); writable project: ~/work; exit to discard this session.\n\n'
exec zsh -i "$@"
