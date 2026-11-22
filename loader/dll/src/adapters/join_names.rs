use amml_msvc::MsvcString;

/// The facts of one `onStartJoinGame` call: the flag in `dl` and its three `std::string const&` arguments. The roles come
/// from the `msvc_name` and notes of `ClientInstance.onStartJoinGame`, the earlier dejavu detour and the game image
/// (both builds): r8 is a correlation id (not given to mods), r9 the server name and the argument at entry+28h the world
/// name. The in-game check L-D6 confirms them; the loader logs all three strings once per join.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JoinNames {
    pub local: bool,
    pub correlation_id: String,
    pub server_name: String,
    pub world_name: String,
}

impl JoinNames {
    /// Reads the names of a call whose `rdx` was `flag` (the flag is its low byte) and whose `r8`, `r9` and stack word at
    /// entry+28h were the addresses `correlation`, `server` and `world`. A null address, or a text that is not UTF-8,
    /// gives an empty name.
    ///
    /// # Safety
    /// A non-zero address is that of a live `std::string` for the call.
    pub unsafe fn read(flag: u64, correlation: u64, server: u64, world: u64) -> Self {
        Self {
            local: flag & 0xFF != 0,
            // SAFETY: the caller guarantees each non-zero address is a live `std::string`.
            correlation_id: unsafe { Self::text(correlation) },
            // SAFETY: as above.
            server_name: unsafe { Self::text(server) },
            // SAFETY: as above.
            world_name: unsafe { Self::text(world) },
        }
    }

    /// The text of the `std::string` at `address`; empty for a null address or text that is not UTF-8.
    ///
    /// # Safety
    /// A non-zero `address` is the address of a live `std::string`.
    pub unsafe fn text(address: u64) -> String {
        let Some(address) = usize::try_from(address).ok().filter(|address| *address != 0) else {
            return String::new();
        };
        // SAFETY: the caller guarantees the address is a live `std::string`.
        let text = unsafe { &*std::ptr::with_exposed_provenance::<MsvcString>(address) };
        text.to_str().map(str::to_string).unwrap_or_default()
    }
}
