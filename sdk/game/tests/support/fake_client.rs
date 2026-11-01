use std::ffi::c_void;

use amml_game::{EntryId, GameEntries};
use amml_msvc::RustObject;

use super::client_state::ClientState;
use super::client_vtable::ClientVtable;
use super::{FakeEntries, FakeOptions};

static CLIENT_VTABLE: ClientVtable = ClientVtable {
    reserved: 0,
    get_local_player: FakeClient::get_local_player,
    is_world_active: FakeClient::is_world_active,
    is_showing_menu: FakeClient::is_showing_menu,
    get_mouse_grabbed: FakeClient::get_mouse_grabbed,
    get_options: FakeClient::get_options,
};

/// A fake `ClientInstance`: the menu is showing, the mouse is not grabbed.
pub struct FakeClient {
    object: RustObject<ClientVtable, ClientState>,
}

impl FakeClient {
    pub const LOCAL_PLAYER_SLOT: u64 = 1;
    pub const WORLD_ACTIVE_SLOT: u64 = 2;
    pub const SHOWING_MENU_SLOT: u64 = 3;
    pub const MOUSE_GRABBED_SLOT: u64 = 4;
    pub const OPTIONS_SLOT: u64 = 5;

    pub fn new(player: *mut c_void, options: *mut c_void, world_active: bool) -> Self {
        Self { object: RustObject::new(&CLIENT_VTABLE, ClientState { player, options, world_active }) }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut self.object).cast()
    }

    /// Entries for every client and options slot, without the one named by `without`.
    pub fn entries(without: Option<EntryId>) -> GameEntries {
        let values = [
            (EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER, Self::LOCAL_PLAYER_SLOT),
            (EntryId::CLIENT_INSTANCE_IS_WORLD_ACTIVE, Self::WORLD_ACTIVE_SLOT),
            (EntryId::CLIENT_INSTANCE_IS_SHOWING_MENU, Self::SHOWING_MENU_SLOT),
            (EntryId::CLIENT_INSTANCE_GET_MOUSE_GRABBED, Self::MOUSE_GRABBED_SLOT),
            (EntryId::CLIENT_INSTANCE_GET_OPTIONS, Self::OPTIONS_SLOT),
            (EntryId::I_OPTION_REGISTRY_GET_HIDE_HUD, FakeOptions::HIDE_HUD_SLOT),
        ];
        let kept: Vec<_> = values.into_iter().filter(|(id, _)| Some(*id) != without).collect();
        FakeEntries::build(&kept)
    }

    fn state<'a>(this: *mut c_void) -> &'a ClientState {
        // SAFETY: `this` is the RustObject made by `FakeClient::new`.
        unsafe { &*RustObject::<ClientVtable, ClientState>::state(this.cast()) }
    }

    extern "C" fn get_local_player(this: *mut c_void) -> *mut c_void {
        Self::state(this).player
    }

    extern "C" fn is_world_active(this: *mut c_void) -> u8 {
        u8::from(Self::state(this).world_active)
    }

    extern "C" fn is_showing_menu(_: *mut c_void) -> u8 {
        1
    }

    extern "C" fn get_mouse_grabbed(_: *mut c_void) -> u8 {
        0
    }

    extern "C" fn get_options(this: *mut c_void) -> *mut c_void {
        Self::state(this).options
    }
}
