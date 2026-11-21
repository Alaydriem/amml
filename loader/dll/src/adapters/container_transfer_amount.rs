use amml_abi::AmmlAmount;

/// `ItemTransferAmount`, the argument of `_handleDropItem` and `_handleDropSelectedItem`: 12 bytes, a qword at +0
/// (the tag and the value) and a dword at +8 (a flag byte and padding). The game reads it through a pointer and
/// copies it. The tag is 0 for a count (`RequestAmount`), 1 for a take type (`TakeType`); a take type of 0 is the
/// whole stack and 2 is one item.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerTransferAmount {
    pub tag: i32,
    pub value: i32,
    pub flag: u8,
    pub padding: [u8; 3],
}

const _: () = assert!(std::mem::size_of::<ContainerTransferAmount>() == 12);

impl ContainerTransferAmount {
    const COUNT: i32 = 0;
    const TAKE: i32 = 1;
    const TAKE_ALL: i32 = 0;
    const TAKE_ONE: i32 = 2;

    /// The game value of `amount`; `None` for a kind the API does not define and for a count of 0.
    pub fn new(amount: AmmlAmount) -> Option<Self> {
        let (tag, value) = match amount.kind {
            AmmlAmount::KIND_WHOLE => (Self::TAKE, Self::TAKE_ALL),
            AmmlAmount::KIND_ONE => (Self::TAKE, Self::TAKE_ONE),
            AmmlAmount::KIND_COUNT if amount.count > 0 => (Self::COUNT, i32::from(amount.count)),
            _ => return None,
        };
        Some(Self { tag, value, flag: 0, padding: [0; 3] })
    }
}
