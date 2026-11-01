/// Where a mod stands in the load, enable, disable, unload sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModPhase {
    /// `amml_mod_load` succeeded.
    Loaded,
    Enabled,
    /// `amml_mod_disable` ran.
    Disabled,
    /// `amml_mod_unload` ran. The DLL stays mapped.
    Unloaded,
    /// The mod asks for an API this loader does not have.
    Refused,
    /// A call failed or panicked; the mod is switched off and its hooks are removed.
    Failed,
}
