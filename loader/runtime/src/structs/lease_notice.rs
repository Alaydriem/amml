use crate::structs::{NoticeKind, ResourceId};

/// One change to a lease that its owner must be told about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaseNotice {
    pub owner: usize,
    pub lease: ResourceId,
    pub kind: NoticeKind,
}
