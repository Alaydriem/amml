use amml_data::EntryId;

/// Something a source met that the host should log; none of these stops the chain.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SourceNote {
    #[error("no cache file for this build")]
    CacheMissing,
    #[error("cache file unreadable: {0}")]
    CacheUnreadable(String),
    #[error("cache file is for build {found}, this build is {expected}")]
    CacheForeignBuild { found: String, expected: String },
    #[error("shipped data file unreadable: {0}")]
    ShippedUnreadable(String),
    #[error("shipped data file is for build {found}, this build is {expected}")]
    ShippedForeignBuild { found: String, expected: String },
    #[error("cached or shipped entry {0} has another kind than the recipe set and was dropped")]
    CacheKindMismatch(EntryId),
    #[error("recipes unreadable: {0}")]
    RecipesUnreadable(String),
    #[error("exe image unreadable: {0}")]
    ImageUnreadable(String),
    #[error("derive failed: {0}")]
    DeriveFailed(String),
    #[error("cache file not written: {0}")]
    CacheWriteFailed(String),
    #[error("derive ran: {hits} reused, {derived} derived in {elapsed_ms} ms")]
    DeriveRan { hits: usize, derived: usize, elapsed_ms: u64 },
    #[error("LeviLamina returned no address for {0}")]
    LookupMissing(EntryId),
    #[error(
        "LeviLamina returned {address:#x} for {id}, outside the game image at {module_base:#x} (size {size_of_image:#x})"
    )]
    LookupOutsideImage { id: EntryId, address: u64, module_base: u64, size_of_image: u64 },
    #[error("LeviLamina returned rva {rva:#x} for function {id}, which is not in an executable section")]
    LookupNotExecutable { id: EntryId, rva: u64 },
    #[error(
        "LeviLamina gave rva {levilamina:#x} for {id}, the cache or derive gave {derived:#x}; the entry is missing"
    )]
    SourcesDisagree { id: EntryId, levilamina: u64, derived: u64 },
}
