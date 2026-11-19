//! Gravitational Modulating Additional Unit for Horizon: the unit item and the step it carries.
//!
//! Upstream (see `FINDINGS.md`) is a Mekanism MekaSuit module: a
//! `ModuleGravitationalModulatingAdditionalUnit` installed in the bodyarmor with five config items
//! (`fly_always`, `stop_immediately`, `fix_fov`, `vertical_speed`, `mute_modulating_sound`) and a
//! keybind that cycles `vertical_speed` (+1, or -1 while sneaking). All of it is gated on
//! `CommonPlayerTickHandler.isGravitationalModulationReady(player)`.
//!
//! The pure rules — the five steps, the direction a switch turns, the rule defaults and the
//! `/gmut info` line — live in [`runtime`], a plain module with no `mod_api` types, so `cargo test`
//! exercises them without the game. [`support`] is the glue: the item's component, the placed
//! unit's step, the commands and the tooltip, all through typed hosts and events.
pub mod runtime;
mod support;

use mod_api::{
    self, ClientHost, ComponentTypesEvent, Flow, Hooks, HostSide, InteractionResult, ItemPart,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
    TooltipEvent, UseItemEvent,
};

/// The mod has no settings: what it keeps is the step each unit stack carries.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Request the unit item and attach [`Unit`] to it, before content loads.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        support::register(registrar);
    }

    /// Register `gmut:vertical_speed`, the component the carried item declares its step in. The
    /// content item is refused at load when this id is unknown, so it must be pushed here.
    fn component_types(&self, _context: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        support::component_types(event);
    }

    /// `/gmut info|mode|next|prev`: the harness controls the in-game check drives.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::register_commands(event);
    }

    /// The bodies of those commands, on the already-parsed invocation.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }
}

/// The unit's part: what its stack does on a right click and what its tooltip adds.
///
/// It is attached to one item only, so every moment below is this mod's item's and no id check is
/// needed inside them.
struct Unit;

#[mod_api::horizon_part]
impl ItemPart for Unit {
    /// Right click: cycle the step forward, or back while sneaking.
    fn use_item(
        &self,
        host: HostSide<'_>,
        event: &mut UseItemEvent<'_>,
    ) -> Option<InteractionResult> {
        support::on_use_item(host, event)
    }

    /// The step the stack carries, and the gate its effects sit behind.
    fn tooltip(&self, host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        support::on_tooltip(host, event);
    }
}
