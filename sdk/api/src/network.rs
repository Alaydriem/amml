use amml_abi::{AmmlNetworkTable, AmmlStr};

use crate::ApiError;

/// The game's network settings (`Ctx::network`): the protocol number of the build and the client's chunk cache.
///
/// `protocol_version` needs `Area::NETWORK_PROTOCOL` and works on any thread. `set_client_chunk_cache` needs
/// `Area::NETWORK_CHUNK_CACHE` and runs on the game thread only. Without a network table of version 2 (a loader before
/// it) the first gives `None` and the second `NotAvailable`.
pub struct Network {
    table: Option<AmmlNetworkTable>,
    module: &'static str,
}

impl Network {
    pub(crate) fn new(table: Option<AmmlNetworkTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// The network protocol number of the game build (for example 2225 on 26.63), the one the client sends in its first
    /// packet. `None` without the area.
    #[allow(unsafe_code)]
    pub fn protocol_version(&self) -> Option<u32> {
        let protocol_version = self.table.and_then(|table| table.protocol_version)?;
        let mut out = 0u32;
        // SAFETY: the table function is the loader's; `out` is a live local.
        let status = unsafe { protocol_version(&raw mut out) };
        status.is_ok().then_some(out)
    }

    /// Sets this mod's wish for the client's chunk (blob) cache. `false` asks that the client tells the server it has no
    /// cache: while any mod holds that wish, the loader writes `false` into every `ClientCacheStatusPacket` the client
    /// sends, before the game serializes it. `true` is the game's own behavior and does not override another mod's
    /// `false`. Calling again replaces the wish; this mod's disable ends it. The change applies to the next status packet
    /// the client sends; one already sent stays as it was. `Err(Refused)` off the game thread;
    /// `Err(NotReady)` before the first frame; `Err(NotAvailable)` without the area.
    #[allow(unsafe_code)]
    pub fn set_client_chunk_cache(&self, enabled: bool) -> Result<(), ApiError> {
        let Some(set) = self.table.and_then(|table| table.set_client_chunk_cache) else {
            return Err(ApiError::NotAvailable);
        };
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let status = unsafe { set(AmmlStr::from_str(self.module), u8::from(enabled)) };
        ApiError::check(status)
    }
}
