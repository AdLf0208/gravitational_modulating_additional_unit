//! Game-side glue for the gravitational modulating additional unit: the item's component, the
//! placed unit's step, the `/gmut` commands and the tooltip.
//!
//! Upstream's module keeps its mode in a Mekanism `ModuleConfigItem` on the MekaSuit stack,
//! cycles it from a keybind packet, and gates every effect on
//! `CommonPlayerTickHandler.isGravitationalModulationReady(player)`. This pin has no
//! keybind→server path, no MekaSuit and no attribute API, so the slice keeps the mode as the
//! item's own `gmut:vertical_speed` component, cycled by right click or `/gmut`, and reports the
//! gate instead of faking it. Everything here is a typed host call; the rules are in
//! [`crate::runtime`].

use std::sync::OnceLock;

use mod_api::{
    ClientHost, Command, CommandArgumentValue, CommandSource, ComponentTypesEvent, Compound, Flow,
    Hand, HostSide, InteractionResult, ItemKey, ItemPlace, ItemStack, PlayerRef,
    RegisterCommandsEvent, Registrar, RunModCommandEvent, ServerHost, Tag, Text, TooltipEvent,
    UseItemEvent,
};

use crate::runtime::{
    Gate, ITEM, MODE_COMPONENT, Placement, Reading, Rules, VerticalSpeed, gate, mode_line,
    shift_for,
};

/// The unit item's key, requested once at registration. The Part is attached to this item alone,
/// so the id is only needed where a stack is read without an event: the carried count.
static UNIT: OnceLock<ItemKey> = OnceLock::new();

/// A stack of the unit, for the checks that read an inventory slot rather than an event.
fn is_unit(stack: &ItemStack) -> bool {
    UNIT.get().is_some_and(|unit| *unit == stack.item())
}

/// Request the unit item and attach the item Part to it.
pub fn register(registrar: &mut dyn Registrar) {
    let item = registrar.item_key(ITEM);
    // The first registration wins; a mod is registered once per process.
    let _ = UNIT.set(item);
    registrar.item_part(item, crate::Unit);
}

/// Upstream's gate (`CommonPlayerTickHandler.isGravitationalModulationReady`): the worn suit
/// carries a working gravitational modulating unit. Mekanism is unported at this pin and there is
/// no armor/module API to read, so the gate is **always** closed; `/gmut info` says so instead of
/// pretending otherwise.
fn modulation_ready() -> bool {
    false
}

/// The step a unit stack carries, OFF when it carries none.
pub fn read_mode(stack: &ItemStack) -> VerticalSpeed {
    let Some(Tag::String(name)) = stack
        .components()
        .and_then(|patch| patch.as_nbt().get(&MODE_COMPONENT.into()))
    else {
        return VerticalSpeed::default();
    };
    // A name this build does not know (a newer or hand-edited component) reads as OFF, as an
    // absent component does: the item still works, on its default step.
    VerticalSpeed::from_name(&name.to_string_lossy()).unwrap_or_default()
}

/// Store a step on a unit stack, as the module's config item keeps it. False when the stack
/// refused the patch, which leaves the step it had.
fn write_mode(stack: &mut ItemStack, mode: VerticalSpeed) -> bool {
    let patch: Compound = [(MODE_COMPONENT.into(), Tag::String(mode.name().into()))]
        .into_iter()
        .collect();
    stack.apply_components(&patch).is_ok()
}

/// Whether the player is sneaking, the direction upstream's keybind turns back.
fn sneaking(host: &dyn ServerHost, player: PlayerRef) -> bool {
    player.entity().is_shift_key_down(host).unwrap_or(false)
}

/// How many units the player's 36 inventory slots and off hand hold.
///
/// Not the item-count helper: that counts only *plain* stacks, and this item always carries its
/// `gmut:vertical_speed` component, so it would always answer 0. The slot numbers are
/// `Inventory`'s: 0 to 35 the main inventory, 40 the off hand.
fn carried_count(host: &dyn ServerHost, player: PlayerRef) -> u32 {
    player
        .inventory_slots(host)
        .unwrap_or_default()
        .into_iter()
        .filter(|(slot, _)| *slot <= 35 || *slot == 40)
        .filter(|(_, stack)| is_unit(stack))
        .map(|(_, stack)| stack.count())
        .sum()
}

/// Whether `hand` holds a unit.
fn holds(host: &mut dyn ServerHost, player: PlayerRef, hand: Hand) -> bool {
    ItemPlace::PlayerHand {
        player: player.uuid(),
        hand,
    }
    .stack(host)
    .is_some_and(|stack| is_unit(&stack))
}

/// Where the player's unit is: a hand, or the pack, or nowhere.
fn placement(host: &mut dyn ServerHost, player: PlayerRef) -> Placement {
    if holds(host, player, Hand::Main) {
        return Placement::Held;
    }
    if holds(host, player, Hand::Off) {
        return Placement::OffHand;
    }
    match carried_count(host, player) {
        0 => Placement::Absent,
        count => Placement::Carried(count),
    }
}

/// The reading `/gmut info` prints for a player.
fn reading(host: &mut dyn ServerHost, player: PlayerRef) -> Reading {
    let mode = [Hand::Main, Hand::Off]
        .into_iter()
        .find_map(|hand| {
            let stack = ItemPlace::PlayerHand {
                player: player.uuid(),
                hand,
            }
            .stack(host)?;
            is_unit(&stack).then(|| read_mode(&stack))
        })
        .unwrap_or_default();
    Reading {
        placement: placement(host, player),
        mode,
        rules: Rules::new(),
        gate: gate(modulation_ready()),
    }
}

/// Put `mode` on the unit in `hand`; the step it replaced, or `None` when that hand holds no
/// unit. The write is the hand slot's own, not the place's exchange: a component write must
/// change the stack that is already there, and an exchange cannot (see below).
fn set_hand_mode(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    hand: Hand,
    mode: VerticalSpeed,
) -> Option<VerticalSpeed> {
    let place = ItemPlace::PlayerHand {
        player: player.uuid(),
        hand,
    };
    let mut stack = place.stack(host)?;
    if !is_unit(&stack) {
        return None;
    }
    let previous = read_mode(&stack);
    if !write_mode(&mut stack, mode) {
        return None;
    }
    // `ItemPlace::edit` is an *exchange* (`ItemPlace::exchange`: extract the old stack, insert
    // the new one), and a creative player's hand is a give-only place at this pin: the game's
    // `game_extract` answers the count without taking anything (`if took == 0 || creative`),
    // and `game_insert` refuses a hand that holds the same item with other components and puts
    // the new stack in the player's inventory instead. So an exchange reports success, adds a
    // second unit and leaves the held one as it was. `set_hand_item` is the direct write
    // (`LivingEntity.setItemInHand`), final immediately, and it keeps the item and the count.
    let living = host.living(player.entity())?;
    let wrote = living
        .set_hand_item(host, hand, Some(stack))
        .unwrap_or(false);
    wrote.then_some(previous)
}

/// Print a line to the player whose command is running.
fn tell(host: &mut dyn ServerHost, player: PlayerRef, line: impl Into<String>) {
    let _ = player.tell(host, Text::literal(line.into()));
}

/// Right click with the unit (`use_item`): cycle the step, as upstream's keybind does — forward,
/// or back while sneaking.
pub fn on_use_item(host: HostSide<'_>, event: &mut UseItemEvent<'_>) -> Option<InteractionResult> {
    let HostSide::Server(host) = host else {
        // The client predicts the swing and writes nothing: the step is the server's.
        return Some(InteractionResult::Success);
    };
    let player = host.player(event.player)?;
    let shift = shift_for(sneaking(host, player));
    let next = read_mode(event.stack).adjust(shift);
    if set_hand_mode(host, player, event.hand, next).is_some() {
        tell(host, player, mode_line(next));
    }
    Some(InteractionResult::SuccessServer)
}

/// The unit's tooltip: the step it carries and the gate its effects sit behind
/// (`stop_immediately` and the rest are the module's own defaults).
pub fn on_tooltip(_host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
    let mode = read_mode(event.stack);
    let label = format!("{} ({:.1}x)", mode.name(), mode.multiplier());
    event.lines.push((
        Text::translate("gmut.tooltip.mode", vec![Text::literal(label)]),
        0xAAAAAA,
    ));
    if gate(modulation_ready()) == Gate::NeedsGravitationalModulatingUnit {
        event.lines.push((
            Text::translate("gmut.tooltip.modulation", Vec::new()),
            0xFF5555,
        ));
    }
}

/// The component id the carried item declares its step in.
pub fn component_types(event: &mut ComponentTypesEvent<'_>) {
    event.types.push(MODE_COMPONENT.to_owned());
}

/// `/gmut info|mode|next|prev`: the harness controls the in-game check drives. Upstream has no
/// commands; its keybind, its config items and the MekaSuit GUI are the controls, and none of the
/// three exists at this pin.
pub fn register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event.commands.push(Command::new("gmut info"));
    event
        .commands
        .push(Command::op("gmut mode <speed:word>", 2));
    event.commands.push(Command::op("gmut next", 2));
    event.commands.push(Command::op("gmut prev", 2));
}

/// The steps `/gmut mode` accepts, for its usage line.
fn step_list() -> String {
    VerticalSpeed::ALL
        .iter()
        .map(|step| step.name())
        .collect::<Vec<_>>()
        .join("|")
}

/// Run a `/gmut` command a player typed. The parsed invocation is already split into its literal
/// path and its named argument, so no whitespace parsing happens here.
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let CommandSource::Player { player, .. } = event.source else {
        // The readout is a player's; the console has no inventory to report.
        return Flow::Continue;
    };
    let player = *player;
    let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
    match literals.as_slice() {
        ["gmut", "info"] => {
            let line = reading(host, player).line();
            tell(host, player, line);
        }
        ["gmut", "mode"] => {
            let Some(speed) = event
                .command
                .argument("speed")
                .and_then(|value| match value {
                    CommandArgumentValue::String(name) => Some(name.as_str()),
                    _ => None,
                })
            else {
                tell(host, player, format!("Usage: /gmut mode <{}>", step_list()));
                return Flow::Cancel;
            };
            match VerticalSpeed::from_name(speed) {
                Some(mode) => report_mode(host, player, mode),
                None => tell(host, player, format!("Usage: /gmut mode <{}>", step_list())),
            }
        }
        ["gmut", "next"] => cycle(host, player, true),
        ["gmut", "prev"] => cycle(host, player, false),
        _ => return Flow::Continue,
    }
    Flow::Cancel
}

/// Turn the held unit one step (forward, or back) and report where it landed.
fn cycle(host: &mut dyn ServerHost, player: PlayerRef, forward: bool) {
    let Some(current) = ItemPlace::PlayerHand {
        player: player.uuid(),
        hand: Hand::Main,
    }
    .stack(host)
    .filter(is_unit)
    .map(|stack| read_mode(&stack)) else {
        tell(host, player, "Hold the unit first.");
        return;
    };
    let next = if forward {
        current.next()
    } else {
        current.previous()
    };
    report_mode(host, player, next);
}

/// Put `mode` on the unit in `hand` and say which step it landed on. A held unit whose stack
/// refused the patch is left as it was and is not reported as changed.
fn report_mode(host: &mut dyn ServerHost, player: PlayerRef, mode: VerticalSpeed) {
    match set_hand_mode(host, player, Hand::Main, mode) {
        Some(_) => tell(host, player, mode_line(mode)),
        None if !holds(host, player, Hand::Main) => tell(host, player, "Hold the unit first."),
        None => {}
    }
}
