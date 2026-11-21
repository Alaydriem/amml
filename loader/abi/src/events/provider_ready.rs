use crate::{AmmlEvent, AmmlEventHeader, AmmlStr};

/// Runs when a mod publishes a provider table. `name` is valid only during the listener call; a
/// listener that keeps it copies the bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProviderReady {
    pub header: AmmlEventHeader,
    pub name: AmmlStr,
    pub major: u16,
    pub minor: u16,
}

impl AmmlEvent for ProviderReady {
    const ID: &'static str = "registry.provider_ready";
}

const _: () = assert!(core::mem::size_of::<ProviderReady>() == 40);
