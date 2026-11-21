use amml_abi::{AmmlLightSource, AmmlLightingTable, AmmlStatus, AmmlStr};

use crate::{Boundary, LightingAdapter, LoaderState};

const TABLE_VERSION: u32 = 1;

pub(crate) static LIGHTING: AmmlLightingTable = AmmlLightingTable {
    size: AmmlLightingTable::SIZE_V1,
    version: TABLE_VERSION,
    set_light_source: LightingTable::set_light_source,
};

/// The lighting table behind `AmmlServices::lighting`, and its `extern "C"` function, which checks its
/// arguments, the mod, the gate, the thread and the area, then delegates to `LightingAdapter` inside the
/// boundary.
pub struct LightingTable;

impl LightingTable {
    unsafe extern "C" fn set_light_source(module: AmmlStr, source: *const AmmlLightSource) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let source = if source.is_null() {
                None
            } else {
                // SAFETY: a non-null `source` holds at least its leading `size` field, as the caller promises.
                let size = unsafe { source.cast::<u32>().read() };
                if size < AmmlLightSource::SIZE_V1 {
                    return AmmlStatus::INVALID_ARGUMENT;
                }
                // SAFETY: `size` covers every v1 field.
                Some(unsafe { source.read() })
            };
            if !state.areas().is_available(LightingAdapter::TERRAIN, TABLE_VERSION) {
                return AmmlStatus::NOT_AVAILABLE;
            }
            let gate = state.events().gate();
            if let Err(status) = gate.require_open() {
                return status;
            }
            if !gate.on_game_thread() {
                return AmmlStatus::REFUSED;
            }
            let result =
                LightingAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.set(owner, source.as_ref()));
            match result {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }
}
