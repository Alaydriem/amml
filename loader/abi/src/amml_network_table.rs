use crate::{AmmlImageView, AmmlPlayerListEntry, AmmlStatus, AmmlStr};

/// `field` of `player_list_text`: the player name.
pub const NETWORK_TEXT_NAME: u8 = 1;
/// `field` of `player_list_text`: the Xbox user id.
pub const NETWORK_TEXT_XUID: u8 = 2;

/// Writes entry `index` (0 up to `count` of the running `PlayerListUpdate`) in `out`. `INVALID_ARGUMENT` for an index
/// past the end, `REFUSED` outside the event.
pub type NetworkPlayerListEntryFn = unsafe extern "C" fn(index: u32, out: *mut AmmlPlayerListEntry) -> AmmlStatus;
/// Writes a view of the text `field` (`NETWORK_TEXT_NAME` or `NETWORK_TEXT_XUID`) of entry `index` in `out`. The view
/// is valid until the event ends. `NOT_AVAILABLE` for a Remove entry or a text that is not UTF-8, `INVALID_ARGUMENT`
/// for an unknown field or an index past the end, `REFUSED` outside the event.
pub type NetworkPlayerListTextFn = unsafe extern "C" fn(index: u32, field: u8, out: *mut AmmlStr) -> AmmlStatus;
/// Writes the uuid of the player of the running `PlayerSkinChanged` in `out_uuid` and its skin handle in `out_skin`
/// (0 when the packet holds no skin). `REFUSED` outside the event.
pub type NetworkPlayerSkinFn = unsafe extern "C" fn(out_uuid: *mut [u8; 16], out_skin: *mut u64) -> AmmlStatus;
/// Writes a view of the main image of `skin` in `out`. `NOT_AVAILABLE` for a blob whose size is not
/// `width * height * 4`, `REFUSED` for a handle that no running event gave.
pub type NetworkSkinImageFn = unsafe extern "C" fn(skin: u64, out: *mut AmmlImageView) -> AmmlStatus;
/// Writes the number of animated images of `skin` in `out`.
pub type NetworkSkinAnimatedCountFn = unsafe extern "C" fn(skin: u64, out: *mut u32) -> AmmlStatus;
/// Writes the kind (the game's `AnimatedTextureType`: 1 face, 2 body 32x32, 3 body 128x128) and a view of the image of
/// animated image `index` of `skin`. `INVALID_ARGUMENT` for an index past the end, `NOT_AVAILABLE` for a blob whose size
/// is not `width * height * 4`.
pub type NetworkSkinAnimatedFn =
    unsafe extern "C" fn(skin: u64, index: u32, out_kind: *mut u32, out: *mut AmmlImageView) -> AmmlStatus;
/// Writes 1 in `out` when `skin` is a persona skin, else 0.
pub type NetworkSkinFlagFn = unsafe extern "C" fn(skin: u64, out: *mut u8) -> AmmlStatus;
/// Writes a view of a text of `skin` in `out`, valid until the event ends. `NOT_AVAILABLE` for a text that is not UTF-8
/// or, for the geometry JSON, one above 1 MB.
pub type NetworkSkinTextFn = unsafe extern "C" fn(skin: u64, out: *mut AmmlStr) -> AmmlStatus;

/// Writes the network protocol number of the game build in `out` (the derived `SharedConstants.NetworkProtocolVersion`).
/// Any thread. `NOT_AVAILABLE` when the area `network.protocol` is not offered. Area `network.protocol`.
pub type NetworkProtocolVersionFn = unsafe extern "C" fn(out: *mut u32) -> AmmlStatus;
/// Sets the wish of `module` for the `enabled` byte of the `ClientCacheStatusPacket` the client sends: 0 asks that the
/// client says it has no blob cache, any other value withdraws the wish. While any mod holds a 0 wish, the loader writes
/// 0 into every such packet before the game sends it. Game thread only (`REFUSED` elsewhere); `NOT_READY` before the
/// first frame; `NOT_AVAILABLE` when the area is not offered. A disabled mod loses its wish. Area `network.chunk_cache`.
pub type NetworkSetClientChunkCacheFn = unsafe extern "C" fn(module: AmmlStr, enabled: u8) -> AmmlStatus;

/// The network functions of `AmmlServices`: the player list and skin reads of the `PlayerListUpdate` and
/// `PlayerSkinChanged` events (areas `network.skin`, `network.skin_animated`, `network.skin_geometry` and
/// `network.skin_details`). The player list and skin functions give `REFUSED` when no event of the calling thread runs;
/// `protocol_version` answers on any thread and `set_client_chunk_cache` on the game thread (`REFUSED` elsewhere). The table only
/// grows: a function added later is appended, `size` and `version` grow, and a reader calls a function only when `size`
/// covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlNetworkTable {
    pub size: u32,
    pub version: u32,
    pub player_list_entry: NetworkPlayerListEntryFn,
    pub player_list_text: NetworkPlayerListTextFn,
    pub player_skin: NetworkPlayerSkinFn,
    pub skin_image: NetworkSkinImageFn,
    pub skin_animated_count: NetworkSkinAnimatedCountFn,
    pub skin_animated: NetworkSkinAnimatedFn,
    pub skin_is_persona: NetworkSkinFlagFn,
    pub skin_geometry_name: NetworkSkinTextFn,
    pub skin_geometry_json: NetworkSkinTextFn,
    pub skin_is_premium: NetworkSkinFlagFn,
    pub skin_resource_patch: NetworkSkinTextFn,
    pub protocol_version: Option<NetworkProtocolVersionFn>,
    pub set_client_chunk_cache: Option<NetworkSetClientChunkCacheFn>,
}

impl AmmlNetworkTable {
    /// `size` of a v1 network table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 96;
    /// `size` of a v2 network table: v1 plus `protocol_version` and `set_client_chunk_cache`.
    pub const SIZE_V2: u32 = 112;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null table, or
    /// one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlNetworkTable) -> Option<AmmlNetworkTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlNetworkTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlNetworkTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole struct
        // and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an `Option`
        // of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlNetworkTable>() == AmmlNetworkTable::SIZE_V2 as usize);
