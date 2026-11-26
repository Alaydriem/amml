use amml_abi::{AmmlStateValue, AmmlStr};

/// The value of one block state for `Blocks::from_state`: an int (such as `weirdo_direction` 2), a bool (such as
/// `upside_down_bit`) or a text (such as `facing_direction` in some blocks, `wood_type` in others).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateValue<'a> {
    Int(i32),
    Bool(bool),
    Text(&'a str),
}

impl StateValue<'_> {
    /// The value as the loader takes it; the text view lives as long as this value.
    pub(crate) fn raw(&self) -> AmmlStateValue {
        let (kind, int, text) = match self {
            Self::Int(int) => (AmmlStateValue::INT, *int, AmmlStr::EMPTY),
            Self::Bool(flag) => (AmmlStateValue::BOOL, i32::from(*flag), AmmlStr::EMPTY),
            Self::Text(text) => (AmmlStateValue::TEXT, 0, AmmlStr::from_str(text)),
        };
        AmmlStateValue { kind, _reserved: [0; 3], int, text }
    }
}
