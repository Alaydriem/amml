use amml_runtime::{InjectedOp, ResourceId};

/// One key or button a mod holds through a lease, by the hold id the mod got.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputHold {
    pub hold: ResourceId,
    pub lease: ResourceId,
    /// The down this hold pressed: a `Key` or a `Button`.
    pub op: InjectedOp,
}
