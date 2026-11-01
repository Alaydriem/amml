pub mod cache_source;
pub mod derive_source;
pub mod levilamina_source;

use amml_data::EntryId;

pub use cache_source::CacheSource;
pub use derive_source::DeriveSource;
pub use levilamina_source::{LeviLaminaSource, SymbolLookup};

use crate::structs::SourceOutcome;

/// One way to find entry values; the sources are a closed set, so this enum dispatches.
pub enum ResolverSource {
    Cache(CacheSource),
    Derive(DeriveSource),
    LeviLamina(LeviLaminaSource),
}

impl ResolverSource {
    /// The value of every id in `wanted` this source can give, with RVAs as values.
    pub fn resolve_all(&self, wanted: &[EntryId]) -> SourceOutcome {
        match self {
            Self::Cache(source) => source.resolve_all(wanted),
            Self::Derive(source) => source.resolve_all(wanted),
            Self::LeviLamina(source) => source.resolve_all(wanted),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Cache(_) => "cache",
            Self::Derive(_) => "derive",
            Self::LeviLamina(_) => "levilamina",
        }
    }
}
