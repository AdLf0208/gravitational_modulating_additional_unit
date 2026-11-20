# Gravitational Modulating Additional Unit — Horizon port (the unit and its stored mode)

A native Horizon port of **Gravitational Modulating Additional Unit 6.4**
(`gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit`, **MIT**): the unit item and the
vertical-speed setting it keeps, as far as this pin can carry them.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); approval is still required.

## What the item does, and does not do

The item `gmut:module_gravitational_modulating_additional_unit` (paper template, stack size 1,
rarity rare) stores one of five steps — `off` 1.0x, `low` 1.5x, `medium` 2.0x, `high` 2.5x,
`ultra` 3.0x — in its own per-stack `gmut:vertical_speed` component. A right click cycles the step
forward (back while sneaking), and `/gmut mode <step>`, `/gmut next`, `/gmut prev` set and cycle the
same value; each change prints upstream's line `Vertical Speed Modifier: <step> (<mult>x)`.
`/gmut info` reports where the unit is (`held` / `off_hand` / `carried(N)` / `absent`), the stored
step, the four rule defaults and the gate. The tooltip shows the stored step and the gate line.

**The stored step is a setting only.** This port does not change the player's vertical velocity,
gravity, flight, field of view or sound in any way, and it never grants flight. Upstream applies
those effects only while a worn MekaSuit carries a working gravitational modulating unit; Mekanism
is not ported at this pin, so the gate is always `needs_gravitational_modulating_unit` and is
reported instead of being faked.

**There is no Mekanism recipe content here.** The jar's shaped recipe and its advancement are not
carried: every ingredient (`mekanism:module_base`, `mekanism:alloys/reinforced`,
`c:circuits/ultimate`, `c:pellets/polonium`) does not exist at this pin, so the recipe could never
match and the game rejected it at load. The item is reachable by `give` / the creative tab only, and
`tools/carry_content.py` regenerates both files when Mekanism lands. There is no suit installation,
no module GUI and no HUD.

## One package: the SDK format

This is a **native `.hmod`** in the format Horizon's mod-API merge (#16941) introduced: one flat
package the launcher builds against the pinned game build. The item, its content and its behaviour
are the same file, so a single install gives both halves.

```
horizon-mod.toml                   the only manifest: id, name, version, api = "0.2",
                                   capabilities, [native] game/horizon/mixins
Cargo.toml                         the crate; horizon-mod-api comes from .horizon/sdk/
src/lib.rs                         the one #[mod_api::horizon_mod] impl Hooks
                                   + one #[mod_api::horizon_part] impl ItemPart
src/runtime.rs                     pure logic (no mod_api types): steps, rules, gate, reading
src/support.rs                     glue: the component, the placement readout, the commands,
                                   the tooltip — all typed host calls
assets/gmut/, data/gmut/           the carried content: model, texture, names, item, tab
LICENSE, LICENSE-NOTICE.md         the complete licence notice, inside the package
gmut.hmod                          the built package (port root, never target/)
tools/                             carry_content.py, add-notices.py, e2e.sh
FINDINGS.md, CONTENT.md, notes.md  upstream, content, verification + gaps
```

## Quick checks

```sh
cd ports/gmut
ports/tools/build-slot.sh run --job G9-gmut --port . --step test -- \
    cargo test --manifest-path Cargo.toml                 # 16 runtime tests
ports/tools/build-slot.sh run --job G9-gmut --port . --step build -- \
    bash -lc 'cd ports/gmut && rm -f Cargo.lock && cargo horizon-mod build'
ports/tools/build-slot.sh run --job G9-gmut --port . --step check -- \
    cargo horizon-mod check gmut.hmod                      # manifest, archive and pin
cargo horizon-mod check --uses                             # what the package touches
```

The in-game leg is `tools/e2e.sh` on a game the launcher built with this package:

```sh
ports/tools/launcher-game.sh build gmut
eval "$(ports/tools/launcher-game.sh env gmut)"
ports/tools/build-slot.sh run --job G9-gmut --port ports/gmut --step e2e -- \
    bash ports/gmut/tools/e2e.sh
```

`notes.md` §“Validation at `0.2.1`” carries the version, bytes, sha256 and every ladder result.
