use std::ffi::c_void;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use amml_abi::AmmlStatus;
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{ClientAdapter, HudDetours, LoaderState, ScreenControllerDetours};

static ADAPTER: OnceLock<HudAdapter> = OnceLock::new();

/// `void HudScreenController::_handleSlotSelection(int slot, ContainerID containerId)`: rcx the controller, edx the
/// slot, r8d the container id.
type HandleSlotSelectionFn = unsafe extern "C" fn(*mut c_void, i32, i32);

/// Area `hud.select_hotbar`: selects a hotbar slot through the game's HUD screen controller, as the game's own
/// hotbar keys do, with no key feed.
///
/// The detour on `HudScreenController.tick` keeps the HUD controller of the last tick, and the shared detour on
/// `ScreenController.dtor` (`ScreenControllerDetours`) makes the adapter forget it before it is freed. A select runs under the client call rules, on the game thread, only while the
/// HUD was ticked within `FRESH`.
pub struct HudAdapter {
    state: &'static LoaderState,
    handle_slot_selection: Option<usize>,
    // The HUD controller of the last tick and the time of that tick; 0 for none.
    hud: Mutex<(usize, Option<Instant>)>,
}

impl HudAdapter {
    pub const SELECT_HOTBAR: &'static str = "hud.select_hotbar";
    /// The hotbar slots: 0 to `HOTBAR_SLOTS - 1`.
    pub const HOTBAR_SLOTS: u8 = 9;
    /// A HUD controller not ticked for this long is not used.
    pub const FRESH: Duration = Duration::from_millis(500);
    const AREA_VERSION: u32 = 1;
    // `ContainerID::Inventory`.
    const INVENTORY_CONTAINER: i32 = 0;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the HUD tick and the screen controller destructor when the catalog shape of `hud.select_hotbar`
    /// resolved, and offers the area when both hooks are in. Logs one line and returns the row. Works once per
    /// process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::SELECT_HOTBAR)
            .collect();
        let handle_slot_selection = state
            .resolve("HudScreenController.handleSlotSelection")
            .ok()
            .and_then(|value| usize::try_from(value).ok())
            .filter(|address| *address != 0);
        let adapter = Self { state, handle_slot_selection, hud: Mutex::new((0, None)) };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the HUD adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() {
                let mut refused: Vec<&'static str> = HudDetours::hooks()
                    .into_iter()
                    .filter(|hook| !hook.install(state, Self::PRIORITY))
                    .map(|hook| hook.entry)
                    .collect();
                if !ScreenControllerDetours::install(state, Self::PRIORITY) {
                    refused.push(ScreenControllerDetours::DTOR);
                }
                if !refused.is_empty() {
                    row.shape = None;
                    row.missing = refused;
                }
            }
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static HudAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; nothing before `install`.
    pub(crate) fn with(body: impl FnOnce(&HudAdapter)) {
        if let Some(adapter) = ADAPTER.get() {
            body(adapter);
        }
    }

    /// Before the HUD tick: `controller` is the HUD controller from now on.
    pub fn note_tick(&self, controller: *mut c_void) {
        let address = controller.expose_provenance();
        if address != 0 {
            *self.lock() = (address, Some(Instant::now()));
        }
    }

    /// Before a screen controller destructor: the HUD controller is forgotten when it is `controller`.
    pub fn forget(&self, controller: *mut c_void) {
        let mut hud = self.lock();
        if hud.0 == controller.expose_provenance() {
            *hud = (0, None);
        }
    }

    /// Selects hotbar slot `slot` through the HUD controller. `REFUSED` for a slot of `HOTBAR_SLOTS` or above, and
    /// when no HUD was ticked within `FRESH`.
    pub fn select_hotbar_slot(&self, client: *const c_void, slot: u8) -> Result<(), AmmlStatus> {
        self.state.events().gate().require_open()?;
        if !self.state.areas().is_available(Self::SELECT_HOTBAR, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.check(client)?;
        if slot >= Self::HOTBAR_SLOTS {
            return Err(AmmlStatus::REFUSED);
        }
        let function = self.handle_slot_selection.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (hud, seen) = *self.lock();
        if hud == 0 || seen.is_none_or(|seen| seen.elapsed() > Self::FRESH) {
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: the entry is `HudScreenController::_handleSlotSelection`, with the signature of
        // `HandleSlotSelectionFn`.
        let select = unsafe { std::mem::transmute::<usize, HandleSlotSelectionFn>(function) };
        // SAFETY: `hud` is the HUD controller of a tick within `FRESH` whose destructor has not run, on the game thread.
        unsafe { select(std::ptr::with_exposed_provenance_mut(hud), i32::from(slot), Self::INVENTORY_CONTAINER) };
        curia::debug!("hotbar slot selected", { slot: slot });
        Ok(())
    }

    // The slot holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, (usize, Option<Instant>)> {
        self.hud.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
