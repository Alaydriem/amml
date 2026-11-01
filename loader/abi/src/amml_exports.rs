use crate::{AmmlApi, AmmlModInfo, AmmlStatus};

/// Type of `amml_mod_load`.
pub type ModLoadFn = unsafe extern "C" fn(api: *const AmmlApi, info: *mut AmmlModInfo) -> AmmlStatus;
/// Type of `amml_mod_enable`.
pub type ModEnableFn = unsafe extern "C" fn() -> AmmlStatus;
/// Type of `amml_mod_disable`.
pub type ModDisableFn = unsafe extern "C" fn() -> AmmlStatus;
/// Type of `amml_mod_unload`.
pub type ModUnloadFn = unsafe extern "C" fn() -> AmmlStatus;

/// Export names every AMML mod DLL provides.
pub struct AmmlExports;

impl AmmlExports {
    pub const LOAD: &'static str = "amml_mod_load";
    pub const ENABLE: &'static str = "amml_mod_enable";
    pub const DISABLE: &'static str = "amml_mod_disable";
    pub const UNLOAD: &'static str = "amml_mod_unload";
}
