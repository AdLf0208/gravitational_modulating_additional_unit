# Gravitational Modulating Additional Unit — Horizon port (slice 1: the unit and its modes)

A native Horizon port of **Gravitational Modulating Additional Unit 6.4**
(`gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit`, **MIT**):
the Mekanism MekaSuit add-on unit, as far as this pin can carry it — the item,
its per-stack vertical-speed step, the toggles and the `/gmut` readout.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); approval is still required.

The mod's actual effects (defy gravity always, stop immediately, fix FOV, muted
modulating sound, the vertical-speed multiplier) only apply when a worn MekaSuit
carries a working gravitational modulating unit, and Mekanism is not ported. They
are recorded as `blocked(no-mekasuit)` in `notes.md` and reported honestly by
`/gmut info` instead of being faked. (The abilities and attribute writes they need
do exist at this SDK pin; the suit is what is missing.)

## One package: the SDK format

This is a **native `.hmod`** in the format Horizon's mod-API merge (#16941)
introduced, which retired `.hnmod`: one flat package the launcher builds against
the pinned game build. The item, its content and its behaviour are the same
file, so a single install gives both halves. It supersedes the `.hnmod` of
upstream PR #580, closed with the format change.

```
horizon-mod.toml                   the only manifest: id, name, version, api = "0.2",
                                   capabilities, [native] game/horizon/mixins
Cargo.toml, Cargo.lock             the crate; horizon-mod-api comes from .horizon/sdk/
src/lib.rs                         the one #[mod_api::horizon_mod] impl Hooks
                                   + one #[mod_api::horizon_part] impl ItemPart
src/runtime.rs                     pure logic (no mod_api types): steps, rules, gate, reading
src/support.rs                     glue: the component, the placement readout, the commands,
                                   the tooltip — all typed host calls
assets/gmut/, data/gmut/           the carried content: model, texture, names, item, tab,
                                   recipe and its advancement
LICENSE, LICENSE-NOTICE.md         the licence, inside the package
gmut.hmod                          the built package (port root, never target/)
tools/                             carry_content.py, add-notices.py, e2e.sh (retired ladder)
FINDINGS.md, CONTENT.md, notes.md  upstream, content, verification + gaps
```

## Quick checks

```sh
cd ports/gmut
ports/tools/build-slot.sh run --job sdk-gmut --port . --step test -- \
    cargo test --manifest-path Cargo.toml                 # 16 runtime tests
ports/tools/build-slot.sh run --job sdk-gmut --port . --step build -- \
    bash -lc 'cd ports/gmut && cargo horizon-mod build'    # -> target/horizon-mod/gmut.hmod
ports/tools/build-slot.sh run --job sdk-gmut --port . --step check -- \
    cargo horizon-mod check gmut.hmod                      # manifest, archive and pin
cargo horizon-mod check --uses                             # what the package touches
```

The in-game scenario (`tools/e2e.sh`) still drives the retired `.hnmod` ladder and
has to be re-pointed at a launcher-built game before it is evidence again; see
`notes.md` §“What the migration does not include”.
