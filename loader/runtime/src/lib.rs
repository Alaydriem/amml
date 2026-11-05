#![forbid(unsafe_code)]

pub mod services;
pub mod structs;

/// Why the loader counts an entry as not for this build; the deriver's `entry_is` judgement.
pub use amml_derive::LayoutRefusal;

pub use services::{
    CacheFile, CacheSource, DeriveSource, Grade, HookChain, HostResolver, LeviLaminaSource, ModRegistry, RecipeDigest,
    ResolverChain, ResolverSource, SymbolLookup,
};
pub use structs::{
    AmmlVersion, ChainChange, ChainError, EntryTable, GradeReport, GradeRow, GradeVerdict, HookEntry, ImageLayout,
    ModRegistryError, ModScan, ModSpec, OriginalValue, OriginalWrite, SkippedMod, SourceNote, SourceOutcome,
};
