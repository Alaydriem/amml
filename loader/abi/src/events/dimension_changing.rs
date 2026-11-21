use crate::{AmmlEvent, AmmlEventHeader};

/// The local player's dimension change has started and is not complete.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DimensionChanging {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for DimensionChanging {
    const ID: &'static str = "world.dimension_changing";
}

const _: () = assert!(core::mem::size_of::<DimensionChanging>() == 16);
