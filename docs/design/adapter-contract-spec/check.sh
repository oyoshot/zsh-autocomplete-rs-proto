#!/bin/sh
# Check every configuration of AdapterContract.tla against the result the
# contract relies on.
#
# usage: sh check.sh
#        TLC='java -cp tla2tools.jar tlc2.TLC' sh check.sh
#
# TLC defaults to the `tlc` command.
set -u
cd "$(dirname "$0")" || exit 1
scratch="${TMPDIR:-/tmp}/adapter-contract-$$"
status=0
while read -r config expected; do
    [ -n "$config" ] || continue
    output=$(${TLC:-tlc} -workers auto -metadir "$scratch" \
        -config "$config.cfg" AdapterContract.tla 2>&1)
    if printf '%s\n' "$output" | grep -q 'No error has been found'; then
        actual=holds
    elif printf '%s\n' "$output" | grep -q 'Invariant .* is violated'; then
        actual=violated
    else
        actual=error
        printf '%s\n' "$output" | tail -20
    fi
    if [ "$actual" = "$expected" ]; then
        echo "ok: $config $actual"
    else
        echo "not ok: $config is $actual, expected $expected"
        status=1
    fi
done <<'CONFIGS'
Rule8Quiet holds
Rule8 holds
WithoutEngineCompare violated
WithoutPostGather violated
WithoutEditCheck violated
OutsideChanges violated
WrongDeclaration violated
CONFIGS
rm -rf "$scratch"
exit $status
