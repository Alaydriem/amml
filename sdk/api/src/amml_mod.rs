use amml_abi::ModId;

use crate::{ApiError, Ctx, DisableCtx, Uses};

/// A mod on the v3 API. Implement it on a type and give the type to `amml_mod!`, which generates
/// the four exports the loader calls and runs each callback inside `catch_unwind`.
///
/// On disable the loader first sends `ProviderGone` to the consumers of the mod's tables, then
/// removes the mod's listeners and waits for their calls on other threads, then calls `disable`. No
/// listener of the mod runs once `disable` starts, so `disable` may tear down what the listeners
/// use; dropping a `Listener` there frees its closure without waiting. The DLL is never unmapped,
/// so a provided table stays readable; it must be a `static`.
pub trait AmmlMod {
    /// The mod namespace; it must equal `namespace` in `amml.hcl`.
    const NAMESPACE: &'static str;
    /// The mod name; it must equal `name` in `amml.hcl`.
    const NAME: &'static str;
    /// What the mod uses, by feature, from `amml_uses!`. A listen of an event named `required` here
    /// is required.
    const USES: Uses = Uses::NONE;

    /// The mod id, `@<namespace>/<name>`.
    fn id() -> String {
        ModId::format(Self::NAMESPACE, Self::NAME)
    }

    /// Runs in the loader's load phase. The game does not run yet: game reads give `NotReady`.
    fn load(_ctx: &Ctx) -> Result<(), ApiError> {
        Ok(())
    }

    /// Adds listeners, provides tables and starts the mod.
    fn enable(ctx: &Ctx) -> Result<(), ApiError>;

    /// Stops the mod. `how.game_alive` tells whether game actions may still be called.
    fn disable(_ctx: &Ctx, _how: &DisableCtx) {}

    /// The last call. The DLL stays mapped afterwards.
    fn unload(_ctx: &Ctx) {}
}
