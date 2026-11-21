/// How much of a stack a drop moves.
///
/// `kind` is `KIND_WHOLE`, `KIND_ONE` or `KIND_COUNT`; `count` is read only for `KIND_COUNT`. The loader gives
/// `INVALID_ARGUMENT` for another kind or a count of 0.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlAmount {
    pub kind: u8,
    pub count: u8,
    pub _reserved: [u8; 2],
}

impl AmmlAmount {
    pub const KIND_WHOLE: u8 = 0;
    pub const KIND_ONE: u8 = 1;
    pub const KIND_COUNT: u8 = 2;

    /// The whole stack.
    pub const WHOLE: Self = Self { kind: Self::KIND_WHOLE, count: 0, _reserved: [0; 2] };
    /// One item.
    pub const ONE: Self = Self { kind: Self::KIND_ONE, count: 0, _reserved: [0; 2] };

    /// `count` items.
    pub const fn count(count: u8) -> Self {
        Self { kind: Self::KIND_COUNT, count, _reserved: [0; 2] }
    }
}

const _: () = assert!(core::mem::size_of::<AmmlAmount>() == 4);
