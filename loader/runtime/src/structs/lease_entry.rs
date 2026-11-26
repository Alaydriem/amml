use amml_abi::AmmlLeaseState;

use crate::structs::{LeaseTerms, ResourceId, SuspendCause};

/// One lease the book knows: active, or suspended with its cause.
pub(crate) struct LeaseEntry {
    pub(crate) lease: ResourceId,
    pub(crate) owner: usize,
    pub(crate) owner_id: String,
    pub(crate) terms: LeaseTerms,
    pub(crate) state: AmmlLeaseState,
    /// `Some` exactly when `state` is `SUSPENDED`.
    pub(crate) suspended_by: Option<SuspendCause>,
}
