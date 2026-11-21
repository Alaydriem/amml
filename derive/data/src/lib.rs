#![forbid(unsafe_code)]

pub mod errors;
pub mod services;
pub mod structs;

pub use errors::DataError;
pub use services::{AdapterCatalog, BuildDataStore, BuildIdReader, DependencyOrder, EntryFingerprints, RecipeLoader};
pub use structs::{
    AdapterNeed, AdapterShape, Availability, BuildData, BuildId, Check, Entry, EntryId, EntryKind, Fingerprint,
    LiteralOk, NeedKind, OperandKind, PatternText, Recipe, RecipeSet, Resolved, Step, ValueType,
};
