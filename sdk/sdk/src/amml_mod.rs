use amml_abi::{AMML_API_VERSION, ModId};

use crate::{Api, SdkError};

/// A mod. Implement it on a type and give the type to `amml_mod!`, which generates the four
/// exports the loader calls and runs each callback inside `catch_unwind`.
pub trait AmmlMod {
    /// The mod namespace; it must equal `namespace` in `amml.hcl`.
    const NAMESPACE: &'static str;
    /// The mod name; it must equal `name` in `amml.hcl`. With the namespace it names the mod
    /// folder and the mod log file.
    const NAME: &'static str;
    /// The lowest loader API version the mod accepts.
    const API_VERSION: u32 = AMML_API_VERSION;

    /// The mod id, `@<namespace>/<name>`: the module string for `Logger` and `ModDirs`.
    fn id() -> String {
        ModId::format(Self::NAMESPACE, Self::NAME)
    }

    /// Runs under the loader's load phase. Read entries here; install hooks in `enable`.
    ///
    /// In the standalone host this runs before the game's static initializers: do not call game
    /// code or `Api::heap_new` here.
    fn load(_api: &Api) -> Result<(), SdkError> {
        Ok(())
    }

    /// Installs hooks and starts the mod. Only `load` and `enable` may install hooks.
    ///
    /// In the standalone host the game is not initialized yet when this runs: no game calls and no
    /// `Api::heap_new`.
    fn enable(api: &Api) -> Result<(), SdkError>;

    fn disable(_api: &Api) {}

    /// The last call. The DLL stays mapped afterwards: the game may hold objects whose vtables live in it.
    fn unload(_api: &Api) {}
}
