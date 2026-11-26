use amml_abi::AmmlAmount;

/// How much of a stack a drop moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Amount {
    /// The whole stack.
    Whole,
    /// One item.
    One,
    /// This many items; 0 is refused by the loader.
    Count(u8),
}

impl From<Amount> for AmmlAmount {
    fn from(amount: Amount) -> Self {
        match amount {
            Amount::Whole => Self::WHOLE,
            Amount::One => Self::ONE,
            Amount::Count(count) => Self::count(count),
        }
    }
}
