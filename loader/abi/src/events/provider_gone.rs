use crate::{AmmlEvent, AmmlEventHeader, AmmlStr};

/// Runs before a provider table is dropped. `name` is valid only during the listener call; a
/// listener that keeps it copies the bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProviderGone {
    pub header: AmmlEventHeader,
    pub name: AmmlStr,
    pub major: u16,
}

impl AmmlEvent for ProviderGone {
    const ID: &'static str = "registry.provider_gone";
}

const _: () = assert!(core::mem::size_of::<ProviderGone>() == 40);
