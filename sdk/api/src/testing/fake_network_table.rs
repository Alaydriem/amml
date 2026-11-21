#![allow(unsafe_code)]

use amml_abi::{AmmlImageView, AmmlNetworkTable, AmmlPlayerListEntry, AmmlStatus, AmmlStr};

use crate::Area;
use crate::testing::{FakeNetwork, FakeState};

/// The functions of the network table of a `FakeGame`. Each checks its output pointers, then the area of its group
/// (`NOT_AVAILABLE` when the fake does not offer it), then asks the `FakeNetwork` of the fake of this thread.
pub(crate) struct FakeNetworkTable;

impl FakeNetworkTable {
    pub(crate) fn table() -> AmmlNetworkTable {
        AmmlNetworkTable {
            size: AmmlNetworkTable::SIZE_V1,
            version: 1,
            player_list_entry: Self::player_list_entry,
            player_list_text: Self::player_list_text,
            player_skin: Self::player_skin,
            skin_image: Self::skin_image,
            skin_animated_count: Self::skin_animated_count,
            skin_animated: Self::skin_animated,
            skin_is_persona: Self::skin_is_persona,
            skin_geometry_name: Self::skin_geometry_name,
            skin_geometry_json: Self::skin_geometry_json,
            skin_is_premium: Self::skin_is_premium,
            skin_resource_patch: Self::skin_resource_patch,
        }
    }

    unsafe extern "C" fn player_list_entry(index: u32, out: *mut AmmlPlayerListEntry) -> AmmlStatus {
        Self::write(out, None, |network| network.entry(index))
    }

    unsafe extern "C" fn player_list_text(index: u32, field: u8, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, None, |network| network.entry_text(index, field))
    }

    unsafe extern "C" fn player_skin(out_uuid: *mut [u8; 16], out_skin: *mut u64) -> AmmlStatus {
        if out_uuid.is_null() || out_skin.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::with(None, |network| network.skin_event()) {
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
    }

    unsafe extern "C" fn skin_image(skin: u64, out: *mut AmmlImageView) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN), |network| network.image(skin))
    }

    unsafe extern "C" fn skin_animated_count(skin: u64, out: *mut u32) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_ANIMATED), |network| network.animated_count(skin))
    }

    unsafe extern "C" fn skin_animated(
        skin: u64,
        index: u32,
        out_kind: *mut u32,
        out: *mut AmmlImageView,
    ) -> AmmlStatus {
        if out_kind.is_null() || out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::with(Some(Area::NETWORK_SKIN_ANIMATED), |network| network.animated(skin, index)) {
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
    }

    unsafe extern "C" fn skin_is_persona(skin: u64, out: *mut u8) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_DETAILS), |network| network.persona(skin).map(u8::from))
    }

    unsafe extern "C" fn skin_geometry_name(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_GEOMETRY), |network| network.geometry_name(skin))
    }

    unsafe extern "C" fn skin_geometry_json(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_GEOMETRY), |network| network.geometry_json(skin))
    }

    unsafe extern "C" fn skin_is_premium(skin: u64, out: *mut u8) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_DETAILS), |network| network.premium(skin).map(u8::from))
    }

    unsafe extern "C" fn skin_resource_patch(skin: u64, out: *mut AmmlStr) -> AmmlStatus {
        Self::write(out, Some(Area::NETWORK_SKIN_DETAILS), |network| network.resource_patch(skin))
    }

    // Runs `read` on the network side of the fake of this thread after the check of `area`.
    fn with<T>(area: Option<Area>, read: impl FnOnce(&FakeNetwork) -> Result<T, AmmlStatus>) -> Result<T, AmmlStatus> {
        let state = FakeState::current().ok_or(AmmlStatus::FAILED)?;
        if let Some(area) = area
            && !state.area_offered(area.id, area.version)
        {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        read(state.network())
    }

    // Checks `out`, then writes what `read` gives to it.
    fn write<T>(
        out: *mut T,
        area: Option<Area>,
        read: impl FnOnce(&FakeNetwork) -> Result<T, AmmlStatus>,
    ) -> AmmlStatus {
        if out.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        match Self::with(area, read) {
            Ok(value) => {
                // SAFETY: `out` is non-null and the caller promises it is writable.
                unsafe { out.write(value) };
                AmmlStatus::OK
            }
            Err(status) => status,
        }
    }
}
