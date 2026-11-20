# VALIDATION — `gmut`

**Current evidence is [`notes.md`](notes.md) §“Validation at `0.2.1` (2026-11-21)”** (nightly
`0b14f8a`, `cargo test` 16/16, `cargo horizon-mod build`/`check`/`fits` green, `gmut.hmod`
**22,177 B** `c3f91a06…a0a3d76`, 15 entries, the in-game leg `PASS` + `EXIT:0` with its control and
selftest at `EXIT:1`, and the coverage table for every family the jar ships). Everything below this
line is the **retired `.hnmod` ladder** at pin `1d63e0a`/mixins `5a64af9`, kept as the record of what
PR #580 proved; it cannot be re-run (its tooling was deleted with the format) and it is not
evidence for the flat package.

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6). Every step ran
through the campaign registry (`ports/tools/build-slot.sh`, or the same runner on
the second machine) with `CARGO_INCREMENTAL=0`; no bare cargo, no
`--target-dir`. Logs: `ports/_shared/evidence/gmut/`.

## Slice 1b — the one-file bundle (2026-11-10)

| # | Step | Command (inside a lease) | Evidence | Verdict |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all -- --check`; `rustfmt --edition 2024 --check mixins/gmut/*.rs` | `fmt.log`, `fmt-helper.log` | both empty → exit 0 |
| 2 | clippy | `cargo clippy --all-targets --locked --offline -p horizon-gmut -- -D warnings` | `clippy.log` | `CLIPPY_EXIT=0` |
| 3 | runtime tests | `cargo test --offline --locked -p horizon-gmut` | `test-green.log` | **16 passed**, 0 failed |
| 4 | content build + embed | `tools/ladder-embed.sh` (builds the `.hmod`, copies it to `mixins/gmut/data/`) | `stage.log`, `package.log` | 4,011 B, `47e09b58…bdf691` |
| 5 | stage | `python3 tools/horizon.py stage` | `stage.log` | `stage_changed: true`; **6 injections, 2 modules, 1 data file**; `native-data/gmut/gmut-content.hmod` staged |
| 6 | check | `python3 tools/horizon.py check` | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| 7 | embedded test | `python3 tools/horizon.py test` | `horizon-test.log` | `16 passed; 0 failed; 1240 filtered out`, `cargo_exit: 0` |
| 8 | fast build | `python3 tools/horizon.py build` | `horizon-build.log` | `cargo_exit: 0` |
| 9 | package | `tools/package.sh` | `package.log` | `.hmod` 4,011 B / 9 entries; **`.hnmod` 13,864 B / 11 entries** (`af7a9568…`), twice, byte-identical |
| 10 | native check | `cargo horizon-mod check gmut.hnmod --kind native-mod` | `check-native.log` | `native-mod`, `native-1/1d63e0a5…/5a64af93…`, 13,864 B, same SHA |
| 11 | **e2e, one file, no `--dev-mod`** | `build-slot.sh e2e … -- tools/e2e.sh` (control leg + embedded leg) | `e2e-embedded.log` | **PASS**, `EXIT:0`, embedded content written and identical |
| 12 | fail-capability | `SELFTEST=1 CONTROL=0 tools/e2e.sh` | `e2e-embedded-selftest.log` | **FAIL** (`the mode is 'low', expected 'high'`), `EXIT:1` |
| 13 | packaged `.hmod` on the port's game | `tools/hmod-load.sh` | `hmod-load.log` | **PASS**, `EXIT:0` (held + control unknown) |
| 14 | bare `.hmod` on an unmodded game | `tools/hmod-load.sh` (before it became port-aware) | `hmod-load-unmodded.log` | refused: `gmut:vertical_speed is not a known component type` — the content needs the native half |
| 15 | guard | `python3 tools/package.py` with a drifted `data/` copy | (not logged; transcript in `notes.md`) | refused: *“is not the built …/content/gmut.hmod: copy the built content mod into data/”*, then restored |

`tools/ladder-embed.sh` is the one-lease runner for 1–10; the game legs (11–14)
ran in two later leases because the queue was deep.

## Slice 1 — the item and its modes (2026-11-09)

| # | Step | Command (inside a slot) | Evidence | Verdict |
|---|---|---|---|---|
| 1 | runtime red | `cargo test --locked --offline` | `test-red.log` | 0 passed, 16 failed (`not yet implemented`) |
| 2 | runtime green | `cargo test --locked --offline` | `test-green.log` | **16 passed**, 0 failed, `TEST_EXIT=0` |
| 3 | fmt | `cargo fmt --all -- --check`; `rustfmt --edition 2024 --check mixins/gmut/*.rs` | `fmt.log` | `FMT_EXIT=0`, `HELPERS_EXIT=0` |
| 4 | clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | `clippy.log` | `CLIPPY_EXIT=0` |
| 5 | stage | `python3 tools/horizon.py stage` | `stage.log` | 2 modules, 5 injections, `stage_changed: true` |
| 6 | check | `python3 tools/horizon.py check` | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| 7 | embedded test | `python3 tools/horizon.py test` | `horizon-test.log` | `game::gmut::tests`: **16 passed**, 1240 filtered, `cargo_exit: 0` |
| 8 | e2e (folder form) | `build-slot.sh e2e … -- tools/e2e.sh` | `e2e.log` | **PASS**, `EXIT:0` |
| 9 | fail-capability | `SELFTEST=1 tools/e2e.sh` | `e2e-selftest.log` | **FAIL** (`the mode is 'low', expected 'high'`), `EXIT:1` |
| 10 | packages | `tools/package.sh` | `package.log` | `.hmod` 4,011 B / 9 entries / `47e09b58…`; `.hnmod` 9,853 B / 9 entries / `a35122f7…` (superseded by slice 1b) |

## Gates

```
python3 ports/tools/check-e2e-logs.py gmut --strict
  ok  gmut  e2e-embedded-selftest.log  FAIL + EXIT:1
  ok  gmut  e2e-embedded.log           PASS + EXIT:0
  ok  gmut  e2e-selftest.log           FAIL + EXIT:1
  ok  gmut  e2e.log                    PASS + EXIT:0
  logs=4  ok=4  missing-exit=0  bad-exit=0

python3 ports/tools/upstream_ready.py gmut
  READY    gmut                         MIT
  ready=1 blocked=0 of 1                      (exit 0)

python3 ports/tools/check-skills.py --notes ports/gmut --strict
  ok  gmut/notes.md: Skills line present
  SKILLS GATE: PASS
```

## Not in this run

- **The inherited scaffold fixtures** (`tools/test_template.py`,
  `tools/test_stage.py`) still have no log. At this pin `test_template.py` stops
  with `E_HOOK_CATALOG` (the fixture has no `native-hooks.json`, and a named hook
  needs the catalogue) and `test_stage.py` drives the old `ports/horizon` layout.
  The real ladder above is the evidence that matters; slice 2 can settle them.
