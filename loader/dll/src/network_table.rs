use amml_abi::{AmmlImageView, AmmlNetworkTable, AmmlPlayerListEntry, AmmlStatus, AmmlStr};

use crate::{Boundary, NetworkAdapter};

const TABLE_VERSION: u32 = 1;

pub(crate) static NETWORK: AmmlNetworkTable = AmmlNetworkTable {
    size: AmmlNetworkTable::SIZE_V1,
    version: TABLE_VERSION,
    player_list_entry: NetworkTable::player_list_entry,
    player_list_text: NetworkTable::player_list_text,
    player_skin: NetworkTable::player_skin,
    skin_image: NetworkTable::skin_image,
    skin_animated_count: NetworkTable::skin_animated_count,
    skin_animated: NetworkTable::skin_animated,
    skin_is_persona: NetworkTable::skin_is_persona,
    skin_geometry_name: NetworkTable::skin_geometry_name,
    skin_geometry_json: NetworkTable::skin_geometry_json,
    skin_is_premium: NetworkTable::skin_is_premium,
    skin_resource_patch: NetworkTable::skin_resource_patch,
};

/// The network table behind `AmmlServices::network`, and its `extern "C"` functions. Each function checks its output
/// pointers (`INVALID_ARGUMENT`), then delegates to `NetworkAdapter` inside the boundary. The adapter answers only on the
/// thread that runs a network event, inside the dispatch of that event (`REFUSED` otherwise); with no network adapter
/// every call gives `NOT_AVAILABLE`.
pub struct NetworkTable;

impl NetworkTable {
    unsafe extern "C" fn player_list_entry(index: u32, out: *mut AmmlPlayerListEntry) -> AmmlStatus {
        Self::write(out, |adapter| adapter.entry(index))
    }

    unsafe extern "C" fn player_list_text(index: u32, field: u8, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, |adapter| adapter.entry_text(index, field))
    }

    unsafe extern "C" fn player_skin(out_uuid: *mut [u8; 16], out_skin: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            if out_uuid.is_null() || out_skin.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match NetworkAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), NetworkAdapter::skin_event) {
                Ok((uuid, skin)) => {
                    // SAFETY: both outputs are non-null and the caller promises they are writable.
                    unsafe {
                        out_uuid.write(uuid);
                        out_skin.write(skin);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn skin_image(skin: u64, out: *mut AmmlImageView) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_image(skin))
    }

    unsafe extern "C" fn skin_animated_count(skin: u64, out: *mut u32) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_animated_count(skin))
    }

    unsafe extern "C" fn skin_animated(
        skin: u64,
        index: u32,
        out_kind: *mut u32,
        out: *mut AmmlImageView,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_kind.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match NetworkAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.skin_animated(skin, index)) {
                Ok((kind, view)) => {
                    // SAFETY: both outputs are non-null and the caller promises they are writable.
                    unsafe {
                        out_kind.write(kind);
                        out.write(view);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn skin_is_persona(skin: u64, out: *mut u8) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_is_persona(skin).map(u8::from))
    }

    unsafe extern "C" fn skin_geometry_name(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_geometry_name(skin))
    }

    unsafe extern "C" fn skin_geometry_json(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_geometry_json(skin))
    }

    unsafe extern "C" fn skin_is_premium(skin: u64, out: *mut u8) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_is_premium(skin).map(u8::from))
    }

    unsafe extern "C" fn skin_resource_patch(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, |adapter| adapter.skin_resource_patch(skin))
    }

    // Checks `out`, runs `read` on the adapter and writes its value to `out`, all inside the boundary.
    fn write<T>(out: *mut T, read: impl FnOnce(&NetworkAdapter) -> Result<T, AmmlStatus>) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match NetworkAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), read) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }
}
