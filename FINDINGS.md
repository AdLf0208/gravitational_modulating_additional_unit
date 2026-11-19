# FINDINGS — Gravitational Modulating Additional Unit 6.4

Upstream analysis for `ports/gmut` (slice 1). Source: the mod's own repository
(clean-room reading, no decompilation, no code copied) plus the shipped jar's
own data files.

- **Jar:** `ports/_shared/licence-cache/GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar`
  (`sha256 11c078cf2031ca863a5cec5d2f69590ddee84e67a16066f2b42ee7d78221982d`, 70,705 B,
  68 entries). `META-INF/neoforge.mods.toml`: `modId="gmut"`, `version="6.4"`,
  `license="MIT"`, `mekanism` **required** (`ordering="AFTER"`, `side="BOTH"`).
- **Repository:** `https://github.com/gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit`,
  shallow clone at **`aa056b8b58cbb57c0eac34af463fb23eea1beb52`** (2025-08-16,
  "Update gradle.properties"); the jar is 6.4, the clone's `gradle.properties`
  carries the same version line. Clone deleted after these findings; the jar
  stays as licence evidence.
- Files read: `GravitationalModulatingUnitTweaks.java`,
  `common/registries/GMUTItems.java`, `common/registries/GMUTModules.java`,
  `common/content/gear/mekasuit/ModuleGravitationalModulatingAdditionalUnit.java`,
  `common/content/gear/mekasuit/VerticalSpeed.java`,
  `common/content/gear/mekasuit/EntityModuleHelper.java`,
  `common/network/GMUTPacketHandler.java`,
  `common/network/to_server/PacketSwitchVerticalSpeedPacket.java`,
  `client/key/GMUTKeyHandler.java`, `client/ClientEventHandler.java`,
  `client/mixin/GravitationalModulationSoundMixin.java`, `common/GMUTLang.java`,
  plus the jar's `assets/gmut/lang/en_us.json`, the item model and the recipe.

## What the mod is

A **Mekanism MekaSuit module**, not a standalone item: `GMUTModules` registers it
through `ModuleDeferredRegister` with `maxStackSize(1).rarity(Rarity.RARE)`, and
`InterModEnqueueEvent` hands it to Mekanism's `addMekaSuitBodyarmorModules`, so
the item is a `ModuleItem` that can be installed in the bodyarmor's module
slots. `EntityModuleHelper.getEnabledGMAUModule` scans `entity.getArmorSlots()`
for an enabled module of the type.

The jar carries exactly one item id
(`gmut:module_gravitational_modulating_additional_unit`), its model, one
texture, seven lang files, one shaped recipe and its recipe advancement. No
blocks, no menus, no screens, no entities, no worldgen.

## Behaviour

All of it runs from `tickServer`/`tickClient` on the module and is gated on
`CommonPlayerTickHandler.isGravitationalModulationReady(player)` — the suit
carries a working gravitational modulating unit with energy.

| Config item | Default | Behaviour |
|---|---|---|
| `fly_always` | `false` | server tick: if not sneaking and not already flying, `abilities.flying = true` + `onUpdateAbilities()` |
| `stop_immediately` | `true` | client tick: while flying with `zza == 0 && xxa == 0`, zero the horizontal delta movement |
| `fix_fov` | `false` | client: `ComputeFovModifierEvent` → `setNewFovModifier(1.0)` while the module is worn |
| `vertical_speed` | `OFF` | client tick: while flying, `j = (jump ? +1 : 0) + (shift ? -1 : 0)`; `deltaMovement.y += j * (speed - 1) * flyingSpeed * 3` |
| `mute_modulating_sound` | `false` | client mixin on the suit's modulating sound |

`VerticalSpeed` is an enum with a float label: `OFF 1.0`, `LOW 1.5`,
`MEDIUM 2.0`, `HIGH 2.5`, `ULTRA 3.0`; `getTextComponent()` is the float as text,
so Mekanism's mode-change message shows `2.0`.

**Mode switch.** `GMUTKeyHandler` binds an unbound key; on press it looks for an
enabled module in the armor and sends `PacketSwitchVerticalSpeedPacket(±1)`
(`-1` while `isShiftKeyDown()`), playing Mekanism's hydraulic sound. The server
handler calls `module.changeMode(player, container, shift, true)`, which
`adjust`s the enum with wrap (`MathUtils.getByIndexMod`) and calls
`module.displayModeChange(player, "Vertical Speed Modifier", nextSpeed)`.

**Recipe.** Shaped 3×3 `ACA / ABA / PPP` with `mekanism:alloys/reinforced` (A),
`mekanism:module_base` (B), `c:circuits/ultimate` (C) and `c:pellets/polonium`
(P); the advancement unlocks it when the player has
`mekanism:module_gravitational_modulating_unit`.

## What this pin can and cannot do with it

- **Can:** register the item and its components (content `.hmod`), store a
  per-stack mode, toggle it (right click / command), print a reading.
- **Cannot:** grant or modify flight or gravity. There is **no attribute API**
  (`AttributeInstance` write, `minecraft:gravity`, `mayfly`) and no helper that
  sets a player's abilities; `player_toggle_flight` is a cancellable
  *notification*, and `set_flying_speed` only changes the creative fly speed.
  Mekanism is unported, so neither the suit nor `isGravitationalModulationReady`
  exists. Both are recorded as gaps, not emulated.

## Where the rewrite differs (deliberate)

1. The mode lives in the **item's own component** `gmut:vertical_speed`, not in
   Mekanism's module config on a suit stack — there is no suit and no module
   container.
2. The mode switch is a **right click** (`native_use_item`), or `/gmut
   mode|next|prev`; the keybind and its client→server packet have no stamped
   path at this pin (`key`/`input_key` are client-side listeners only).
3. `fly_always`, `stop_immediately`, `fix_fov`, `mute_modulating_sound` and the
   vertical-speed effect are **not applied**; their defaults are reported by
   `/gmut info` so the state is visible.
4. The gate (`isGravitationalModulationReady`) is reported as
   `needs_gravitational_modulating_unit` — always, at this pin.
