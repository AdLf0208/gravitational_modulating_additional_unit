# Content carry — `ports/gmut/assets` and `ports/gmut/data`

Written by [`tools/carry_content.py`](tools/carry_content.py) from
`ports/_shared/licence-cache/GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar`
(`sha256 11c078cf…982d`). Re-running the script reproduces every file
byte-for-byte; `--check` lists them without writing.

**Layout note (2026-11-18):** the files below are the same bytes the carry produced, moved from
`content/assets/` and `content/data/` to the SDK package's `assets/` and `data/` at the port root;
the script still writes into `content/`, so run it into a scratch directory and compare. The
content-only `horizon-mod.toml` it wrote is replaced by the root manifest.

## Carried verbatim from the jar

| File | Why |
|---|---|
| `assets/gmut/models/item/module_gravitational_modulating_additional_unit.json` | the item's model (`minecraft:item/generated`, one layer) |
| `assets/gmut/textures/item/module_gravitational_modulating_additional_unit.png` | its texture |
| `assets/gmut/lang/en_us.json` | the jar's names (module labels, key description, item group) — useful for the reading and for later slices |
| `data/gmut/recipe/module_gravitational_modulating_additional_unit.json` | the shaped recipe, unchanged |
| `data/gmut/advancement/recipes/misc/module_gravitational_modulating_additional_unit.json` | its recipe advancement, unchanged |

## Written for 26.3 / Horizon (not in the jar)

| File | Why |
|---|---|
| `assets/gmut/items/module_gravitational_modulating_additional_unit.json` | 26.3 item definition (`{"model":{"type":"minecraft:model",…}}`); 1.21.1 has none |
| `data/gmut/horizon/item/module_gravitational_modulating_additional_unit.json` | the item declaration: `minecraft:paper` template, `max_stack_size: 1`, `rarity: rare` (upstream's builder), and the initial `gmut:vertical_speed: "off"` component |
| `data/gmut/horizon/creative_tab/gmut.json` | a tab with the one item (upstream has no tab of its own; its item is only reachable through Mekanism's) |
| `horizon-mod.toml` | the manifest, `side = "both"`, `capabilities = ["registries"]` |

## Patched after the carry

`assets/gmut/lang/en_us.json` gains four keys (the rest of the jar's file is
untouched):

- `item.gmut.module_gravitational_modulating_additional_unit` — Horizon's item
  name key is `item.<namespace>.<path>`; NeoForge's `module.gmut.*` key would
  render as a raw key.
- `itemGroup.gmut` — the creative tab's title key.
- `gmut.tooltip.mode`, `gmut.tooltip.modulation` — this port's tooltip.

## Left out, and why

1. **`pack.mcmeta`, `META-INF/`, the `.cache/` files, the mixin configs and all
   Java classes** — the `.hmod` is a content package; the behaviour is the
   native half.
2. **Six non-English lang files** (`de_de`, `ja_jp`, `ko_kr`, `pt_br`, `tr_tr`,
   `zh_cn`): carried names exist only in English here; adding translations of
   keys this port does not use would be noise. They can be dropped in later
   without a code change.
3. **The recipe's ingredients are Mekanism's and the `c:` convention:**
   `mekanism:module_base`, `mekanism:alloys/reinforced`, `c:circuits/ultimate`,
   `c:pellets/polonium`. Mekanism is unported and the pin ships no `c:` tags, so
   the carried recipe loads and can never match. It is kept as the faithful
   record of the jar; `notes.md` gap 1 carries the consequence.
4. **The recipe advancement's criterion** names
   `mekanism:module_gravitational_modulating_unit`; like the recipe it is inert
   until Mekanism exists. Nothing reads advancements in-game here.
