use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlClientFlag, AmmlScreen, AmmlStartWorldOutcome, AmmlStr};

use crate::client_tables::ClientTables;
use crate::{ApiError, Hud, JoinId, JoinTarget, LocalPlayer, Menu, Screen, StartWorldOutcome};

/// The game client of an event (`EventCtx::client`) or of a disable (`DisableCtx::client`). It lives no
/// longer than that call and is used on the game thread only.
///
/// Every call goes through the loader, which checks that the game runs, that the call runs on the game
/// thread and that the handle is the current primary client; a read it refuses gives `None`. A handle
/// stays valid while it is the primary client. The calls are not bound to a world: they work on the
/// title screen too.
pub struct Client<'f> {
    tables: ClientTables,
    handle: *const c_void,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> Client<'f> {
    /// The longest command text `send_command` takes, in bytes.
    pub const MAX_COMMAND: usize = 4096;

    pub(crate) fn new(tables: ClientTables, handle: *const c_void, module: &'static str) -> Self {
        Self { tables, handle, module, _event: PhantomData }
    }

    /// The local player; `None` while no world is joined or when the loader refuses the read.
    pub fn local_player(&self) -> Option<LocalPlayer<'f>> {
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.tables.client.local_player)(self.handle, &mut out) };
        (status.is_ok() && !out.is_null())
            .then(|| LocalPlayer::new(out, self.tables.actor, self.tables.inventory, self.module))
    }

    pub fn world_active(&self) -> Option<bool> {
        self.flag(AmmlClientFlag::WORLD_ACTIVE)
    }

    pub fn showing_menu(&self) -> Option<bool> {
        self.flag(AmmlClientFlag::SHOWING_MENU)
    }

    /// The screen on top; a screen this crate does not know is `Screen::Other`.
    pub fn top_screen(&self) -> Option<Screen> {
        let mut out = AmmlScreen::OTHER;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.tables.client.top_screen)(self.handle, &mut out) };
        status.is_ok().then(|| Screen::from(out))
    }

    /// Whether the game holds the mouse, as last set by the game or the loader. `None` until the loader
    /// saw the game grab or release it once.
    pub fn mouse_captured(&self) -> Option<bool> {
        self.flag(AmmlClientFlag::MOUSE_CAPTURED)
    }

    pub fn hide_hud(&self) -> Option<bool> {
        self.flag(AmmlClientFlag::HIDE_HUD)
    }

    /// Grabs or releases the mouse. The loader restores the value from before the first set of any mod
    /// when the last mod that set it is disabled.
    pub fn set_mouse_captured(&self, captured: bool) -> Result<(), ApiError> {
        self.set_flag(AmmlClientFlag::MOUSE_CAPTURED, captured)
    }

    /// Hides or shows the HUD, under the restore rule of `set_mouse_captured`.
    pub fn set_hide_hud(&self, hidden: bool) -> Result<(), ApiError> {
        self.set_flag(AmmlClientFlag::HIDE_HUD, hidden)
    }

    /// Opens the pause screen through the game's focus-lost path, then releases every injected input with
    /// the reason `FocusLost`. The game opens the screen there only when its option
    /// `pause_menu_on_focus_lost` is on: with it off, the result is `Refused` and nothing happens.
    /// `NotAvailable` when the loader cannot find or read that option on this build.
    pub fn open_pause_screen(&self) -> Result<(), ApiError> {
        // SAFETY: the loader checks the handle before it calls into the game.
        #[allow(unsafe_code)]
        let status = unsafe { (self.tables.client.open_pause_screen)(self.handle) };
        ApiError::check(status)
    }

    /// Asks the game to leave the joined world; `WorldLeft` follows. `Refused` while no world is joined;
    /// `NotAvailable` without area `client.leave_world`.
    pub fn leave_world(&self) -> Result<(), ApiError> {
        let leave = self.tables.client.leave_world.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the handle before it calls into the game.
        #[allow(unsafe_code)]
        let status = unsafe { leave(self.handle) };
        ApiError::check(status)
    }

    /// Whether the game's primary client is ready for a join (`join_server`, `start_local_world`). `None` without
    /// area `client.can_join` or when the loader refuses the read.
    pub fn can_join(&self) -> Option<bool> {
        let can_join = self.tables.client.can_join?;
        let mut out = 0u8;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { can_join(self.handle, &mut out) };
        status.is_ok().then_some(out != 0)
    }

    /// Starts the local world `level_id` (the folder name under `minecraftWorlds`), as the Play screen does. The game
    /// answers at once; its "not ready" answer is `NotReady` (ask again on a later frame). `Refused` in a world;
    /// `NotAvailable` without area `client.start_local_world`.
    pub fn start_local_world(&self, level_id: &str) -> Result<StartWorldOutcome, ApiError> {
        let start = self.tables.client.start_local_world.ok_or(ApiError::NotAvailable)?;
        let mut out = AmmlStartWorldOutcome::OTHER_REFUSAL;
        // SAFETY: the id lives for the call, the loader checks the handle, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { start(self.handle, AmmlStr::from_str(level_id), &mut out) };
        ApiError::check(status)?;
        Ok(StartWorldOutcome::from(out))
    }

    /// Joins the server of `target`, as the game's own server join does: the game resolves the host and tests the
    /// connection. The answer comes as `JoinServerResult` with the returned id, at a later frame, to this mod only.
    /// `Refused` in a world or while a join waits for its answer; `NotAvailable` without area `client.join_server`.
    pub fn join_server(&self, target: JoinTarget<'_>) -> Result<JoinId, ApiError> {
        let join = self.tables.client.join_server.ok_or(ApiError::NotAvailable)?;
        let mut out = 0u64;
        let name = target.name.unwrap_or("");
        // SAFETY: the strings live for the call, the loader checks the handle, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe {
            join(
                AmmlStr::from_str(self.module),
                self.handle,
                AmmlStr::from_str(target.host),
                target.port,
                AmmlStr::from_str(name),
                &mut out,
            )
        };
        ApiError::check(status)?;
        Ok(JoinId(out))
    }

    /// Opens a menu page. `Refused` in a world or when the game refuses the page; `NotAvailable` without area
    /// `client.open_menu`.
    pub fn open_menu(&self, menu: Menu) -> Result<(), ApiError> {
        let open = self.tables.client.open_menu.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the handle before it calls into the game.
        #[allow(unsafe_code)]
        let status = unsafe { open(self.handle, menu.to_abi()) };
        ApiError::check(status)
    }

    /// Sends `text` as a chat command, as typed in chat with its slash (`/say hello`). At most
    /// `Client::MAX_COMMAND` bytes, `Refused` above and while no world is joined; `NotAvailable` without area
    /// `client.send_command`.
    pub fn send_command(&self, text: &str) -> Result<(), ApiError> {
        let send = self.tables.client.send_command.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the text lives for the call and the loader checks the handle.
        #[allow(unsafe_code)]
        let status = unsafe { send(self.handle, AmmlStr::from_str(text)) };
        ApiError::check(status)
    }

    /// Tells the game that the user did something: runs the game's any-input handler once, as the game does for
    /// each input event. This resets the game's idle timer (the idle screen and the low update rate after 15
    /// minutes with no input). It makes no key, button or pointer event, and no `KeyInput` or `MouseInput`
    /// listener sees it. `Refused` while the game is not ready for it (no input handler yet); `NotAvailable` on a
    /// build without area `client.activity` or a loader before client table v3.
    pub fn note_activity(&self) -> Result<(), ApiError> {
        let note = self.tables.client.note_activity.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the handle before it calls into the game.
        #[allow(unsafe_code)]
        let status = unsafe { note(self.handle) };
        ApiError::check(status)
    }

    /// The game HUD of this client (`Hud::select_hotbar_slot`).
    pub fn hud(&self) -> Hud<'f> {
        Hud::new(self.tables.client, self.handle)
    }

    fn flag(&self, flag: AmmlClientFlag) -> Option<bool> {
        let mut out = 0u8;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.tables.client.flag)(self.handle, flag, &mut out) };
        status.is_ok().then_some(out != 0)
    }

    fn set_flag(&self, flag: AmmlClientFlag, value: bool) -> Result<(), ApiError> {
        // SAFETY: the module id lives for the call and the loader checks the handle before it writes.
        #[allow(unsafe_code)]
        let status = unsafe {
            (self.tables.client.set_flag)(AmmlStr::from_str(self.module), self.handle, flag, u8::from(value))
        };
        ApiError::check(status)
    }
}
