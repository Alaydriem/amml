/// How a `read_columns` call went: the columns read, the columns skipped, and the columns that need a retry.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AmmlColumnReadStats {
    pub read: u16,
    pub skipped: u16,
    pub retry: u16,
    pub _reserved: u16,
}

const _: () = assert!(core::mem::size_of::<AmmlColumnReadStats>() == 8);
