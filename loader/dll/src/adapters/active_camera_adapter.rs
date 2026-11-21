use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlCameraKind, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::{MemberCall, MsvcVec};
use amml_runtime::CameraKindNames;

use crate::{
    CameraEntityLayout, CameraRegistryHold, ClientAdapter, EnttLookup, LoaderState, NameReader, NonOwnerPointerLayout,
    SessionAdapter, ThreadNote,
};

static ADAPTER: OnceLock<ActiveCameraAdapter> = OnceLock::new();

// What the adapter reads of the game, checked against the entries of the running build.
struct ActiveCameraReads {
    // The `ClientInstance.getCameraRegistry` slot.
    registry_slot: usize,
    // The offset of `mCameraEntities` in the `CameraRegistry`.
    entities_at: usize,
    camera_type: u32,
    active_type: u32,
    // The size of a `CameraComponent`, the stride of its payload pages.
    component_size: usize,
    names: NameReader,
}

/// The `camera.active` area: which vanilla camera the client shows the world through.
///
/// `ClientInstance::getCameraRegistry` gives the `CameraRegistry`; the camera with the game's `ActiveCameraComponent`
/// in its `mCameraEntities` is the active one, and the name in its `CameraComponent` (`minecraft:first_person`,
/// `minecraft:third_person`, `minecraft:third_person_front`, anything else is `OTHER`) gives the kind. Both components
/// are found through the game's entt lookup (`EnttLookup`), bounds-checked at each step. The registry pointer the game
/// hands out holds one use of its control block, given back on every path (`CameraRegistryHold`).
///
/// A call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread, the primary
/// client and a joined world (`REFUSED`); `FAILED` when the game gives no registry or no camera is active. A refused
/// camera vector (null, inverted, a length that is not whole elements or more than 256) logs one warning per process.
/// The thread of a call is logged once and when it changes.
pub struct ActiveCameraAdapter {
    state: &'static LoaderState,
    reads: ActiveCameraReads,
    thread: ThreadNote,
    // Set once the first refused camera vector is logged.
    vector_logged: AtomicBool,
}

impl ActiveCameraAdapter {
    pub const AREA: &'static str = "camera.active";
    const AREA_VERSION: u32 = 1;
    // The most camera entities a read walks; the game holds a handful.
    const MAX_CAMERAS: usize = 256;
    // The `CameraComponent` size of each `HashedString.layout`: the name is the first member, so a layout that grows
    // the name grows the component by the same amount.
    const COMPONENT_SIZE_30: usize = 0x120;
    const COMPONENT_SIZE_38: usize = 0x128;

    /// Offers `camera.active` when its catalog shape resolved and the entries fit their mirrors: the camera entity
    /// stride is the size of `CameraEntityLayout`, `HashedString.layout` names a mirror, and the `CameraComponent` size
    /// of the shape is the one that layout gives. Else `camera layout mismatch` names the entry. Logs one `area adapter`
    /// or `area not available` line and returns the row. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let value = |name: &str| state.resolve(name).ok();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        let mut reads = None;
        for row in &mut rows {
            let Some(shape) = row.shape else { continue };
            match Self::reads_of(shape, &value) {
                Ok(found) => reads = Some(found),
                Err(entry) => {
                    curia::warn!("camera layout mismatch", { area: row.id, entry: entry, detail: "does not fit its mirror" });
                    row.shape = None;
                    row.missing = vec![entry];
                }
            }
        }
        if let Some(reads) = reads {
            let adapter = Self { state, reads, thread: ThreadNote::new(), vector_logged: AtomicBool::new(false) };
            if ADAPTER.set(adapter).is_err() {
                curia::warn!("the active camera adapter is already installed");
                return rows;
            }
        }
        for row in &rows {
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

    // The reads of the shape `shape`; else the entry that is missing or does not fit its mirror.
    fn reads_of(shape: &str, value: &impl Fn(&str) -> Option<u64>) -> Result<ActiveCameraReads, &'static str> {
        const STRIDE: &str = "CameraRegistry.cameraEntityStride";
        const LAYOUT: &str = "HashedString.layout";
        const FACTOR: &str = "CameraComponent.sizeFactor";
        const SHIFT: &str = "CameraComponent.sizeShift";
        const SIZE: &str = "CameraComponent.size";
        let entry = |name: &'static str| value(name).ok_or(name);
        let narrow = |name: &'static str| entry(name).and_then(|found| usize::try_from(found).map_err(|_| name));
        let id = |name: &'static str| entry(name).and_then(|found| u32::try_from(found).map_err(|_| name));
        if entry(STRIDE)? != CameraEntityLayout::STRIDE {
            return Err(STRIDE);
        }
        let layout = entry(LAYOUT)?;
        let names = NameReader::new(layout).ok_or(LAYOUT)?;
        let (size_entry, component_size) = if shape == AdapterCatalog::ACTIVE_CAMERA_FACTOR_SHAPE {
            let size = entry(FACTOR)?.checked_shl(u32::try_from(entry(SHIFT)?).map_err(|_| SHIFT)?).ok_or(SHIFT)?;
            (FACTOR, usize::try_from(size).map_err(|_| FACTOR)?)
        } else {
            (SIZE, narrow(SIZE)?)
        };
        let expected = if layout == NameReader::LAYOUT_30 { Self::COMPONENT_SIZE_30 } else { Self::COMPONENT_SIZE_38 };
        if component_size != expected {
            return Err(size_entry);
        }
        Ok(ActiveCameraReads {
            registry_slot: narrow("ClientInstance.getCameraRegistry")?,
            entities_at: narrow("CameraRegistry.mCameraEntities")?,
            camera_type: id("CameraComponent.typeId")?,
            active_type: id("ActiveCameraComponent.typeId")?,
            component_size,
            names,
        })
    }

    /// The installed adapter; `None` before `install`, or when the area is not offered.
    pub fn get() -> Option<&'static ActiveCameraAdapter> {
        ADAPTER.get()
    }

    /// The vanilla camera `client` shows the world through; `client` must be the primary client.
    pub fn active_camera(&self, client: *const c_void) -> Result<AmmlCameraKind, AmmlStatus> {
        let client = self.enter(client)?;
        let function = ClientAdapter::virtual_fn(client, Some(self.reads.registry_slot)).ok_or(AmmlStatus::FAILED)?;
        let this = std::ptr::with_exposed_provenance_mut::<c_void>(client);
        // SAFETY: the slot holds `Bedrock::NonOwnerPointer<CameraRegistry> ClientInstance::getCameraRegistry()`, a
        // member function that writes the 24-byte pointer through a hidden result pointer; `client` is the live
        // primary client, on the game thread.
        let pointer = unsafe { MemberCall::call_sret0::<NonOwnerPointerLayout>(function as *const c_void, this) };
        // SAFETY: `pointer` is what the game just handed out, with its own use of the control block.
        let hold = unsafe { CameraRegistryHold::new(pointer) };
        let registry = hold.registry().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `registry` is the live `CameraRegistry` of the primary client, kept alive by `hold`, read on the
        // game thread; the offsets and ids are derived for this build and checked at install.
        unsafe { self.read_active(registry) }.ok_or(AmmlStatus::FAILED)
    }

    // The gate, the area, the thread, the primary client and a joined world of a call: `NOT_READY`, `NOT_AVAILABLE`,
    // `REFUSED`. The thread of the call is logged the first time and when it changes.
    fn enter(&self, client: *const c_void) -> Result<usize, AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::AREA, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.thread.changed(thread) {
            curia::info!("world thread", { hook: "ClientInstance.getCameraRegistry", game: on_game_thread });
        }
        if !on_game_thread {
            return Err(AmmlStatus::REFUSED);
        }
        let client = client.expose_provenance();
        match SessionAdapter::primary_client() {
            Some(primary) if primary == client && gate.epoch().0 != 0 => Ok(client),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // One `camera layout mismatch` warning for the first refused `mCameraEntities` vector of the process, so a moved
    // member is told apart from "no active camera" in the log.
    fn refuse_vector(&self, detail: &'static str, bytes: usize) {
        if !self.vector_logged.swap(true, Ordering::Relaxed) {
            curia::warn!("camera layout mismatch", {
                area: Self::AREA,
                entry: "CameraRegistry.mCameraEntities",
                detail: detail,
                bytes: bytes,
            });
        }
    }

    // The kind of the first engaged camera entity that holds the active camera component; `None` when none does.
    //
    // Safety: `registry` is a live `CameraRegistry`, read on the game thread; `self.reads` fit this build.
    #[allow(unsafe_code)]
    unsafe fn read_active(&self, registry: usize) -> Option<AmmlCameraKind> {
        let vector = registry.checked_add(self.reads.entities_at)?;
        // SAFETY: the registry holds its `mCameraEntities` vector at the derived offset.
        let [first, last, _] =
            unsafe { (*std::ptr::with_exposed_provenance::<MsvcVec<CameraEntityLayout>>(vector)).raw_parts() };
        if first.is_null() {
            // A vector that never allocated (null begin and end) is empty: no camera, not a mismatch.
            if !last.is_null() {
                self.refuse_vector("null begin", 0);
            }
            return None;
        }
        let stride = size_of::<CameraEntityLayout>();
        let Some(bytes) = last.addr().checked_sub(first.addr()) else {
            self.refuse_vector("inverted range", 0);
            return None;
        };
        if bytes % stride != 0 || bytes / stride > Self::MAX_CAMERAS {
            self.refuse_vector("bad length", bytes);
            return None;
        }
        (0..bytes / stride).find_map(|index| {
            // SAFETY: the element is inside `first..last`, which the game's vector holds initialised.
            let element = unsafe { &*first.add(index) };
            if element.engaged != 1 || element.entt.is_null() {
                return None;
            }
            // SAFETY: `entt` is the entt registry of a live, engaged camera entity.
            let active = unsafe { EnttLookup::storage(element.entt, self.reads.active_type) }
                // SAFETY: the active camera component storage of that registry.
                .and_then(|storage| unsafe { EnttLookup::packed_index(storage, element.entity) });
            active?;
            // SAFETY: as above; the camera component storage of that registry.
            let component = unsafe { EnttLookup::storage(element.entt, self.reads.camera_type) }
                // SAFETY: the storage holds components of `component_size` bytes, checked at install.
                .and_then(|storage| unsafe {
                    EnttLookup::component(storage, element.entity, self.reads.component_size)
                });
            // SAFETY: the component is a live `CameraComponent`; its name is the first member, of the layout checked
            // at install.
            let name = component.and_then(|address| unsafe { self.reads.names.read(address) });
            Some(CameraKindNames::kind_of(name.as_deref().unwrap_or("")))
        })
    }
}
