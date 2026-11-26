use std::ffi::c_void;

use amml_abi::{
    AmmlClientFlag, AmmlClientTable, AmmlMenu, AmmlScreen, AmmlStartWorldOutcome, AmmlStatus, AmmlStr, AmmlWorldInfo,
    AmmlWorldText,
};

use crate::{
    Boundary, ClientAdapter, FocusAdapter, HudAdapter, LoaderState, RouterAdapter, SessionActions, SessionAdapter,
    WorldInfoAdapter,
};

const TABLE_VERSION: u32 = 8;

pub(crate) static CLIENT: AmmlClientTable = AmmlClientTable {
    size: AmmlClientTable::SIZE_V8,
    version: TABLE_VERSION,
    current: ClientTable::current,
    local_player: ClientTable::local_player,
    flag: ClientTable::flag,
    top_screen: ClientTable::top_screen,
    set_flag: ClientTable::set_flag,
    open_pause_screen: ClientTable::open_pause_screen,
    leave_world: Some(ClientTable::leave_world),
    can_join: Some(ClientTable::can_join),
    start_local_world: Some(ClientTable::start_local_world),
    join_server: Some(ClientTable::join_server),
    open_menu: Some(ClientTable::open_menu),
    send_command: Some(ClientTable::send_command),
    select_hotbar_slot: Some(ClientTable::select_hotbar_slot),
    note_activity: Some(ClientTable::note_activity),
    world_info: Some(ClientTable::world_info),
    world_text: Some(ClientTable::world_text),
    replay_app_focus_lost: Some(ClientTable::replay_app_focus_lost),
    in_gameplay: Some(ClientTable::in_gameplay),
    join_server_direct: Some(ClientTable::join_server_direct),
    title_screen: Some(ClientTable::title_screen),
    go_to_title: Some(ClientTable::go_to_title),
};

/// The client table behind `AmmlServices::client`, and its `extern "C"` functions. Each function checks
/// its pointers and delegates to `ClientAdapter` inside the boundary, so a panic never crosses it. With no
/// client adapter installed (a host without one) every client call gives `NOT_AVAILABLE`.
pub struct ClientTable;

impl ClientTable {
    /// The primary client while the game runs, on the game thread; null when there is none (before the
    /// first client update, after a world leave). The gate closed gives `NOT_READY`, another thread `REFUSED`.
    unsafe extern "C" fn current(out: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let gate = state.events().gate();
            if let Err(status) = gate.require_open() {
                return status;
            }
            if !gate.on_game_thread() {
                return AmmlStatus::REFUSED;
            }
            let client = SessionAdapter::primary_client().unwrap_or(0);
            // SAFETY: `out` is non-null and the caller promises it is writable.
            unsafe { out.write(std::ptr::with_exposed_provenance(client)) };
            AmmlStatus::OK
        })
    }

    unsafe extern "C" fn local_player(client: *const c_void, out: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.local_player(client)) {
                Ok(player) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(std::ptr::with_exposed_provenance(player)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn flag(client: *const c_void, flag: AmmlClientFlag, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.flag(client, flag)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(value)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn top_screen(client: *const c_void, out: *mut AmmlScreen) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.top_screen(client)) {
                Ok(screen) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(screen) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn set_flag(
        module: AmmlStr,
        client: *const c_void,
        flag: AmmlClientFlag,
        value: u8,
    ) -> AmmlStatus {
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
            Self::status(Self::adapter().and_then(|adapter| adapter.set_flag(owner, client, flag, value != 0)))
        })
    }

    unsafe extern "C" fn open_pause_screen(client: *const c_void) -> AmmlStatus {
        Boundary::guard(|| Self::status(Self::adapter().and_then(|adapter| adapter.open_pause_screen(client))))
    }

    unsafe extern "C" fn leave_world(client: *const c_void) -> AmmlStatus {
        Boundary::guard(|| Self::status(Self::actions().and_then(|actions| actions.leave_world(client))))
    }

    unsafe extern "C" fn can_join(client: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::actions().and_then(|actions| actions.can_join(client)) {
                Ok(ready) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(ready)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn start_local_world(
        client: *const c_void,
        level_id: AmmlStr,
        out: *mut AmmlStartWorldOutcome,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `level_id` is valid for the call.
            let level_id = match unsafe { level_id.as_str() } {
                Ok(text) => text,
                Err(status) => return status,
            };
            match Self::actions().and_then(|actions| actions.start_local_world(client, level_id)) {
                Ok(outcome) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(outcome) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn join_server(
        module: AmmlStr,
        client: *const c_void,
        host: AmmlStr,
        port: u16,
        name: AmmlStr,
        out: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises the three views are valid for the call.
            let texts = unsafe { (module.as_str(), host.as_str(), name.as_str()) };
            let (module, host, name) = match texts {
                (Ok(module), Ok(host), Ok(name)) => (module, host, name),
                _ => return AmmlStatus::INVALID_ARGUMENT,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match Self::actions().and_then(|actions| actions.join_server(owner, client, host, port, name)) {
                Ok(id) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn join_server_direct(
        module: AmmlStr,
        client: *const c_void,
        host: AmmlStr,
        port: u16,
        name: AmmlStr,
        out: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises the three views are valid for the call.
            let texts = unsafe { (module.as_str(), host.as_str(), name.as_str()) };
            let (module, host, name) = match texts {
                (Ok(module), Ok(host), Ok(name)) => (module, host, name),
                _ => return AmmlStatus::INVALID_ARGUMENT,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match Self::actions().and_then(|actions| actions.join_server_direct(owner, client, host, port, name)) {
                Ok(id) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn title_screen(client: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match RouterAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.on_title_screen(client)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(value)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn go_to_title(client: *const c_void) -> AmmlStatus {
        Boundary::guard(|| Self::status(Self::actions().and_then(|actions| actions.go_to_title(client))))
    }

    unsafe extern "C" fn open_menu(client: *const c_void, menu: AmmlMenu) -> AmmlStatus {
        Boundary::guard(|| Self::status(Self::actions().and_then(|actions| actions.open_menu(client, menu))))
    }

    unsafe extern "C" fn send_command(client: *const c_void, text: AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            // SAFETY: the caller promises `text` is valid for the call.
            let text = match unsafe { text.as_str() } {
                Ok(text) => text,
                Err(status) => return status,
            };
            Self::status(Self::actions().and_then(|actions| actions.send_command(client, text)))
        })
    }

    unsafe extern "C" fn note_activity(client: *const c_void) -> AmmlStatus {
        Boundary::guard(|| Self::status(Self::adapter().and_then(|adapter| adapter.note_activity(client))))
    }

    unsafe extern "C" fn in_gameplay(client: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::adapter().and_then(|adapter| adapter.in_gameplay(client)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(value)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn replay_app_focus_lost(module: AmmlStr, client: *const c_void) -> AmmlStatus {
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
            let focus = FocusAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            Self::status(focus.and_then(|focus| focus.replay(owner, client)))
        })
    }

    unsafe extern "C" fn world_info(client: *const c_void, out: *mut AmmlWorldInfo) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::world().and_then(|world| world.world_info(client)) {
                Ok(info) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(info) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn world_text(
        client: *const c_void,
        field: AmmlWorldText,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match Self::world().and_then(|world| world.world_text(client, field)) {
                Ok(text) => {
                    let bytes = text.as_bytes();
                    let copied = bytes.len().min(cap);
                    if copied > 0 {
                        // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
                        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
                    }
                    // SAFETY: `out_len` is non-null and the caller promises it is writable.
                    unsafe { out_len.write(bytes.len()) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn select_hotbar_slot(client: *const c_void, slot: u8) -> AmmlStatus {
        Boundary::guard(|| {
            let adapter = HudAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            Self::status(adapter.and_then(|adapter| adapter.select_hotbar_slot(client, slot)))
        })
    }

    fn actions() -> Result<&'static SessionActions, AmmlStatus> {
        SessionActions::get().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    fn world() -> Result<&'static WorldInfoAdapter, AmmlStatus> {
        WorldInfoAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    fn adapter() -> Result<&'static ClientAdapter, AmmlStatus> {
        ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    fn status(result: Result<(), AmmlStatus>) -> AmmlStatus {
        match result {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        }
    }
}
