use amml_abi::{AmmlExports, ModDisableFn, ModEnableFn, ModLoadFn, ModUnloadFn};

use crate::{Library, ModLoadError};

/// The four functions a mod DLL exports.
#[derive(Clone, Copy)]
pub struct ModExports {
    pub load: ModLoadFn,
    pub enable: ModEnableFn,
    pub disable: ModDisableFn,
    pub unload: ModUnloadFn,
}

impl ModExports {
    /// Looks up the four exports; a missing one is an error naming it.
    pub fn find(library: &Library) -> Result<Self, ModLoadError> {
        // SAFETY: the exports are declared by the AMML contract with exactly these signatures.
        unsafe {
            Ok(Self {
                load: library.symbol(AmmlExports::LOAD).ok_or(ModLoadError::MissingExport(AmmlExports::LOAD))?,
                enable: library.symbol(AmmlExports::ENABLE).ok_or(ModLoadError::MissingExport(AmmlExports::ENABLE))?,
                disable: library
                    .symbol(AmmlExports::DISABLE)
                    .ok_or(ModLoadError::MissingExport(AmmlExports::DISABLE))?,
                unload: library.symbol(AmmlExports::UNLOAD).ok_or(ModLoadError::MissingExport(AmmlExports::UNLOAD))?,
            })
        }
    }
}
