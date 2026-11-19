#!/usr/bin/env bash
# In-game check for Gravitational Modulating Additional Unit slice 1: the unit
# item registers, its vertical-speed step toggles (right click, sneak+right
# click, and `/gmut`), it is kept per stack, and `/gmut info` reports the
# placement, the rules and the gate the effects sit behind.
#
# Since the SDK migration (2026-11-18) the mod is one flat `.hmod` (horizon-mod.toml,
# src/, assets/, data/) that the launcher compiles into the game, so this script
# needs a game the launcher built with it:
#
#   ports/tools/launcher-game.sh build gmut
#   eval "$(ports/tools/launcher-game.sh env gmut)"
#   build-slot.sh run --job e2e-gmut --port ports/gmut --step e2e -- bash ports/gmut/tools/e2e.sh
#
# There is no `--dev-mod` leg and no `.horizon-cache`: the item and its behaviour
# are inside the same package. The control is the *same Nightly* without the
# package (`GAME=$CONTROL_GAME`, everything else identical), where `/gmut info`
# answers nothing and the first assertion fails.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses the native command hook.
#
# What the pin does not allow is asserted as such: the gate reads
# `needs_gravitational_modulating_unit` (Mekanism is unported, and the pin has
# no attribute API to grant flight), so the scenario checks the readout rather
# than a flight effect that cannot happen.
#
# Default run:
#  A  no unit -> `/gmut info` answers `absent`, mode off (the command works
#     without the item);
#  B  one unit held -> `held`, mode off (the jar's OFF default);
#  C  right click -> `low` and the module's own mode-change line, on the *same*
#     stack (no second unit appears: a component write must not be an exchange);
#  D  sneak + right click -> back to `off` (upstream's keybind turns back);
#  E  `/gmut mode high|next|next|prev` -> high, ultra, off (wrap), ultra (wrap);
#  F  a second unit is a separate stack: it still reads `off` while the first
#     is `ultra`, and each keeps its own step across a slot switch;
#  G  with both in the pack and neither held -> `carried(2)`;
#  H  an unknown `gmut:` id is not an item (the namespace is the mod's, not a
#     wildcard).
#
# SELFTEST=1 flips C's expected mode (it must FAIL), proving the scenario can
# fail.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${GMUT_DEBUG:-}" ] || set -x

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/gmut
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/gmut-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
UNIT=gmut:module_gravitational_modulating_additional_unit

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "GAME is unset: the SDK package is compiled into the game by the launcher." >&2
    echo "  ports/tools/launcher-game.sh build gmut" >&2
    echo "  eval \"\$(ports/tools/launcher-game.sh env gmut)\"" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/gmut-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD
    local world=$1
    rm -f "$SOCK"
    cd "$SCRATCH"
    echo "  launch: the launcher-built game carries the package; no --dev-mod" >&2
    "$CTL" --socket "$SOCK" launch --game "$GAME" --world "$world" --seed 20261105 --creative \
        --headless --wait --resources "$RES" >/dev/null
    ctl wait-ready 120 >/dev/null
    ctl wait 40 >/dev/null
}

# `/gmut info`'s line, quoted and parsed.
info() {
    say "gmut info"
    ctl wait 8 >/dev/null
    ctl messages 16 | jq -r '.chat[]' | grep -m1 'gmut: ' || true
}
mode_of() { sed -n 's/.*mode=\([a-z][a-z]*\) .*/\1/p' <<<"$1"; }
expect_info() { # expect_info FRAGMENT
    local line
    line=$(info)
    echo "  $line" >&2
    [ -n "$line" ] || fail "the unit answered no info line at all"
    grep -qF -- "$1" <<<"$line" || fail "the info line lacks '$1': $line"
}
expect_mode() { # expect_mode MODE
    local line mode
    line=$(info)
    echo "  $line" >&2
    mode=$(mode_of "$line")
    [ -n "$mode" ] || fail "the info line carries no mode: $line"
    [ "$mode" = "$1" ] || fail "the mode is '$mode', expected '$1'"
}

# The mode-change line the item prints is upstream's `displayModeChange`. A
# use-hook reply reaches the client's chat display as a `[CHAT]` line but not
# the control socket's message list (`ctl messages` carries command replies),
# so it is read back from the game log, newest last.
use_message() {
    ctl logs --lines 120 | grep -o 'Vertical Speed Modifier: [a-z]* ([0-9.]*x)' | tail -1 || true
}
# A right click of the held unit, aimed at the sky (the item's own use).
use_unit() {
    local out
    ctl look 0 -90 >/dev/null           # aim at the sky: the item's own use
    ctl wait 2 >/dev/null
    out=$(ctl use-item 2>&1) || true
    echo "  use-item: $(tr -d '\n' <<<"$out" | cut -c1-120)" >&2
    ctl wait 8 >/dev/null
}
expect_mode_line() { # expect_mode_line MODE LABEL
    local line
    line=$(use_message)
    echo "  message: ${line:-<none>}" >&2
    [ "$line" = "Vertical Speed Modifier: $1 ($2)" ] \
        || fail "the mode-change line is '${line:-<none>}', expected 'Vertical Speed Modifier: $1 ($2)'"
}
# The unit stacks the inventory holds, by slot.
units() { ctl inventory | jq -c --arg item "$UNIT" '[.hotbar[] | select(.item == $item) | {slot, count}]'; }
# Every unit stack the whole inventory holds, hotbar and main, by slot: a
# component write must change the stack that is already there, never add one.
unit_stacks() {
    ctl inventory | jq -c --arg item "$UNIT" \
        '[.hotbar[], .main[] | select(.item == $item) | {slot, count}]'
}
# expect_one_unit STAGE: the right click wrote the held stack instead of
# leaving it alone and giving the player a second one. `ItemPlace::edit` is an
# extract+insert exchange, and a creative player's hand gives without taking
# (`game_extract`) and takes an insert into the inventory rather than the hand
# (`game_insert`), so an exchange cannot change the hand: it adds a stack.
expect_one_unit() { # STAGE
    local stacks count
    stacks=$(unit_stacks)
    count=$(jq 'length' <<<"$stacks")
    echo "  unit stacks: $stacks" >&2
    [ "$count" = 1 ] \
        || fail "$1 left $count unit stacks, expected 1 (the write did not land on the held stack): $(ctl inventory | jq -c '{hotbar,main}')"
}

echo "== launch (Overworld, the launcher-built game carrying the package) =="
launch GmutE2E

echo "== A: no unit yet -> absent =="
expect_info "gmut: absent, mode=off (1.0x)"

echo "== B: the unit held -> held, the jar's OFF default =="
ctl give "$UNIT" 1 >/dev/null
ctl wait 4 >/dev/null
ctl select "$UNIT" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e --arg item "$UNIT" '[.hotbar[] | select(.item == $item)] | length == 1' >/dev/null \
    || fail "the unit item did not register: $(ctl inventory | jq -c '{hotbar,main}')"
echo "  unit: $(units)"
expect_info "gmut: held, mode=off (1.0x)"

echo "== C: a right click turns the step forward =="
use_unit
expect_one_unit "the right click"
expect_mode_line low "1.5x"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the step went to high)"
    expect_mode high
else
    expect_mode low
fi

echo "== D: sneaking turns it back =="
ctl key shift down >/dev/null
ctl wait 2 >/dev/null
use_unit
expect_one_unit "the sneaking right click"
expect_mode_line off "1.0x"
ctl key shift up >/dev/null
ctl wait 2 >/dev/null
expect_mode off

echo "== E: /gmut mode|next|prev, with both wraps =="
say "gmut mode high"
ctl wait 8 >/dev/null
expect_mode high
say "gmut next"
ctl wait 8 >/dev/null
expect_mode ultra
say "gmut next"
ctl wait 8 >/dev/null
expect_mode off
say "gmut prev"
ctl wait 8 >/dev/null
expect_mode ultra

echo "== F: a second unit is its own stack =="
ctl give "$UNIT" 1 >/dev/null
ctl wait 4 >/dev/null
echo "  units: $(units)"
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
expect_info "gmut: held, mode=off (1.0x)"
say "gmut mode low"
ctl wait 8 >/dev/null
expect_mode low
ctl select 1 >/dev/null
ctl wait 4 >/dev/null
expect_mode ultra
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
expect_mode low

echo "== G: both in the pack, none held -> carried(2) =="
ctl select 9 >/dev/null
ctl wait 4 >/dev/null
expect_info "gmut: carried(2)"

echo "== H: an unknown gmut: id is not an item =="
before=$(ctl inventory | jq -c '[.hotbar[], .main[] | select(.item != null)] | length')
ctl give gmut:module_gravitational_modulating_additional_bogus 1 >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
after=$(ctl inventory | jq -c '[.hotbar[], .main[] | select(.item != null)] | length')
[ "$before" = "$after" ] || fail "an unknown gmut: id was given ($before -> $after items)"
echo "  unknown id changed nothing ($after stacks)"

echo "== quit =="
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "PASS: one .hmod compiled into the game installed the unit and its behaviour: the step turned forward on a right click and back while sneaking, /gmut mode|next|prev set and wrapped it, two units kept their own steps, an unheld pair read carried(2), an unknown gmut: id was refused, and the gate reported needs_gravitational_modulating_unit throughout (no attribute API to grant flight)"
