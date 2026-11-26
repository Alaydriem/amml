use std::mem::offset_of;

/// Layout mirror of the head of the game's `PlayerJoinWorldContext`, the by-value argument of
/// `ClientInstance::startExternalNetworkWorld` and the last argument of `MinecraftGame::joinMultiplayerWithAddress` (the
/// game passes a pointer to it), up to and with the server transfer flag. Only this head holds on both builds (the
/// string at 0, the bytes at 20h..23h); the whole object is 0B0h bytes on 26.63 and 160h bytes on 70.26 (an optional
/// block engaged at +148h and a shared state at +150h/+158h that the game's default constructor allocates), so the
/// loader never builds one: the game builds it (`ClientInstance.connectByAddress`).
#[repr(C)]
pub struct PlayerJoinWorldContextLayout {
    _text: [u8; 0x20],          // 0x00 a std::string the callee moves
    _leader: u8,                // 0x20
    _destination: u8,           // 0x21
    pub is_server_transfer: u8, // 0x22
}

const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, is_server_transfer) == 0x22);

impl PlayerJoinWorldContextLayout {
    /// The server transfer flag of the context at `context`; false for a null address.
    ///
    /// # Safety
    /// A non-zero `context` is the address of a live `PlayerJoinWorldContext`, readable up to its transfer flag.
    pub unsafe fn is_server_transfer(context: usize) -> bool {
        if context == 0 {
            return false;
        }
        let address = context.wrapping_add(offset_of!(PlayerJoinWorldContextLayout, is_server_transfer));
        // SAFETY: the caller guarantees the context is live and readable up to the flag; the read is one byte.
        unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() != 0 }
    }
}
