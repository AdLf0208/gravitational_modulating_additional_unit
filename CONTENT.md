# Content carry — `ports/gmut/assets` and `ports/gmut/data`

Written by [`tools/carry_content.py`](tools/carry_content.py) from
`ports/_shared/licence-cache/GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar`
(`sha256 11c078cf…982d`). Re-running the script reproduces every file
byte-for-byte; `--check` lists them without writing.

**Layout note (2026-11-18):** the files below are the same bytes the carry produced, moved from
`content/assets/` and `content/data/` to the SDK package's `assets/` and `data/` at the port root;
the script still writes into `content/`, so run it into a scratch directory and compare. The
content-only `horizon-mod.toml` it wrote is replaced by the root manifest.

**What the content is not.** The jar's recipe and its advancement are **not carried** in 0.2.1: every
ingredient is a Mekanism item or tag that does not exist at this pin, so the recipe can never match
and the game rejected it at load (`recipes: invalid ingredient (unresolved tag or malformed input):
{"tag":"mekanism:alloys/reinforced"}`). They are left out until Mekanism lands, and
[`tools/carry_content.py`](tools/carry_content.py) regenerates them from the kept jar then. The item
is reachable by `give` / the creative tab only. The lang file keeps the jar's Mekanism module-GUI
keys for later slices; this port implements no suit GUI. The item's `gmut:vertical_speed` component
stores a step and nothing more — the port does not change the player's velocity (see
[`README.md`](README.md) and `notes.md` §“What the item does”).

## Carried verbatim from the jar

| File | Why |
|---|---|
| `assets/gmut/models/item/module_gravitational_modulating_additional_unit.json` | the item's model (`minecraft:item/generated`, one layer) |
| `assets/gmut/textures/item/module_gravitational_modulating_additional_unit.png` | its texture |
| `assets/gmut/lang/en_us.json` | the jar's names (module labels, key description, item group) — useful for the reading and for later slices |

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
3. **The recipe and its advancement** (removed in 0.2.1): the recipe's ingredients are
   Mekanism's and the `c:` convention — `mekanism:module_base`, `mekanism:alloys/reinforced`,
   `c:circuits/ultimate`, `c:pellets/polonium`. Mekanism is unported and the pin ships no `c:` tags,
   so the recipe can never match, and the game rejected it at load:
   `recipes: invalid ingredient (unresolved tag or malformed input):
   {"tag":"mekanism:alloys/reinforced"}` (nine such lines per load in
   `evidence/gmut-e2e/e2e-launcher.log`). Shipping dead data that logs errors is worse than not
   shipping it, so both files are left out until Mekanism lands; `tools/carry_content.py` still
   regenerates them from the jar. The item is give/creative only.
4. **The recipe advancement's criterion** names
   `mekanism:module_gravitational_modulating_unit`; it was removed with the recipe for the same
   reason. Nothing reads advancements in-game here.
