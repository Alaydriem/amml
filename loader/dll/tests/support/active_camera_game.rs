use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU8, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{FakeRegistry, FakeStorage};

pub const CAMERA_TYPE: u32 = 0x4F60_47C7;
pub const ACTIVE_TYPE: u32 = 0x1130_40A1;

// The control block of the stand-in camera registry pointer, laid out as the MSVC `_Ref_count_base`: vtable, uses,
// weaks. The uses start high, so a release never reaches zero and the vtable is never called.
#[repr(C)]
struct ControlBlock {
    vtable: AtomicUsize,
    uses: AtomicI32,
    weaks: AtomicI32,
}

static CONTROL: ControlBlock =
    ControlBlock { vtable: AtomicUsize::new(0), uses: AtomicI32::new(100), weaks: AtomicI32::new(1) };
// The valid flag the control block's element points to.
static FLAG: AtomicU8 = AtomicU8::new(1);
// The address of the stand-in `CameraRegistry`, 0 for none.
static REGISTRY: AtomicUsize = AtomicUsize::new(0);
static SCENE: Mutex<Option<Scene>> = Mutex::new(None);

// Everything the game objects point to, kept alive for the test.
struct Scene {
    _entt: FakeRegistry,
    _cameras: FakeStorage,
    _active: FakeStorage,
    _entities: Vec<[u64; 4]>,
    _registry: Vec<u64>,
    _names: Vec<Vec<u8>>,
}

/// One camera entity of a scene: the name in its `CameraComponent`, whether it holds the active component, and whether
/// its entity slot is engaged.
#[derive(Clone, Copy)]
pub struct SceneCamera {
    pub name: &'static str,
    pub active: bool,
    pub engaged: bool,
}

impl SceneCamera {
    pub fn named(name: &'static str, active: bool) -> Self {
        Self { name, active, engaged: true }
    }
}

/// Stand-ins for the game objects `camera.active` reads: `ClientInstance::getCameraRegistry` (slot 10 of the stand-in
/// client vtable; it takes one use of the control block, as the game does), a `CameraRegistry` holding the camera entity
/// vector at 1D8h, and an entt registry with the `CameraComponent` and `ActiveCameraComponent` storages.
pub struct ActiveCameraGame;

impl ActiveCameraGame {
    pub const SLOT: u64 = 10;
    pub const ENTITIES_AT: usize = 0x1D8;

    /// The entries of `camera.active` in the shape of a build: layout 1 and `CameraComponent.size` 128h (70.26) when
    /// `size_shape`, else layout 0 and a factor 9 and shift 5 (26.63). `size` replaces the component size entries'
    /// value (the layout's own size when `None`).
    pub fn entries(size_shape: bool, size: Option<u64>) -> Vec<(&'static str, Resolved)> {
        let field = |value: u64| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let mut entries = vec![
            (
                "ClientInstance.getCameraRegistry",
                Resolved { kind: EntryKind::VSlot, value: Self::SLOT, recipes_agreeing: 2 },
            ),
            ("CameraRegistry.mCameraEntities", field(Self::ENTITIES_AT as u64)),
            ("CameraRegistry.cameraEntityStride", field(0x20)),
            ("CameraComponent.typeId", field(u64::from(CAMERA_TYPE))),
            ("ActiveCameraComponent.typeId", field(u64::from(ACTIVE_TYPE))),
            ("HashedString.layout", field(u64::from(size_shape))),
        ];
        if size_shape {
            entries.push(("CameraComponent.size", field(size.unwrap_or(0x128))));
        } else {
            entries.push(("CameraComponent.sizeFactor", field(size.unwrap_or(9))));
            entries.push(("CameraComponent.sizeShift", field(5)));
        }
        entries
    }

    /// Builds the registry of `cameras` for the layout of a shape (`size_shape` true is the 38h `HashedString`), and
    /// resets the valid flag and the use count.
    pub fn build(size_shape: bool, cameras: &[SceneCamera]) {
        let size = if size_shape { 0x128 } else { 0x120 };
        let mut camera_storage = FakeStorage::new(&[true, true], &[true], size);
        let mut active_storage = FakeStorage::new(&[true, true], &[], 1);
        let mut entities: Vec<[u64; 4]> = Vec::new();
        let mut entity_ids = Vec::new();
        let mut names: Vec<Vec<u8>> = Vec::new();
        for (index, camera) in cameras.iter().enumerate() {
            let entity = (((index as u32) + 1) << 18) | (0x805 + index as u32);
            entity_ids.push(entity);
            let packed = index as u32;
            camera_storage.insert(entity, packed);
            names.extend(Self::write_name(camera_storage.component_mut(packed), size_shape, camera.name));
            if camera.active {
                active_storage.insert(entity, packed);
            }
        }
        let entt = FakeRegistry::new(&[(CAMERA_TYPE, &camera_storage), (ACTIVE_TYPE, &active_storage)], 4);
        for (camera, entity) in cameras.iter().zip(&entity_ids) {
            entities.push([0, entt.address() as u64, u64::from(*entity), u64::from(camera.engaged)]);
        }
        let mut registry = vec![0u64; (Self::ENTITIES_AT + 0x20) / 8];
        let first = entities.as_ptr() as u64;
        let last = first + (entities.len() * 0x20) as u64;
        registry[Self::ENTITIES_AT / 8] = first;
        registry[Self::ENTITIES_AT / 8 + 1] = last;
        registry[Self::ENTITIES_AT / 8 + 2] = last;
        REGISTRY.store(registry.as_ptr() as usize, Ordering::SeqCst);
        FLAG.store(1, Ordering::SeqCst);
        CONTROL.uses.store(100, Ordering::SeqCst);
        *SCENE.lock().unwrap() = Some(Scene {
            _entt: entt,
            _cameras: camera_storage,
            _active: active_storage,
            _entities: entities,
            _registry: registry,
            _names: names,
        });
    }

    // The name in the `HashedString` at the start of `component`: the 30h layout holds the hash, then the
    // `std::string`; the 38h layout the hash, the variant storage and the index byte at 28h (0, the `std::string`). A
    // name past the 15-byte small string buffer lives in a heap block, which this gives back to keep alive.
    fn write_name(component: &mut [u8], size_shape: bool, name: &str) -> Option<Vec<u8>> {
        let len = name.len() as u64;
        let heap = (name.len() > 15).then(|| name.as_bytes().to_vec());
        match &heap {
            Some(block) => {
                component[8..16].copy_from_slice(&(block.as_ptr() as u64).to_le_bytes());
                component[0x20..0x28].copy_from_slice(&len.to_le_bytes());
            }
            None => {
                component[8..8 + name.len()].copy_from_slice(name.as_bytes());
                component[0x20..0x28].copy_from_slice(&15u64.to_le_bytes());
            }
        }
        component[0x18..0x20].copy_from_slice(&len.to_le_bytes());
        if size_shape {
            component[0x28] = 0;
        }
        heap
    }

    /// Moves the end pointer of the camera entity vector to `bytes` past its first element.
    pub fn set_entities_bytes(bytes: u64) {
        let mut scene = SCENE.lock().unwrap();
        let registry = &mut scene.as_mut().unwrap()._registry;
        let first = registry[Self::ENTITIES_AT / 8];
        registry[Self::ENTITIES_AT / 8 + 1] = first + bytes;
    }

    /// Points the camera entity vector at the entities of the scene again (after `null_entities`).
    pub fn restore_entities() {
        let mut scene = SCENE.lock().unwrap();
        let scene = scene.as_mut().unwrap();
        let first = scene._entities.as_ptr() as u64;
        let last = first + (scene._entities.len() * 0x20) as u64;
        scene._registry[Self::ENTITIES_AT / 8] = first;
        scene._registry[Self::ENTITIES_AT / 8 + 1] = last;
        scene._registry[Self::ENTITIES_AT / 8 + 2] = last;
    }

    /// Makes the camera entity vector null (no first, no last element).
    pub fn null_entities() {
        let mut scene = SCENE.lock().unwrap();
        let registry = &mut scene.as_mut().unwrap()._registry;
        registry[Self::ENTITIES_AT / 8] = 0;
        registry[Self::ENTITIES_AT / 8 + 1] = 0;
        registry[Self::ENTITIES_AT / 8 + 2] = 0;
    }

    /// The valid flag byte of the control block: 1 while the registry lives.
    pub fn set_valid(valid: bool) {
        FLAG.store(u8::from(valid), Ordering::SeqCst);
    }

    /// The registry is gone: `getCameraRegistry` gives an empty pointer.
    pub fn clear_registry() {
        REGISTRY.store(0, Ordering::SeqCst);
    }

    /// The use count of the control block.
    pub fn uses() -> i32 {
        CONTROL.uses.load(Ordering::SeqCst)
    }

    /// `NonOwnerPointer<CameraRegistry> getCameraRegistry()`: rcx the client, rdx the result buffer; writes the flag
    /// pointer, the control block and the registry, and takes one use.
    #[inline(never)]
    pub extern "C" fn camera_registry(this: *mut c_void, out: *mut [usize; 3]) -> *mut [usize; 3] {
        black_box(this);
        CONTROL.uses.fetch_add(1, Ordering::SeqCst);
        let value = [
            std::ptr::from_ref(&FLAG) as usize,
            std::ptr::from_ref(&CONTROL) as usize,
            REGISTRY.load(Ordering::SeqCst),
        ];
        // SAFETY: the caller passes a writable 24-byte buffer.
        unsafe { out.write(value) };
        out
    }
}
