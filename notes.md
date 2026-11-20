# Gravitational Modulating Additional Unit — port notes (slice 1: the unit item and its modes)

Ported from **Gravitational Modulating Additional Unit 6.4**
(`GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar`, SHA-256
`11c078cf2031ca863a5cec5d2f69590ddee84e67a16066f2b42ee7d78221982d`) for
Minecraft Java 1.21.1 / NeoForge. **MIT → licence rank 0**; local-only until
publishing is approved (ARCHITECTURE.md §8). Pin **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**
(SDK nightly, 2026-11-18), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. The
`.hnmod`-era pin was `1d63e0a5` (`game.rs` `64813cf4…`, `native_hooks.rs` `10d293c3…`), mixins
`5a64af93…` (schema 6); everything measured there is labelled historical below.

Status: **ported-partial, 0.2.1** — the item, its per-stack vertical-speed setting, the right-click
and `/gmut` toggles, the `/gmut info` readout and the tooltip are implemented, and their rules are
green in `cargo test`. **The setting is stored and cycled; it is not applied.** The port does not
change the player's vertical velocity, gravity, flight, field of view or sound, and it never grants
flight. The effects the mod exists for need a worn MekaSuit carrying a working gravitational
modulating unit, so they stay **`blocked(no-mekasuit)`**: Mekanism (the MekaSuit parent) is
unported. The abilities/attribute *writes* that used to block them exist at the SDK pin
(`player_set_abilities`, `living_set_attribute_base`) — see gap 1 — but this port deliberately does
not use them, because upstream applies them only through the suit.

## What the item does — authoritative text for the listing (0.2.1)

The item `gmut:module_gravitational_modulating_additional_unit` is a single item on the paper
template (max stack size 1, rarity rare). It stores one of five steps — `off` (1.0x), `low` (1.5x),
`medium` (2.0x), `high` (2.5x), `ultra` (3.0x) — in its own per-stack `gmut:vertical_speed`
component. A right click cycles the stored step forward, or back while sneaking; `/gmut mode <step>`
sets it, `/gmut next` and `/gmut prev` cycle it with wrap, and each change prints upstream's line
`Vertical Speed Modifier: <step> (<mult>x)`. `/gmut info` reports where the unit is (`held`,
`off_hand`, `carried(N)` or `absent`), the stored step, four rule defaults and the gate. The tooltip
shows the stored step and the gate line.

**It does not change velocity.** The stored step is a label and a number only: no vertical-speed
multiplier, no gravity change, no flight, no FOV fix and no sound change is applied, and flight is
never granted. Those effects are gated upstream on a worn MekaSuit carrying a working gravitational
modulating unit; Mekanism is not ported at this pin, so the gate is permanently
`needs_gravitational_modulating_unit` and is reported honestly.

**It carries no Mekanism recipe content.** The jar's shaped recipe and its advancement are **not**
carried in 0.2.1: every ingredient (`mekanism:module_base`, `mekanism:alloys/reinforced`,
`c:circuits/ultimate`, `c:pellets/polonium`) does not exist at this pin, the game rejected the
recipe at load with `recipes: invalid ingredient (unresolved tag or malformed input)`, and dead data
that logs errors is worse than no data. `tools/carry_content.py` regenerates both when Mekanism
lands. The item is reachable by `give` and from its creative tab only. There is no suit installation,
no module GUI, no HUD and no config page.

**Delivered as one flat `.hmod`.** `gmut 0.2.1` is `horizon-mod.toml` +
`Cargo.toml` + `src/` + `assets/gmut/` + `data/gmut/`, built by the launcher
against the pinned game build: the item, its content and its behaviour are one
package, which is what the registry's one-file-per-project rule needs. The
2026-11-17 mod-API merge (#16941) retired `.hnmod`; this port's `.hnmod` (PR
#580) was closed with the other 210, and `## Migration to the SDK format` below
records what replaced it.

**Why this mod now:** `precheck-mod.py gmut` → exit 0, `DISPATCHABLE`, jar
licence MIT (rank 0, permissive), no worldgen, and its one dependency
(`mekanism`) is a required parent recorded in the queue as unported — so it is
an add-on whose *item* can still land faithfully while the effects wait.
Brief: `ports/_shared/queue/brief-gmut.md`.

## Class + tier and the approach

- **Class:** `integration` (a Mekanism MekaSuit add-on unit); **tier:** `logic`
  (`ports/ledger.csv`; provenance `ports/_shared/source-map/atm10-sources.csv`,
  which still says `unknown (read mods.toml at import)` — the jar says **MIT**).
- **Approach** (`horizon/docs/modpacks.md`, "Classes"/"Tiers"): an
  `integration`+`logic` job — carry the item's data and write its behaviour as
  callbacks (use, commands, tooltip), not a machine and not a deep entity/attribute
  job. The class's own rule ("after both sides are ported") is why the *effects*
  stay blocked instead of being emulated; the item half is what can be faithful.
- **Carry-over vs agent port:** the model, texture and names are carried from the jar
  (`tools/carry_content.py`, decisions in [`CONTENT.md`](CONTENT.md)); the mode
  component, its toggles, the readout and the tooltip are the agent port. The jar's recipe and its
  advancement are **not** carried in 0.2.1: their Mekanism ingredients do not exist at this pin, the
  game rejected the recipe at load, and they are regenerated when Mekanism lands (CONTENT.md §“Left
  out”).

## What the slice does

- **The item.** `gmut:module_gravitational_modulating_additional_unit`, the
  `minecraft:paper` template with `max_stack_size: 1` and `rarity: rare`
  (upstream's `maxStackSize(1).rarity(Rarity.RARE)`), with the initial
  `gmut:vertical_speed: "off"` component.
- **The step.** Upstream's `VerticalSpeed` (`OFF 1.0`, `LOW 1.5`, `MEDIUM 2.0`,
  `HIGH 2.5`, `ULTRA 3.0`), kept **per stack** in the item's own
  `gmut:vertical_speed` component and cycled with `MathUtils.getByIndexMod`'s
  wrap. **Stored only:** no code reads it to change the player's movement at
  this pin (see "What the item does" above).
- **The toggles.** A right click (`native_use_item`, API 13) turns the step
  forward, or back while sneaking — upstream's keybind direction. `/gmut mode
  <off|low|medium|high|ultra>`, `/gmut next`, `/gmut prev` do the same from chat.
  Each change prints upstream's `displayModeChange` line
  (`Vertical Speed Modifier: high (2.5x)`).
- **The readout.** `/gmut info` prints
  `gmut: <placement>, mode=<step> (<mult>x), rules fly_always=…, stop_immediately=…,
  fix_fov=…, mute_modulating_sound=…, modulation=<gate>`; the placement is
  `held` / `off_hand` / `carried(N)` / `absent`, the rules are upstream's config
  defaults, and the gate is `needs_gravitational_modulating_unit` at this pin.
- **The tooltip.** The step, and (while the gate is closed) the jar's own line
  "Requires a Gravitational Modulating Unit".

## Hooks (SDK format: typed `impl Hooks` methods and one item Part)

Everything below is the API at nightly `0b14f8a`; the retired `.hnmod` helper names are in
`## Migration to the SDK format`. No `[[native.mixin]]` is declared: nothing here needs game
internals.

| Hook / Part method (contract) | Where | What it does |
|---|---|---|
| `register` | `src/support.rs::register` | requests `gmut:module_gravitational_modulating_additional_unit` and attaches `Unit` to it |
| `component_types` (collect `types`) | `src/lib.rs` → `support::component_types` | registers `gmut:vertical_speed` before content loads |
| `register_commands` (collect `commands`) | `src/lib.rs` → `support::register_commands` | `/gmut info`, `/gmut mode <speed:word>`, `/gmut next`, `/gmut prev` |
| `run_mod_command` (cancel `ran`) | `src/lib.rs` → `support::run_command` | the parsed invocation's bodies; answers through `PlayerRef::tell` |
| `ItemPart::use_item` (replace `result`) | `src/lib.rs` → `support::on_use_item` | cycles the hand's step, forward or (sneaking) back; `Success` on the client, `SuccessServer` on the server, so the hand swings |
| `ItemPart::tooltip` (collect `lines`) | `src/lib.rs` → `support::on_tooltip` | the step and the gate line on the item |

One Part attached to one item (`r.item_part(item, Unit)`), so every moment above is this mod's
item's alone and no id check is needed inside it.

Historical (the retired `.hnmod`, kept for reading the artifact): `component_types.rs`,
`use_item.rs`, `tooltip.rs`, `register.rs`, `perform.rs` were one helper per stamped hook, plus
`embedded_content.rs`, a raw-target injection on
`horizon_client::game::mods::content_root` that unpacked the content `.hmod` from the bundle.
That injection has no SDK counterpart and no longer has a job: content is in the package.


## Pin facts that shaped this slice

The first three are the `.hnmod`-era pin's (`1d63e0a5`); the fourth and fifth hold at `0b14f8a`
too, where the same helpers exist under the SDK's names.

1. **No attribute API at `1d63e0a`; partly lifted at `0b14f8a`.** At the `.hnmod` pin there
   was no write to `AttributeInstance`/`Abilities`, so `fly_always` (which sets
   `abilities.flying` upstream) and the gravity multiplier could not be applied;
   `player_toggle_flight` (API 125) was only a *cancellable notification* of a toggle the player
   made. **The SDK surface does have both writes now** — `player_set_abilities(player,
   PlayerAbilities { may_fly, flying, fly_speed, … })` and `living_set_attribute_base(living,
   "minecraft:gravity", base)` — so the *mechanism* exists at `0b14f8a` even though this
   migration does not use it: upstream only grants flight when a worn suit carries a working
   gravitational modulating unit, and the suit is the next gap. The honest state is
   `blocked(no-mekasuit)` for the effect and “API present, not wired” for the write; wiring it
   without a game leg would be an unproved behaviour claim, so it is the next slice's job.
2. **No client→server key path.** `key`/`input_key` are client-side listeners
   (ARCHITECTURE §3), so upstream's keybind packet has no honest route; the
   right click and `/gmut` are the reachable controls. Recorded as a gap.
3. **The item-count helper skips this item.** It counted only *plain* stacks
   (`damage == 0 && extra.is_none()` at `1d63e0a`), and this item always carries
   its component, so it always answered 0. The placement read counts the 36
   inventory slots and the off hand directly — in the SDK through
   `PlayerRef::inventory_slots`, with the same numbers.
4. **A use-hook reply does not reach `ctl messages`.** A message emitted from a
   right click appeared as a `[CHAT]` line in `latest.log` and on screen, but
   `ctl messages` (which carries command replies) did not list it; the e2e read
   that line back from `ctl logs`. The SDK's `PlayerRef::tell` is the same kind of
   notice, so the re-pointed e2e keeps reading `ctl logs` for it.
5. **The content item's name key is `item.gmut.<path>`** (Horizon's item naming),
   not the jar's NeoForge `module.gmut.*` key.

## Content (`assets/` and `data/` in the package)

Generated by [`tools/carry_content.py`](tools/carry_content.py) from the kept
jar; [`CONTENT.md`](CONTENT.md) lists every carried and every written file and
why. 6 files: the item model and texture, the 26.3 item definition, the item
declaration, a creative tab and the lang file — now `assets/gmut/` and
`data/gmut/` at the port root, in the same `.hmod` as the code. The jar's recipe
and advancement are not carried (0.2.1): their Mekanism ingredients do not exist
at this pin and the game rejected the recipe at load, so shipping them only added
invalid-ingredient errors; the carry script regenerates them when Mekanism lands.

## One file: the embedded content (slice 1b — **historical, retired 2026-11-18**)

Superseded by the SDK layout, where the content is simply `assets/` and `data/` in the one
`.hmod`. Kept because it is how the closed PR #580 shipped, and because it is the shape any
`.hnmod`-era port's notes describe.

The registry took **one `file` per project** and published native mods as `.hnmod`, so a split
content+code port could not be listed faithfully (a player would get the code without the item,
or the item without the behaviour). The port used the pattern AE2 ships and `charginggadgets`
proved here (`ARCHITECTURE.md` §8, `docs/native-mods.md` “Data files”):

1. the built `content/gmut.hmod` was copied to `mixins/gmut/data/gmut-content.hmod`
   (byte-identical; `tools/package.py` **refused** a `data/` copy that differed);
2. `mixins/gmut/embedded_content.rs` compiled it in with
   `horizon_client::native_data!("gmut", "gmut-content.hmod")`, wrote it to
   `.horizon-cache/gmut-embedded-content.hmod` when the bytes differed, and returned
   `ControlFlow::Break(content_root(root, enabled, &with))`;
3. one `[[injection]]` on `target = "horizon_client::game::mods::content_root"` (HEAD,
   cancellable, bindings `root` / `enabled` / `dev`) ran it before content loads;
4. `tools/package.py` packed `mixins/<id>/data/`, so the bundle carried 11 entries instead of 9.

**The finding that outlived it:** the content alone is not loadable on an unmodded game — the
item declares `gmut:vertical_speed` and only the native `component_types` hook registers that id,
so a bare content `.hmod` was refused with *naming a component type that is not*
(`hmod-load-unmodded.log`). That is still true with one `.hmod`, and it is why the content and the
`component_types` hook must be in the same package.

## Migration to the SDK format (2026-11-18)

**What forced it.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992 cleaning up the
old files) retired `.hnmod`: a native mod is now **one flat `.hmod`** — `horizon-mod.toml` +
`Cargo.toml` + `src/lib.rs` (one `impl mod_api::Hooks`) + `assets/` + `data/` — built by the
launcher against the pinned game build. The registry closed 210 `.hnmod` submissions on
2026-11-18 with that single reason, this port's PR #580 among them. SDK: nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78, toolchain rust 1.99.1,
`content_api` 0.2, `cargo-horizon-mod` at `~/.cargo/bin`. Layout reference:
`cargo horizon-mod new /tmp/opencode/gmut-new --template native` (which fetches `.horizon/sdk/`,
749 surface items); method names were only ever taken from `cargo horizon-mod hooks <text>` and
the SDK's own sources, never guessed.

### Files created

| File | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and identity: `id = "gmut"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native] game = "26.3"`, `horizon = 0b14f8a…`, `mixins = e59da90…` |
| `Cargo.toml` | `name = "gmut"`, `version = "0.2.0"`, `edition = "2024"`, `publish = false`, one dependency: `horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (`register`, `component_types`, `register_commands`, `run_mod_command`) and `#[mod_api::horizon_part] impl ItemPart for Unit` (`use_item`, `tooltip`) |
| `src/runtime.rs` | **the old `mixins/gmut/runtime.rs`, byte-identical** (408 lines, sha256 `1c79d041…df58d`, `cmp` clean before and after the migration) |
| `src/support.rs` | the game-side glue: the unit's `ItemKey`, the `gmut:vertical_speed` read/write, the placement readout, the commands and the tooltip |
| `assets/gmut/…`, `data/gmut/…` | the carried content, moved from `content/assets/gmut` and `content/data/gmut` unchanged |
| `gmut.hmod` | the built package (port root, never `target/`) |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` |

### Files deleted (retired format)

`mixins/gmut/` (the whole tree: `horizon-mixin.toml`, `runtime.rs`, `support.rs` and the six
one-function helpers `component_types.rs`, `use_item.rs`, `tooltip.rs`, `register.rs`,
`perform.rs`, `embedded_content.rs`, plus `data/gmut-content.hmod`), `gmut.hnmod` (13,864 B,
`af7a9568…`, the closed PR's artifact), `content/` (its `assets/`/`data/` moved, its
`horizon-mod.toml` replaced by the root one, its built `gmut.hmod` no longer a separate
deliverable), `compatibility.json` (the pin is `[native]` now), `rust-toolchain.toml` (the SDK
pins rust 1.99.1), the old standalone `Cargo.lock`, and the retired ladder:
`tools/{horizon.py,package.py,package.sh,ladder-embed.sh,test_stage.py,test_template.py,hmod-load.sh}`
and `tools/audit/` (a `.rs`/`.toml` crate the SDK package walker refuses).
`tools/{carry_content.py,add-notices.py,e2e.sh}`, `FINDINGS.md`, `CONTENT.md`, `notes.md` and the
licences stay. The old evidence logs stay in `ports/_shared/evidence/gmut/` and still describe
what they measured; they are not evidence for this package.

**Trap (cost a build):** `cargo horizon-mod build` walks the package folder and **refuses any
`.rs` or `.toml` outside `src/` and the root allow-list** (`unexpected package path:
mixins/gmut/runtime.rs`, then `tools/audit/Cargo.toml`). `target/`, `.horizon/` and `.git` are
skipped; `assets/`, `data/`, `icon.png`, and root `README`/`LICENSE`/`NOTICE`/`COPYING` are
packed. So the retired tree had to be deleted, not kept beside the new sources.

### Each old hook's mapping

| Old (`.hnmod`) | New (SDK) | Notes |
|---|---|---|
| `[[module]] gmut` = `runtime.rs` | `src/runtime.rs`, `pub mod runtime` in `src/lib.rs` | unchanged file, unchanged 16 tests |
| `[[module]] gmut_state` = `support.rs` | `src/support.rs` | `crate::game::Game` calls replaced by host handles; no `#[allow(private_interfaces)]` is needed any more |
| `component_types` helper | `Hooks::component_types(&self, cx, event)` → `event.types.push(…)` | same collect contract, same id |
| `use_item` helper (`claimed: &mut bool`, `Game`) | `ItemPart::use_item(host: HostSide, event)` → `Option<InteractionResult>` | item-scoped Part, so no id test; `Success` (client) / `SuccessServer` (server) reproduce "claims the use, the hand swings"—the old hook set `claimed = true` |
| `tooltip` helper (`Game`, `Text` lines) | `ItemPart::tooltip(&self, host: &dyn ClientHost, event)` | `Text::tr(key, &[…])` → `Text::translate(key, vec![…])` |
| `register_commands` helper (`Vec<NativeCommand>`) | `Hooks::register_commands(host, event)` → `event.commands` | `NativeCommand::new/op` → `mod_api::Command::new/op`, same usages and levels |
| `perform_command` helper (raw `&str`) | `Hooks::run_mod_command(host, event) -> Flow` | **not** a hook rename: the body now reads `event.command.literals` and `event.command.argument("speed")`, so there is no whitespace parsing at all; feedback goes through `PlayerRef::tell` |
| `embedded_content` raw injection | **no counterpart, not needed** | content lives in `assets/`/`data/` of the same package; nothing is unpacked at run time |
| `native_data!` content embedding | — | gone with it |

Host calls that replace the `Game` accessors: `game.held()`/`held_slot()` →
`ItemPlace::PlayerHand { player: uuid, hand }.stack(host)`; `game.inventory`/`offhand` →
`PlayerRef::inventory_slots(host)` (0–35 and 40, as `Inventory` numbers them);
`game.native_sneaking()` → `EntityRef::is_shift_key_down(host)`; `game.stack_in_mut` +
`hand_stack_replaced` → `ItemPlace::edit(host, tx, …)` inside `host.transaction(…)`;
`game.native_feedback` → `PlayerRef::tell(host, Text)`.
The unit's item is compared by `ItemKey` (requested once in `register` and kept in a
`OnceLock`), because the SDK does not expose `content::registry_keys::item_id` to a mod.

### Manifest mapping

`content/horizon-mod.toml` (content-only: `id`, `name`, `version = "6.4"`, `authors`,
`api`, `side`, `capabilities`) + `horizon-native.json` (`game`, `horizon`, `mixins`, `id`,
`version`, `schema`) → **one** root `horizon-mod.toml`. Its `version` is the **port's**
(`0.2.0`, a new version for the resubmission), not the original mod's `6.4`; `name` is the mod's
name; `authors` keeps `gisellevonbingen` and adds the port's; `license = "MIT"`, `homepage`,
`issues` are new fields the old content manifest did not carry. `[native]` holds the three exact
40-hex commits; there is no `schema`, no `[api] path`, and no `priority` any more.

### Build and check commands (all inside a slot)

```sh
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step test  -- cargo test --manifest-path ports/gmut/Cargo.toml
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step fmt   -- cargo fmt --manifest-path ports/gmut/Cargo.toml --all -- --check
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step lint  -- cargo clippy --manifest-path ports/gmut/Cargo.toml --all-targets -- -D warnings
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step build -- bash -lc 'cd ports/gmut && cargo horizon-mod build'
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step check -- cargo horizon-mod check ports/gmut/gmut.hmod
ports/tools/build-slot.sh run --job sdk-gmut --port ports/gmut --step check -- cargo horizon-mod fits ports/gmut/gmut.hmod
```

**Traps met while running them:**

1. **`build-slot.sh` does not `cd` into the port** (`--port` only picks the cache links): a bare
   `cargo test` there fails with *could not find `Cargo.toml` in the campaign root*. Pass
   `--manifest-path`, or `bash -lc 'cd ports/gmut && …'`.
2. **The remote runner is unusable today:** forwarding answered `cd: /home/deepseek/atm10/gmut:
   No such file or directory`, so every step here ran with `BUILD_LOCAL=1`. Worth a look before the
   next wave — it is 10 slots the campaign is not getting.
3. `cargo horizon-mod build` writes to `target/horizon-mod/gmut.hmod`; the deliverable is **copied
   to the port root**, because `target/` is disposable.
4. `cargo horizon-mod check <file>` validates the manifest, the archive and the pin — it does
   **not** compile the mod (`native source: unsandboxed; not executed by the registry check`).
   `cargo test` against `.horizon/sdk` is what compiles it, and `check --uses` lists what the
   package is declared to touch (162 API items, 6 hooks, 2 Part kinds, `capabilities =
   ["registries"]`).
5. `cargo fmt` must not be allowed to rewrite `src/runtime.rs`: it did not (the carried file was
   already rustfmt-clean), and the sha256 was re-checked afterwards.

### What the migration does **not** include

The in-game leg. `tools/e2e.sh` still drives the retired ladder (a `horizon-client` built by
`tools/horizon.py` with the module injected); the SDK path builds the game through the launcher
(`horizon-launcher native-build … --instance …`), which needs an instance and a game built at
`0b14f8a`, and this box has neither. The rules and the glue compile and the package checks; the
scenario's in-game re-run is the next slice's first job, and the script says so in its header.

The port's CI was re-pointed too (`.github/workflows/ci.yml`): it installed rust 1.95.0, cloned
`horizon-mixins` and read `compatibility.json` for a revision — all three retired. It now
installs rust 1.99.1, installs `cargo-horizon-mod` from the pinned `0b14f8a`, builds (which
fetches `.horizon/sdk/`), and runs fmt, clippy, test, `check` and `check --uses`. **The
`cargo install --git … --rev 0b14f8a` step is the one line not run here** (no CI runner, and
building the tool locally to test it was outside this brief's box time); it is the documented
tool at the pinned revision, and a reviewer's first CI run is what proves it.


## Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` were run before
hand-back (logs in the migration table below). The port has no `#[allow]`: the SDK's typed hosts
removed the `private_interfaces` exception the injected module needed. The migration used
`using-superpowers` (then `brainstorming`, `writing-plans`, `test-driven-development`,
`systematic-debugging`, `verification-before-completion`) and `rust-best-practices` — borrowed
event types, no `unwrap` outside tests, `Option<InteractionResult>` returned as the Part contract
expects, one transaction around the only write, and the pure rules left in `runtime.rs`.


## Verification

### The SDK package (2026-11-18) — historical evidence for 0.2.0

SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78. Every step ran through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0` and **`BUILD_LOCAL=1`** (the remote runner
was unreachable — see trap 2 above); no bare cargo outside a lease, no `--target-dir`. Logs:
`ports/_shared/evidence/gmut/sdk-*.log`.

| Step | Command (inside a lease) | Evidence | Result |
|---|---|---|---|
| runtime carry | `cmp mixins/gmut/runtime.rs src/runtime.rs`, `sha256sum` both | `runtime-carry.log` | **identical**, 408 lines, `1c79d041…df58d`, before and after the migration |
| runtime tests | `cargo test --manifest-path ports/gmut/Cargo.toml` | `sdk-test.log` | **16 passed**, 0 failed, exit 0 |
| fmt | `cargo fmt --all -- --check` | `sdk-fmt.log` | exit 0 |
| clippy | `cargo clippy --all-targets -- -D warnings` | `sdk-clippy.log` | exit 0, **0 warnings** |
| package build | `cd ports/gmut && cargo horizon-mod build` | `sdk-build.log`, `sdk-build-2.log` | `Built ./target/horizon-mod/gmut.hmod`; two consecutive builds **byte-identical** (`95489434…37b5`) |
| package check | `cargo horizon-mod check ports/gmut/gmut.hmod` | `sdk-check.log` | exit 0 |
| uses | `cd ports/gmut && cargo horizon-mod check --uses` | `check-uses.log` | 162 API items: **6 hooks**, 2 Part kinds, 1 capability (`registries`), 2 macros |
| fits | `cargo horizon-mod fits ports/gmut/gmut.hmod` | `sdk-fits.log` | exit 0: *made for* `0b14f8a` |
| retired files | `find`/`rm` of the `.hnmod` tree | `removed-files.log` | listed, and the reason each went |

The delivered package at that date was **`gmut.hmod`, 24,443 B, SHA-256
`95489434caf1e85ea664a8e118067e78055ee0526b3f589dba7d6d17556d37b5`, 18 entries** — the 18th
being `Cargo.lock`, which `cargo test` had written into the port directory and the packer took from
the directory's file list. It is build output, not part of the mod: the 2026-11-20 pass rebuilt with
`rm -f Cargo.lock` before each build (17 entries, 22,112 B, see *Packaging gate* below).
The 17 entries are
(`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`,
`assets/gmut/` ×4, `data/gmut/` ×4, `src/{lib,runtime,support}.rs`, `uses.json`).
`check` quotes it back: `gmut 0.2.0 (native-mod, format
native-1/0b14f8ac…/e59da909…, for 26.3) … 24443 bytes … unsigned`.

**Not verified here:** the in-game leg (below), and therefore every behaviour claim is a
compile-and-rule claim, not a `ctl` transcript for *this* package. The `.hnmod` transcripts in
the next subsection are what the previous artifact did.

### Historical: the `.hnmod` ladder (pin `1d63e0a`, retired 2026-11-18)

Kept because it is what PR #580 shipped and what the behaviour was proved on; none of it is
evidence for the flat package.

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6); checkouts
`ports/_shared/pins/horizon-1d63e0a` and `.../horizon-mixins-5a64af9`. Logs:
`ports/_shared/evidence/gmut/`.

| Step | Evidence | Result |
|---|---|---|
| runtime TDD (red) | `test-red.log` | **0 passed, 16 failed** — every body `todo!()` |
| runtime tests (green) | `test-green.log` | **16 passed**, 0 failed, `TEST_EXIT=0` |
| fmt | `fmt.log`, `fmt-helper.log` | `cargo fmt --all -- --check` ok; `rustfmt --edition 2024 --check mixins/gmut/*.rs` ok (7 helpers) |
| clippy | `clippy.log` | `--all-targets -- -D warnings`, `CLIPPY_EXIT=0` |
| stage | `stage.log` | `stage_changed: true`; plan: 2 modules, **6 injections, 1 data file**; `native-data/gmut/gmut-content.hmod` (4,011 B) staged |
| check | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| embedded test | `horizon-test.log` | `test result: ok. **16 passed**; 0 failed; 1240 filtered out`, `cargo_exit: 0` |
| build | `horizon-build.log` | `cargo_exit: 0`, fast `horizon-client` |
| e2e (slice 1) | `e2e.log` | **PASS**, `EXIT:0` (folder form, with and without `--dev-mod`) |
| fail-capability (slice 1) | `e2e-selftest.log` | `SELFTEST=1` → **EXIT:1** |
| **e2e, one `.hnmod`, no `--dev-mod`** | `e2e-embedded.log` | **PASS**, `EXIT:0` — content from `mixins/gmut/data/`, behaviour from the bundle, plus the unusable-cache control |
| fail-capability (embedded) | `e2e-embedded-selftest.log` | `SELFTEST=1` → **EXIT:1** (`the mode is 'low', expected 'high'`) |
| packaged `.hmod` on the port's game | `hmod-load.log` | **PASS**, `EXIT:0` (leg 1 registers + reads held; leg 2 unknown) |
| bare `.hmod` on an unmodded game | `hmod-load-unmodded.log` | **refused**: `gmut:vertical_speed is not a known component type` — the content needs the native half (why the port is one file) |
| `.hmod` build/check | `package.log` | 4,011 B, `47e09b58…bdf691`, 9 entries, “no code: content only” |
| `.hnmod` (one file) | `package.log` | **13,864 B**, `af7a9568…eba46d`, **11 entries** |
| `cargo horizon-mod check --kind native-mod` | `check-native.log` | `native-mod`, `native-1/1d63e0a5…/5a64af93…`, 13,864 B, same SHA |

The 4,011 B `.hmod` and the 13,864 B `.hnmod` were produced twice by two leases
and came out byte-identical (`af7a9568…`), so `tools/package.py`'s fixed dates
and modes still give a reproducible bundle. `tools/package.py` also **refuses** a
`data/` copy that drifted from `content/gmut.hmod` — exercised once by appending
a byte: *“…/mixins/gmut/data/gmut-content.hmod is not the built
…/content/gmut.hmod: copy the built content mod into data/”* — then restored
(sha `47e09b58…` again).

### In-game transcript (one `.hnmod`, no `--dev-mod`; quoted from `e2e-embedded.log`)

```
== launch (Overworld, one native bundle; no --dev-mod) ==
== control: the same game with no embedded content file ==
  launch: no --dev-mod; the bundle carries its own content
  gmut: absent, mode=off (1.0x), rules fly_always=off, … modulation=needs_gravitational_modulating_unit
  without the embedded file: no unit item, the reading is absent
  launch: no --dev-mod; the bundle carries its own content
  embedded content: 4011 B at /tmp/opencode/gmut-home.3zkiwD/.horizon-cache/gmut-embedded-content.hmod, identical to content/gmut.hmod
== A: no unit yet -> absent ==
  gmut: absent, mode=off (1.0x), …
== B: the unit held -> held, the jar's OFF default ==
  unit: [{"slot":0,"count":1}]
  gmut: held, mode=off (1.0x), …
== C: a right click turns the step forward ==
  use-item: {"block":null,"block_before":null,"changed":false,"inventory_change":{},"ok":true,"screen":"playing"}
  message: Vertical Speed Modifier: low (1.5x)
  gmut: held, mode=low (1.5x), …
== D: sneaking turns it back ==
  message: Vertical Speed Modifier: off (1.0x)
  gmut: held, mode=off (1.0x), …
== E: /gmut mode|next|prev, with both wraps ==
  … mode=high (2.5x) … mode=ultra (3.0x) … mode=off (1.0x) … mode=ultra (3.0x)
== F: a second unit is its own stack ==
  units: [{"slot":0,"count":1},{"slot":1,"count":1}]
  … mode=off … mode=low … mode=ultra (first unit) … mode=low (second unit)
== G: both in the pack, none held -> carried(2) ==
  gmut: carried(2), mode=off (1.0x), …
== H: an unknown gmut: id is not an item ==
  unknown id changed nothing (2 stacks)
== quit ==
PASS: one .hnmod with no --dev-mod installed the unit and its behaviour: … and with the embedded content unwritable the id was unknown
EXIT:0
```

### In-game transcript (slice 1, folder form; quoted from `e2e.log`)

```
== A: no unit yet -> absent ==
  gmut: absent, mode=off (1.0x), rules fly_always=off, stop_immediately=on, fix_fov=off, mute_modulating_sound=off, modulation=needs_gravitational_modulating_unit
== B: the unit held -> held, the jar's OFF default ==
  unit: [{"slot":0,"count":1}]
  gmut: held, mode=off (1.0x), … modulation=needs_gravitational_modulating_unit
== C: a right click turns the step forward ==
  use-item: {"block":null,"block_before":null,"changed":false,"inventory_change":{},"ok":true,"screen":"playing"}
  message: Vertical Speed Modifier: low (1.5x)
  gmut: held, mode=low (1.5x), …
== D: sneaking turns it back ==
  message: Vertical Speed Modifier: off (1.0x)
  gmut: held, mode=off (1.0x), …
== E: /gmut mode|next|prev, with both wraps ==
  … mode=high (2.5x) … mode=ultra (3.0x) … mode=off (1.0x) … mode=ultra (3.0x)
== F: a second unit is its own stack ==
  units: [{"slot":0,"count":1},{"slot":1,"count":1}]
  … mode=off … mode=low … mode=ultra (first unit) … mode=low (second unit)
== G: both in the pack, none held -> carried(2) ==
  gmut: carried(2), mode=off (1.0x), …
== H: an unknown gmut: id is not an item ==
  unknown id changed nothing (2 stacks)
== quit, then the no-content control ==
  gmut: absent, mode=off (1.0x), …
  without --dev-mod the id is unknown and the reading is absent
PASS: the unit registered, its step turned forward on a right click and back while sneaking, /gmut mode|next|prev set and wrapped it, two units kept their own steps, an unheld pair read carried(2), an unknown gmut: id was refused, and the gate reported needs_gravitational_modulating_unit throughout (no attribute API to grant flight)
EXIT:0
```

The run is fail-capable three ways: `SELFTEST=1` flips C's assertion and exits 1;
**F** is a control that a global mode would fail (the second unit must read
`off` while the first is `ultra`); and the leading control launch (`.horizon-cache`
unusable, so the bundle cannot write its content) shows the same id unknown and
`absent`. **G** counts two carried units, which a
grepping-on-the-held-slot-only implementation would fail.

### Packaged artifacts

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/gmut.hmod` (the input of the bundle) | 4,011 B | `47e09b5849acf3cd408dfe1d64b6038b453f7dba95f29d2978010adb59bdf691` | 9 |
| `mixins/gmut/data/gmut-content.hmod` (inside the bundle) | 4,011 B | `47e09b58…bdf691` | 9 |
| **`gmut.hnmod` (the one deliverable)** | **13,864 B** | `af7a9568c920e37bbfa160fe3db44c3c4d9a6bed52ba11f17efa4eefe1eba46d` | **11** |

The 11 entries: `horizon-native.json`, `mixins/gmut/{embedded_content,runtime,support,component_types,use_item,tooltip,register,perform}.rs`,
`mixins/gmut/horizon-mixin.toml`, `mixins/gmut/data/gmut-content.hmod`.

### Campaign gates at this touch

- `python3 ports/tools/precheck-mod.py gmut` → exit 0, `DISPATCHABLE`, jar
  licence MIT (rank 0). It also asks the ledger row to be corrected from
  `MIT`-by-claim to the jar's value — it is already `MIT`; the tool's note is
  the generic "the JAR value wins" line.
- `python3 ports/tools/check-e2e-logs.py gmut --strict` → **4 ok, 0 missing-exit,
  0 bad-exit** (`e2e-embedded.log` PASS+0, `e2e-embedded-selftest.log` FAIL+1,
  `e2e.log` PASS+0, `e2e-selftest.log` FAIL+1).
- `python3 ports/tools/check-skills.py --notes ports/gmut --strict` → **PASS**.
- `python3 ports/tools/upstream_ready.py gmut` → **READY gmut MIT**, exit **0**
  (one file, no split-port problem).

## Deviations and gaps (slice 1)

1. **`blocked(no-attribute-api)` at `1d63e0a`; `blocked(no-mekasuit)` now.** Upstream's
   `fly_always` sets `abilities.flying`, and `vertical_speed` adds
   `j * (speed - 1) * flyingSpeed * 3` to the player's delta movement while flying. The
   `.hnmod` pin had neither an abilities write nor a per-tick velocity helper, so the mode was
   stored and read out, never applied. **Re-checked at `0b14f8a`**: the machinery is there —
   `PlayersServerHost::player_set_abilities` (may_fly/flying/fly_speed, sent to the client and
   saved with the player) and `LivingServerHost::living_set_attribute_base` (any attribute,
   `minecraft:gravity` included) — so what blocks the effect is no longer the API but the
   **suit**: upstream applies it only when a worn MekaSuit carries a working gravitational
   modulating unit, and Mekanism is unported. Applying it unconditionally would grant flight to
   anyone holding the item, which the mod does not do. Next slice: wire it behind a gate that is
   honest (off, with the API in place) or behind `mekanism` when it lands, and prove it in game.
2. **`blocked(no-mekasuit)` — the gate.** Upstream's
   `CommonPlayerTickHandler.isGravitationalModulationReady` needs a worn
   MekaSuit with a gravitational modulating unit; Mekanism is unported, so the
   gate is always `needs_gravitational_modulating_unit`. The reading is honest
   about it instead of pretending the module applies.
3. **`stop_immediately`, `fix_fov`, `mute_modulating_sound`** are client-side
   effects (delta-movement zeroing, `ComputeFovModifierEvent`, a sound mixin).
   They are reported as their defaults; none is applied. FOV would need the
   client FOV hook (`native_compute_fov_modifier`, API 362), which is a
   different shape than this logic slice.
4. **No keybind.** Upstream cycles the mode from a key (its own unbound key,
   sneak for backwards). `key`/`input_key` are client-side listeners with no
   stamped client→server action, so the right click carries the same direction
   rule and `/gmut` is the harness path.
5. **The mode is the item's component, not a suit module config.** There is no
   suit and no module container; the component keeps upstream's per-stack
   semantics (proved in the e2e: two units hold different steps).
6. **The recipe is not carried (0.2.1).** `mekanism:module_base`,
   `mekanism:alloys/reinforced`, `c:circuits/ultimate` and
   `c:pellets/polonium` do not exist at this pin; the recipe could never match
   and the game rejected it at load (`recipes: invalid ingredient`), so the
   recipe and its advancement were removed instead of shipping dead data. The
   item is reachable by `give`/creative only; the carry script regenerates the
   recipe when Mekanism lands.
7. **The item cannot be installed in a suit** (upstream's `IMC` registration),
   and it is not a Mekanism `ModuleItem`; `Rarity.RARE` and stack size are the
   only item properties carried.
8. **The tooltip is code-complete but not driven in-game**: `ctl` cannot hover an
   item, so the two lines are not in the transcript (the mode is asserted
   through `/gmut info` instead).
9. **No Mods-GUI/config page** for the four rule flags: upstream configures them
   through Mekanism's module GUI, which does not exist here.
10. **The content loads only together with the code.** The item declares
    the `gmut:vertical_speed` component, whose id the native `component_types`
    hook registers, so a content-only package is refused
    (`gmut:vertical_speed is not a known component type`,
    `hmod-load-unmodded.log`). That is why content and code are one package; with
    the SDK layout they are one **file**, so the failure mode is gone by
    construction.
11. **SDK era: nothing is re-proved in game yet.** The rules compile and the
    package passes `check`/`fits`, but `cargo horizon-mod check` does not compile
    the mod (`native source: unsandboxed; not executed by the registry check`) and
    this box has no launcher instance for a native build at `0b14f8a`, so the
    behaviour transcript from PR #580 is the last in-game proof. The next slice
    re-points `tools/e2e.sh` at a launcher-built game and re-runs it (control,
    selftest and the per-stack legs).
12. **SDK era: the component registration cannot be checked by the tool.** The
    pin's `docs/mods.md` says it outright — *“Unknown types fail when Horizon
    builds the content root; `cargo horizon-mod check` cannot check native
    registrations”* — so the `gmut:vertical_speed` id in `data/gmut/horizon/item/`
    is only proved when a game loads the package. That is the first assertion of
    the re-pointed e2e.
13. **SDK era: the retired fixtures are gone, not fixed.** `tools/horizon.py`,
    `tools/package.{py,sh}`, `tools/ladder-embed.sh`, `tools/test_{stage,template}.py`,
    `tools/hmod-load.sh` and `tools/audit/` were deleted with the format
    (`removed-files.log`). Their replacement is four commands (see the migration
    section); there is no `stage`, no injected module and no bucket of hooks to
    catalogue any more.


## Provenance (upstream analysis)

Repository
`https://github.com/gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit`,
commit **`aa056b8b58cbb57c0eac34af463fb23eea1beb52`** (2025-08-16); shallow clone
deleted after the findings were written. Files read and behaviour extracted:
[`FINDINGS.md`](FINDINGS.md). The jar stays as licence evidence in
`ports/_shared/licence-cache/GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar`.
Clean-room rewrite: no Java was copied or translated and no jar was decompiled.

## Review repairs (PR #580)

- *"Fix the original-project link in mods/gmut.toml … The public description
  also truncates sentences ('cycled with …', 'turns the step', 'needs a worn')"*
  — the entry links
  `https://github.com/gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit`,
  credits gisellevonbingen, and the description is rewritten with complete
  prose (the five steps, the gate, and flight itself as a gap).
- `mixins/gmut/LICENSE` and `NOTICE.md` travel inside the artifact; staging is
  the orchestrator's (horizon-mods#686).

### Verification after the repairs

- Artifact: `gmut.hnmod`, 16,688 B, SHA-256 `d506ff131a85169c5ad6c1accfedc6edef3f1b6aafcc01c9eb5daaaa87f81306` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/gmut.hmod`).
- `python3 ports/tools/upstream_ready.py gmut` and `check-skills.py --notes ports/gmut`: exit 0.

## Resubmission (after #580 was closed)

PR **#580** was closed on 2026-11-18 with the rest of the `.hnmod` submissions,
because Horizon's mod-API merge (#16941) retired that container. The resubmission is a **new
version** (`0.2.0`), not a patch of the closed PR. The changelog line for its body:

> **Supersedes #580, closed because Horizon retired `.hnmod` (#17992).** Changes: the port is now
> one flat SDK package — `horizon-mod.toml` with `api = "0.2"` and `[native]` pinned to
> `0b14f8acfd97fa2c750e42795249dcc981178ca4` (mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game
> 26.3) — instead of a `.hnmod` bundle with a mixin plan and an embedded content `.hmod`. The six
> hook helpers became typed methods: `component_types`, `register_commands` and `run_mod_command` on
> `mod_api::Hooks`, and `use_item` + `tooltip` on an `ItemPart` attached to the unit (the
> `content_root` injection is gone: the item, its model, texture, names, recipe and advancement are
> `assets/gmut/` and `data/gmut/` inside the same file). The commands run on the parsed invocation
> instead of splitting a string, and feedback goes to the player through the host.
> **`src/runtime.rs` and its 16 tests are unchanged** (byte-identical, sha256 `1c79d041…df58d`).
> Nothing carried is new and nothing implemented before is dropped; the gaps this pin forces
> (no attribute API, no MekaSuit, no keybind, inert recipe) are unchanged and listed above.
> Artifact: `gmut.hmod`, 24,443 B, SHA-256
> `95489434caf1e85ea664a8e118067e78055ee0526b3f589dba7d6d17556d37b5`, 18 entries; `cargo
> horizon-mod check`/`fits` green at `0b14f8a`. The in-game re-run of the scenario is the next
> slice (see gap 11).


## Packaging gate and in-game leg (2026-11-20)

**This section records the packaging-only pass.** Its package (22,112 B `1af4c3ac…`) is superseded
by the fixed one in `## The component write that did not stick` below (22,432 B `a23a1cab…`): the
`Cargo.lock` handling, the ladder commands and the launcher shape it establishes are what the fix
then re-ran unchanged.

The package carried a packed `Cargo.lock` (18 entries): `cargo test` writes one into the port
directory and `cargo horizon-mod build` packs the directory's file list. Rebuilt with
`rm -f Cargo.lock` before **each** build — twice, byte-identical — and nothing else changed:

| step | command | result |
|---|---|---|
| lock | `rm -f Cargo.lock` before every build | absent before both builds and after them |
| tests | `cargo test --offline` | **16 passed**, 0 failed, exit 0 (`evidence/gmut/sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| format | `rustfmt --edition 2024 --check src/*.rs` | exit 0 (`sdk-fmt.log`; `cargo fmt --all` reaches into `.horizon/sdk`) |
| build 1 + 2 | `cargo horizon-mod build`, `cmp` | **byte-identical** (`sdk-build.log`, `sdk-build-2.log`) |
| package | `unzip -l gmut.hmod` | **22,112 B**, SHA-256 `1af4c3ac129b2f06e3eb1ab795f735e0565ab9e61d4b7a61d2ff05180a02d2a7`, **17 entries**, `no packed Cargo.lock` |
| package check | `cargo horizon-mod check gmut.hmod` | exit 0 (`sdk-check.log`) |
| fits | `cargo horizon-mod fits gmut.hmod --build 0b14f8a` | exit 0, *"0b14f8a made for"* (`sdk-fits.log`) |
| gate | `python3 ports/tools/verify-sdk-package.py gmut` | **1/1** |
| in-game (modded) | `launcher-game.sh build gmut --rebuild` then the leg | **FAIL** + `EXIT:1` (`evidence/gmut-e2e/e2e-launcher.log`) |

`tools/e2e.sh` was re-pointed from the retired `.hnmod` ladder (`.horizon-cache`, `--dev-mod`,
`content/gmut.hmod`) at the launcher-built game: it now launches `$GAME` with no `--dev-mod`, and
the control is the same Nightly without the package (`GAME=$CONTROL_GAME`). The leg reaches C and
fails there:

```
== C: a right click turns the step forward ==
  message: Vertical Speed Modifier: low (1.5x)
  gmut: held, mode=off (1.0x), …
FAIL: the mode is 'off', expected 'low'
```

The right click prints the module's own line, so the server branch of `on_use_item` ran and
`set_hand_mode` answered `Some`; the `/gmut info` read afterwards still sees `mode=off`. The
component write is not in the stack by then. **Fixed the same day — see
`## The component write that did not stick (2026-11-20)` below**; that pass changed `src/support.rs`
only, not `runtime.rs`, the manifest or the version.

## The component write that did not stick (2026-11-20)

**Defect.** A right click on the held unit printed `Vertical Speed Modifier: low (1.5x)` and left
the held stack on `mode=off`; the leg failed at C with `FAIL: the mode is 'off', expected 'low'`
(`evidence/gmut-e2e/e2e-launcher.log`, 2026-11-20 01:04). The write was reported as landed
(`set_hand_mode` answered `Some`), so the glue's own success value was wrong too.

**Where it went (proved, not guessed).** The pinning assertion added to C — `expect_one_unit`,
"a component write must change the held stack, never add one" — was run against the unfixed
package and printed the whole inventory (`evidence/gmut-e2e/e2e-red-unit-stacks.log`):

```
== C: a right click turns the step forward ==
  unit stacks: [{"slot":0,"count":1},{"slot":1,"count":1}]
FAIL: the right click left 2 unit stacks, expected 1 …
  {"hotbar":[{"count":1,"item":"gmut:…unit","slot":0},
             {"components":"{\"gmut:vertical_speed\":\"low\"}","count":1,"item":"gmut:…unit","slot":1}, …
```

So the held stack (slot 0) kept no components, and a **second** unit carrying
`gmut:vertical_speed: "low"` was added to slot 1: the mutation was applied to a copy that was
written back into the player's *inventory*, not to the hand.

**The symbol chain.** `set_hand_mode` wrote through `ItemPlace::edit`
(`horizon-mod-vocab/src/block/item_place.rs:580`), whose last step is
`self.exchange(host, tx, &ItemResource::of(&stack), count) == count` — an **exchange**, i.e.
`extract` the old stack and `insert` the new one. Both reach the game's own place writer
(`ItemPlace::moved_at`'s `_ => backend.game_insert / game_extract`,
`horizon-client/src/game/modding/native_item_places.rs`), and for a **creative** player's hand that
pair cannot change the stack:

- `game_extract` (line 313): `let creative = g.creative && matches!(place, ItemPlace::PlayerHand { .. }); … if took == 0 || creative { return; }` — it answers the count without taking anything.
- `game_insert` (line 176): `if g.creative && matches!(place, ItemPlace::PlayerHand { .. }) { … if !have…any(|h| h.same_item_same_components(&stack)) { g.add_stack_to_inventory(ItemStack { count: 1, ..stack }); } return; }` — it never writes the hand; the new stack goes to the inventory.

`exchange` therefore sees `out == amount && insert == out`, reports success, and `edit` answers
`true`. That is why the mode line printed and the read still said `off`: the only stack that
received `low` was the new one in slot 1.

**Fix.** Write the hand's own stack directly with `LivingRef::set_hand_item`
(`living_set_hand_item`, `LivingEntity.setItemInHand`, API 382): the port reads the held stack,
patches the component on that copy, and writes it back into the same hand. The API documents it as
"a direct write: it takes effect and is final immediately", so the transaction wrapper went with the
exchange. `write_mode` (the `apply_components` patch) is unchanged; a refused patch still leaves
the stack as it was and is not reported as changed.

**Why this is the port's fix, not the engine's.** The exchange semantics are the documented
behaviour of a creative place (`forInfiniteMaterials`: "an extract gives without taking and an
insert is taken whole, one of it going to the player only if they have none"), so
`ItemPlace::edit` is simply the wrong write for "change this stack in place", and `set_hand_item`
is the API's in-place stack write. Worth an upstream note only as documentation (an `edit` on a
creative hand cannot land); no upstream report filed.

**Test that pins it.** `tools/e2e.sh` C and D now call `expect_one_unit`, which asserts that the
whole inventory (hotbar *and* main) holds exactly one unit stack after each right click, and the
failure message prints both lists. Watched red on the unfixed package
(`e2e-red-unit-stacks.log`, `2 unit stacks`) and green after the fix
(`e2e-launcher.log`, `unit stacks: [{"slot":0,"count":1}]` with `mode=low`, then `mode=off`).

**Version.** Stays `0.2.0`: the fixed package has never been published (PR #580 was the retired
`.hnmod`), so no consumer holds an older `0.2.0`.

### Verification after the fix (2026-11-20)

All steps through `ports/tools/build-slot.sh` with `BUILD_LOCAL=1` (the SDK path needs the fetched
`.horizon/sdk`), `rm -f Cargo.lock` before every build, `CARGO_INCREMENTAL=0`, no `--target-dir`.

| step | command | evidence | result |
|---|---|---|---|
| tests | `cargo test --offline` | `evidence/gmut/sdk-test.log` | **16 passed**, 0 failed, exit 0 |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| format | `rustfmt --edition 2024 --check src/*.rs` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `cargo horizon-mod build`, `cmp` | `sdk-build.log`, `sdk-build-2.log` | **byte-identical**, `a23a1cab…e0e5` |
| package | `unzip -l gmut.hmod` | — | **22,432 B**, SHA-256 `a23a1cabdd725915a1d1f4d828ec924dfea24ca05431a3762176e385c437e0e5`, **17 entries**, no packed `Cargo.lock` |
| package check | `cargo horizon-mod check gmut.hmod` | `sdk-check.log` | exit 0 |
| fits | `cargo horizon-mod fits gmut.hmod --build 0b14f8a` | `sdk-fits.log` | exit 0, *"0b14f8a made for"* |
| gate | `python3 ports/tools/verify-sdk-package.py gmut` | — | **1/1** |
| in-game (modded) | `launcher-game.sh build gmut`, then the leg | `evidence/gmut-e2e/e2e-launcher.log` | **PASS** + `EXIT:0`; C and D each `unit stacks: [{"slot":0,"count":1}]` |
| in-game (control) | `GAME=$CONTROL_GAME`, same command | `e2e-launcher-control.log` | **FAIL** + `EXIT:1` (`the unit answered no info line at all`) |
| fail-capability | `SELFTEST=1` | `e2e-launcher-selftest.log` | **FAIL** + `EXIT:1` (`the mode is 'low', expected 'high'`) |
| logs gate | `python3 ports/tools/check-e2e-logs.py gmut-e2e --strict` | — | **4 ok, 0 missing-exit, 0 bad-exit** |

## Review repairs — 0.2.1 (2026-11-21)

The maintainer's 2026-11-20 22:33 review of `#1434` (at `559664fc`) raised three findings; this
section is the record of each fix and the validation the PR body can link.

### 1. The licence notice is complete

`LICENSE` now carries the upstream copyright line **verbatim**, with `2022` restored:

```
Copyright (c) 2022 gisellevonbingen
Copyright (c) 2026 AdLf0208 (this Horizon port)
```

The exact upstream line was read from the repository's `LICENSE` at its `1.21.1` branch (the branch
whose `gradle.properties` says `mod_version=6.4`), not guessed. The template's stray
`Horizon Veinminer contributors` line was removed: it was copied from another port and never
described this one. `LICENSE-NOTICE.md` now names the package-root files that actually exist
(`LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `horizon-mod.toml`, `Cargo.toml`, `src/`, `assets/gmut/`,
`data/gmut/`, `uses.json`) and records that the item texture is upstream's byte for byte, so the
upstream MIT notice covers it.

### 2. The listing text now matches the release

The old text said the step "changes vertical velocity" and described Mekanism recipe content. Neither
is true. The authoritative paragraph is §“What the item does — authoritative text for the listing
(0.2.1)” above; `README.md`, `horizon-mod.toml`'s `description` and `CONTENT.md` were rewritten from
it. In short: the item **stores and cycles** one of five steps; it does **not** change velocity,
gravity, flight, FOV or sound; and **no Mekanism recipe content is carried** — the jar's recipe and
advancement were removed in 0.2.1 because their ingredients do not exist at this pin and the game
rejected them at load.

### 3. The tagged validation is here

§“Validation at 0.2.1” below is the section to link. There is no `RECEIPT.md` in the publishing
repository, and this section is what replaces it.

### Upstream-defect note: the carried recipe was dead data

The jar's shaped recipe (`mekanism:module_base`, `mekanism:alloys/reinforced`,
`c:circuits/ultimate`, `c:pellets/polonium`) can never match at this pin, and shipping it produced
nine `recipes: invalid ingredient (unresolved tag or malformed input)` errors on every load
(`evidence/gmut-e2e/e2e-launcher.log`, 2026-11-21 01:06 run). Per the campaign's “a carried recipe
that is never crafted is dead data: test it, or drop it and say why”, both the recipe and its
advancement were removed. The item is give/creative only, which the listing now says.
`tools/carry_content.py` still regenerates both from the kept jar, so the slice that lands Mekanism
re-adds them. The 0.2.1 leg logs **zero** invalid-ingredient errors.

## Validation at `0.2.1` (2026-11-21)

This is the tagged validation section: it names the version, the exact bytes and sha256, and every
step of the ladder, so a pull-request body can link it instead of a missing receipt.

**Artifact.** `gmut.hmod`, **22,177 B**, SHA-256
**`c3f91a06d4df0c8b0935fff082b1846ab605d4bc6b7f940e3d505e2d6a0a3d76`**, **15 entries**, declared
version **0.2.1**, `api = "0.2"`, `[native]` at Horizon
`0b14f8acfd97fa2c750e42795249dcc981178ca4` + mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3. No packed `Cargo.lock` (removed before each
build). Two consecutive builds with `rm -f Cargo.lock` before each were **byte-identical** (`cmp`
clean, same sha256).

| Step | Command (inside a `build-slot.sh` lease) | Evidence | Result |
|---|---|---|---|
| tests | `cargo test --offline --manifest-path ports/gmut/Cargo.toml` | `evidence/gmut/sdk-test.log` | **16 passed**, 0 failed, exit 0 |
| clippy | `cargo clippy --all-targets --offline … -- -D warnings` | `sdk-clippy.log` | exit 0, **0 warnings** |
| format | `rustfmt --edition 2024 --check src/*.rs` | `sdk-fmt.log` | exit 0 |
| build 1 + 2 | `rm -f Cargo.lock && cargo horizon-mod build`, twice, `cmp` | `sdk-build.log`, `sdk-build-2.log` | **byte-identical**, `c3f91a06…a0a3d76` |
| package | `unzip -l gmut.hmod` | — | **22,177 B**, **15 entries**, no `Cargo.lock` |
| package check | `cargo horizon-mod check gmut.hmod` | `sdk-check.log` | exit 0: `gmut 0.2.1 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3) … 22177 bytes … unsigned` |
| fits, the pin | `cargo horizon-mod fits gmut.hmod --build 0b14f8a` | `sdk-fits.log` | exit 0: `0b14f8a  made for` |
| fits, **a different build** | `cargo horizon-mod fits gmut.hmod --build 0a875da9` | `sdk-fits-other.log` | exit 0: `0a875da  fits` (SDK release 2026-11-20) |
| gate | `python3 ports/tools/verify-sdk-package.py gmut` | `verify-sdk-package.log` | **1/1** |
| licence in the archive | `unzip -p gmut.hmod LICENSE \| grep -n Copyright` | `LICENSE` in the archive | line 3 `Copyright (c) 2022 gisellevonbingen` |
| retired paths | `grep -rn -E '\.hnmod\|mixins/' <unpacked archive>` | — | **NONE** |
| in-game (modded) | `launcher-game.sh build gmut`, then the leg in a slot | `evidence/gmut-e2e/e2e-launcher.log` | **PASS** + `EXIT:0` |
| in-game (control) | `GAME=$CONTROL_GAME`, same command | `e2e-launcher-control.log` | **FAIL** + `EXIT:1` (`the unit answered no info line at all`) |
| fail-capability | `SELFTEST=1`, same command | `e2e-launcher-selftest.log` | **FAIL** + `EXIT:1` (`the mode is 'low', expected 'high'`) |
| logs gate | `python3 ports/tools/check-e2e-logs.py gmut-e2e --strict` | — | **4 ok, 0 missing-exit, 0 bad-exit** |

The leg (`e2e-launcher.log`) proves, with parsed `ctl` replies: A `absent`; B `held` with the OFF
default; C five right clicks through `low, medium, high, ultra, off`, each changing the stack's own
`gmut:vertical_speed` component and leaving **one** unit stack; D sneak+right click back to `ultra`;
E `/gmut mode` on all five steps plus `next` wrapping `ultra→off` and `prev` wrapping `off→ultra`;
F two units holding different steps across slot switches; G `carried(2)`; H an unknown `gmut:` id
refused; I `off_hand`, then back to `carried(2)`; J `flying=false`, `fall_flying=false` with the step
on `ultra`, and `modulation=needs_gravitational_modulating_unit` on every line.

## Coverage — every family the jar ships

| Family (from the jar) | Status | Evidence / reason |
|---|---|---|
| The unit item: id, model, texture, name, stack size 1, rarity rare, creative tab | **tested** | B: `ctl give` + `ctl inventory` reads the id back; `item.gmut.*` name in `assets/gmut/lang/en_us.json`; the tab carries the id |
| The five-step setting, stored per stack in the item's component | **tested** | C: `stored component: gmut:vertical_speed=low/medium/high/ultra/off`; F: two units hold `ultra` and `low` independently |
| Right-click cycle, forward and back while sneaking | **tested** | C (five steps forward), D (`off→ultra` while sneaking) |
| `/gmut mode <step>`, `/gmut next`, `/gmut prev` | **tested** | E: all five via `mode`; `next` wraps `ultra→off`; `prev` wraps `off→ultra`; replies parsed |
| `/gmut info`: placement, stored step, rule defaults, gate | **tested** | A/B/C/D/E/F/G/I/J parse the line; all four placements (`absent`, `held`, `carried(2)`, `off_hand`) appear |
| Per-stack independence | **tested** | F: slot 1 `ultra`, slot 2 `low`, each re-read after switching |
| Tooltip: stored step + “Requires a Gravitational Modulating Unit” | **blocked(ctl-hover)** | `horizon-ctl` has no item-hover command; the code path is compile-checked and the values it renders are asserted through `/gmut info` |
| Flight, gravity, FOV fix, sound mute, the vertical-speed multiplier | **not ported** | J: `flying=false` with the step on `ultra`; the gate is always `needs_gravitational_modulating_unit`; `blocked(no-mekasuit)` (Mekanism unported) |
| MekaSuit installation, module GUI, HUD, config page | **not ported** | no Mekanism and no module container at this pin; `blocked(no-mekasuit)` |
| The jar's shaped recipe + its advancement | **blocked(no-mekanism)** | removed in 0.2.1: every ingredient is absent at this pin and the game rejected the recipe at load (`invalid ingredient`); `tools/carry_content.py` regenerates both |
| The keybind (cycle on a key, sneak for backwards) | **not ported** | no client→server key action at this pin; the right click and `/gmut` carry the same direction rule |

Skills: superpowers/using-superpowers,verification-before-completion,systematic-debugging; BeautifulRust/none (no Rust change)
