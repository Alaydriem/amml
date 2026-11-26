/// How a `WorldKindsInBoxFn` call went. `kind` is `KINDS` (the first `count` entries of `ids` are the distinct block
/// type ids of the box, in the order the scan met them), `NOT_LOADED` (a chunk of the box is not loaded, or a block could
/// not be read: nothing was listed, `count` is 0) or `TOO_MANY` (the box holds more kinds than `cap`: nothing was listed,
/// `count` is 0). `cells` is the number of cells the scan read and `elapsed_us` the microseconds it took (saturated at
/// `u32::MAX`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlKindsResult {
    pub kind: u8,
    pub _reserved: [u8; 3],
    pub count: u32,
    pub cells: u32,
    pub elapsed_us: u32,
}

impl AmmlKindsResult {
    /// The box was scanned and `count` kinds were written.
    pub const KINDS: u8 = 1;
    /// A chunk of the box is not loaded, or a block could not be read.
    pub const NOT_LOADED: u8 = 2;
    /// The box holds more kinds than `cap`.
    pub const TOO_MANY: u8 = 3;
}

const _: () = assert!(core::mem::size_of::<AmmlKindsResult>() == 16);
