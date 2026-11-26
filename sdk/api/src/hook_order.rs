use amml_abi::AmmlBand;

/// Where a listener runs among the listeners of one event. Lower bands run first; listeners of one
/// band run in the order they were added.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HookOrder(pub AmmlBand);

impl HookOrder {
    pub const FIRST: Self = Self(AmmlBand::FIRST);
    pub const WRITE: Self = Self(AmmlBand::WRITE);
    pub const NORMAL: Self = Self(AmmlBand::NORMAL);
    pub const READ_AFTER_WRITERS: Self = Self(AmmlBand::READ_AFTER_WRITERS);
    pub const LAST: Self = Self(AmmlBand::LAST);
}
