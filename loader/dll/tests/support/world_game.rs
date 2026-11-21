use std::collections::VecDeque;
use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, Harness};

// A stand-in World::WorldSystem of 0x100 bytes: the LocalWorldStarter pointer at `STARTER_OFFSET`.
static WORLD_SYSTEM: [AtomicUsize; 32] = [const { AtomicUsize::new(0) }; 32];
// A stand-in World::LocalWorldStarter; only its address is used.
static STARTER: AtomicUsize = AtomicUsize::new(0);
static RESULTS: Mutex<VecDeque<u32>> = Mutex::new(VecDeque::new());
static IDS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static THIS_SEEN: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// Stand-ins for the world start path: `MinecraftGame::getWorldSystem` (slot 5 of the `ClientGame`
/// MinecraftGame vtable), a WorldSystem that holds a LocalWorldStarter pointer, and
/// `LocalWorldStarter::startLocalWorld`, which gives queued results (0 when none is queued).
pub struct WorldGame;

impl WorldGame {
    /// The offset of `mLocalWorldStarter` in the stand-in WorldSystem.
    pub const STARTER_OFFSET: usize = 0x78;

    /// The `ClientGame` entries and the three world start entries; `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        WORLD_SYSTEM[Self::STARTER_OFFSET / 8].store(Self::starter(), Ordering::SeqCst);
        let mut entries = ClientGame::entries(without);
        let world = [
            ("MinecraftGame.getWorldSystem", Resolved { kind: EntryKind::VSlot, value: 5, recipes_agreeing: 2 }),
            (
                "WorldSystem.mLocalWorldStarter",
                Resolved { kind: EntryKind::Field, value: Self::STARTER_OFFSET as u64, recipes_agreeing: 2 },
            ),
            (
                "LocalWorldStarter.startLocalWorld",
                Resolved {
                    kind: EntryKind::Function,
                    value: Self::start_local_world as *const () as usize as u64 - Harness::BASE,
                    recipes_agreeing: 2,
                },
            ),
        ];
        entries.extend(world.into_iter().filter(|(id, _)| !without.contains(id)));
        entries
    }

    /// The address of the stand-in LocalWorldStarter.
    pub fn starter() -> usize {
        std::ptr::from_ref(&STARTER) as usize
    }

    /// Sets the LocalWorldStarter pointer the WorldSystem holds; 0 is null.
    pub fn set_starter(value: usize) {
        WORLD_SYSTEM[Self::STARTER_OFFSET / 8].store(value, Ordering::SeqCst);
    }

    /// The results the next `startLocalWorld` calls give, in order.
    pub fn queue_results(results: &[u32]) {
        RESULTS.lock().unwrap().extend(results);
    }

    /// How many times `startLocalWorld` ran.
    pub fn calls() -> usize {
        IDS.lock().unwrap().len()
    }

    /// The WorldID text of each `startLocalWorld` call, in order.
    pub fn ids() -> Vec<String> {
        IDS.lock().unwrap().clone()
    }

    /// The `this` of each `startLocalWorld` call, in order.
    pub fn this_seen() -> Vec<usize> {
        THIS_SEEN.lock().unwrap().clone()
    }

    /// `World::WorldSystem& getWorldSystem() const` on the stand-in MinecraftGame.
    #[inline(never)]
    pub extern "C" fn world_system(this: *mut c_void) -> *mut c_void {
        black_box(this);
        std::ptr::from_ref(&WORLD_SYSTEM).cast_mut().cast()
    }

    // `StartLocalWorldResult startLocalWorld(const World::WorldID&)`: reads the MSVC string in rdx.
    #[inline(never)]
    extern "C" fn start_local_world(this: *mut c_void, id: *const [usize; 4]) -> u32 {
        // SAFETY: the caller passes a live 32-byte MSVC std::string.
        let [first, second, size, capacity] = unsafe { id.read() };
        let data = if capacity <= 15 { id.cast::<u8>() } else { first as *const u8 };
        black_box(second);
        // SAFETY: the string holds `size` bytes at `data`.
        let bytes = unsafe { std::slice::from_raw_parts(data, size) };
        IDS.lock().unwrap().push(String::from_utf8_lossy(bytes).into_owned());
        THIS_SEEN.lock().unwrap().push(this as usize);
        RESULTS.lock().unwrap().pop_front().unwrap_or(0)
    }
}
