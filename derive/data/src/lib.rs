#![forbid(unsafe_code)]

pub mod errors;
pub mod services;
pub mod structs;

pub use errors::DataError;
pub use services::{BuildDataStore, BuildIdReader, DependencyOrder, EntryFingerprints, RecipeLoader};
pub use structs::{
    BuildData, BuildId, Check, Entry, EntryId, EntryKind, Fingerprint, LiteralOk, OperandKind, PatternText, Recipe,
    RecipeSet, Resolved, Step, ValueType,
};
