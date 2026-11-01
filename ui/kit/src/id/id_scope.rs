use std::any::TypeId;
use std::hash::{Hash, Hasher};

/// The type part of an id enum's hash. `Id::unique` hashes the value alone, and a derived `Hash` on an enum hashes
/// only the variant number and the fields, so variant 3 of one id enum would be the same egui id as variant 3 of any
/// other. Each id enum calls `IdScope::hash_type::<Self, _>` first in its `Hash`, so its ids are its own.
pub struct IdScope;

impl IdScope {
    pub fn hash_type<T: 'static, H: Hasher>(state: &mut H) {
        TypeId::of::<T>().hash(state);
    }
}
