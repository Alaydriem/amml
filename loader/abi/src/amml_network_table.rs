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

/// The network functions of `AmmlServices`: the player list and skin reads of the `PlayerListUpdate` and
/// `PlayerSkinChanged` events (areas `network.skin`, `network.skin_animated`, `network.skin_geometry` and
/// `network.skin_details`). Every function gives `REFUSED` when no event of the calling thread runs. The table only
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
}

impl AmmlNetworkTable {
    /// `size` of a v1 network table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 96;

    /// A copy of the table at `table`; `None` for a null table or one shorter than v1.
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
        // SAFETY: `size` covers every v1 field, all function pointers written from a valid table.
        Some(unsafe { table.read_unaligned() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlNetworkTable>() == AmmlNetworkTable::SIZE_V1 as usize);
