use std::ffi::c_void;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicI32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_data::{EntryKind, Resolved};

use super::{FakeVtable, Harness};

// The Level vtable: 0 getLevelSeed64, 1 getLevelData.
static LEVEL_VTABLE: FakeVtable<2> = FakeVtable([InfoGame::level_seed as *const (), InfoGame::level_data as *const ()]);
// A stand-in DBStorage vtable global; only its address is used.
static STORAGE_VTABLE: [AtomicUsize; 2] = [const { AtomicUsize::new(0) }; 2];

static LEVEL_PRESENT: AtomicBool = AtomicBool::new(true);
static REALM: AtomicBool = AtomicBool::new(false);
static SEED: AtomicU64 = AtomicU64::new(0);
static GCI_SIZE: AtomicUsize = AtomicUsize::new(InfoGame::SIZE_2663);
static CONNECTED: AtomicBool = AtomicBool::new(false);
static KIND: AtomicI16 = AtomicI16::new(0);
static PORT: AtomicI32 = AtomicI32::new(0);
static HOST: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static DTORS: AtomicUsize = AtomicUsize::new(0);
static GETTERS: AtomicUsize = AtomicUsize::new(0);
static STORAGE_CALLS: AtomicUsize = AtomicUsize::new(0);
static LEVEL_OBJECT: OnceLock<usize> = OnceLock::new();
static LEVEL_DATA: OnceLock<usize> = OnceLock::new();
static STORAGE: OnceLock<usize> = OnceLock::new();

/// Stand-ins for the game functions and objects of the world info: three `ClientInstance` slots, a level with a seed
/// and level data, a connection info getter and destructor for both layouts, and a `DBStorage` constructor and
/// `checkShutdownDone` that the loader hooks.
pub struct InfoGame;

impl InfoGame {
    /// `GameConnectionInfo.size` on 26.63 and on 70.26.
    pub const SIZE_2663: usize = 0x2F0;
    pub const SIZE_2670: usize = 0x308;
    const NAME_AT: usize = 0x2B0;
    const COMMANDS_AT: usize = 0x4C9;
    const PATH_AT: usize = 0x98;
    const LEVEL_DATA_SIZE: usize = 0x500;
    const STORAGE_SIZE: usize = 0x200;

    /// Every entry of `client.world_info`, pointing to the stand-ins, for a connection info of `size` bytes; the client
    /// slots are 7, 8 and 9 of the `ClientGame` client vtable.
    pub fn entries(size: usize) -> Vec<(&'static str, Resolved)> {
        GCI_SIZE.store(size, Ordering::SeqCst);
        LEVEL_PRESENT.store(true, Ordering::SeqCst);
        REALM.store(false, Ordering::SeqCst);
        SEED.store(0, Ordering::SeqCst);
        CONNECTED.store(false, Ordering::SeqCst);
        DTORS.store(0, Ordering::SeqCst);
        GETTERS.store(0, Ordering::SeqCst);
        Self::set_level_name(b"");
        Self::set_commands(false);
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        vec![
            ("ClientInstance.getGameConnectionInfo", slot(7)),
            ("ClientInstance.getLevel", slot(8)),
            ("ClientInstance.isInRealm", slot(9)),
            ("GameConnectionInfo.size", field(size as u64)),
            ("GameConnectionInfo.dtor", function(Self::connection_dtor as *const () as usize)),
            ("Level.getLevelSeed64", slot(0)),
            ("ClientLevel.getLevelData", slot(1)),
            ("LevelData.mLevelName", field(Self::NAME_AT as u64)),
            ("LevelData.mCommandsEnabled", field(Self::COMMANDS_AT as u64)),
            (
                "DBStorage.vtable",
                Resolved {
                    kind: EntryKind::Global,
                    value: std::ptr::from_ref(&STORAGE_VTABLE) as usize as u64 - Harness::BASE,
                    recipes_agreeing: 2,
                },
            ),
            ("DBStorage.ctor", function(Self::storage_ctor as *const () as usize)),
            ("DBStorage.checkShutdownDone", function(Self::storage_shutdown as *const () as usize)),
            ("DBStorage.mFullPath", field(Self::PATH_AT as u64)),
        ]
    }

    /// The stand-in level has a seed `seed` from now on.
    pub fn set_seed(seed: u64) {
        SEED.store(seed, Ordering::SeqCst);
    }

    /// Whether the client has a level.
    pub fn set_level_present(present: bool) {
        LEVEL_PRESENT.store(present, Ordering::SeqCst);
    }

    pub fn set_realm(realm: bool) {
        REALM.store(realm, Ordering::SeqCst);
    }

    /// The level name bytes of the stand-in level data, as the game's `std::string`: any bytes, a long one in a heap
    /// block.
    pub fn set_level_name(name: &[u8]) {
        let data = Self::level_data_address();
        Self::write_string(data + Self::NAME_AT, name);
    }

    pub fn set_commands(enabled: bool) {
        let data = Self::level_data_address();
        // SAFETY: `data` is a leaked block of `LEVEL_DATA_SIZE` bytes, larger than the offset.
        unsafe { std::ptr::with_exposed_provenance_mut::<u8>(data + Self::COMMANDS_AT).write(u8::from(enabled)) };
    }

    /// The client has a connection of `kind` (the game's `ConnectionType`) to `host` and `port`; `None` clears it.
    pub fn set_connection(connection: Option<(i16, &[u8], i32)>) {
        match connection {
            Some((kind, host, port)) => {
                KIND.store(kind, Ordering::SeqCst);
                *HOST.lock().unwrap() = host.to_vec();
                PORT.store(port, Ordering::SeqCst);
                CONNECTED.store(true, Ordering::SeqCst);
            }
            None => CONNECTED.store(false, Ordering::SeqCst),
        }
    }

    /// The runs of the storage constructor and `checkShutdownDone` stand-ins together.
    pub fn storage_calls() -> usize {
        STORAGE_CALLS.load(Ordering::SeqCst)
    }

    /// The runs of the connection info destructor, and of the getter.
    pub fn calls() -> (usize, usize) {
        (DTORS.load(Ordering::SeqCst), GETTERS.load(Ordering::SeqCst))
    }

    /// Makes the game build a `DBStorage` whose folder is `path`: the stand-in constructor runs through its address, so
    /// the loader's detour sees it. Gives the address of the storage.
    pub fn construct_storage(path: &str) -> usize {
        let storage = Self::storage_address();
        Self::write_string(storage + Self::PATH_AT, path.as_bytes());
        let construct = black_box(Self::storage_ctor as extern "C" fn(usize, usize, usize) -> usize);
        construct(storage, black_box(0), black_box(0))
    }

    /// The game asks the live storage whether its shutdown is done; the stand-in answer is 1.
    pub fn shutdown_storage() -> u64 {
        let shutdown = black_box(Self::storage_shutdown as extern "C" fn(usize) -> u64);
        shutdown(black_box(Self::storage_address()))
    }

    /// Overwrites the path of the stand-in storage with another text, as the memory of a destroyed storage would be.
    pub fn destroy_storage() {
        Self::write_string(Self::storage_address() + Self::PATH_AT, b"destroyed");
    }

    // A leaked zeroed block of `size` bytes, its address kept in `cell`.
    fn leaked(cell: &OnceLock<usize>, size: usize) -> usize {
        *cell.get_or_init(|| {
            let block: &'static mut [u64] = Box::leak(vec![0u64; size.div_ceil(8)].into_boxed_slice());
            block.as_mut_ptr() as usize
        })
    }

    fn level_data_address() -> usize {
        Self::leaked(&LEVEL_DATA, Self::LEVEL_DATA_SIZE)
    }

    fn storage_address() -> usize {
        Self::leaked(&STORAGE, Self::STORAGE_SIZE)
    }

    /// The address of the stand-in level object: its first qword is the level vtable.
    fn level_address() -> usize {
        *LEVEL_OBJECT.get_or_init(|| {
            let block: &'static mut [u64] = Box::leak(vec![0u64; 2].into_boxed_slice());
            block[0] = std::ptr::from_ref(&LEVEL_VTABLE) as usize as u64;
            block.as_mut_ptr() as usize
        })
    }

    // Writes the 32 bytes of an MSVC `std::string` holding `bytes` at `at`: inline up to 15 bytes, else a leaked heap
    // block.
    fn write_string(at: usize, bytes: &[u8]) {
        let mut words = [0u64; 4];
        if bytes.len() <= 15 {
            let mut inline = [0u8; 16];
            inline[..bytes.len()].copy_from_slice(bytes);
            words[0] = u64::from_le_bytes(inline[..8].try_into().unwrap());
            words[1] = u64::from_le_bytes(inline[8..].try_into().unwrap());
            words[3] = 15;
        } else {
            let block: &'static mut [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
            words[0] = block.as_ptr() as usize as u64;
            words[3] = bytes.len() as u64;
        }
        words[2] = bytes.len() as u64;
        // SAFETY: `at` is inside a leaked block with room for the 32 bytes; the write is unaligned-safe.
        unsafe { std::ptr::with_exposed_provenance_mut::<[u64; 4]>(at).write_unaligned(words) };
    }

    /// The `getGameConnectionInfo` stand-in: it fills the optional behind `out` as the game does.
    #[inline(never)]
    pub extern "C" fn connection_info(this: *mut c_void, out: *mut u8) -> *mut u8 {
        black_box(this);
        GETTERS.fetch_add(black_box(1), Ordering::SeqCst);
        let size = GCI_SIZE.load(Ordering::SeqCst);
        // SAFETY: the loader gives a block of the value and the flag after it, `size + 8` bytes.
        unsafe {
            out.add(size).write(0);
            if CONNECTED.load(Ordering::SeqCst) {
                let (host_at, port_at) = if size == Self::SIZE_2663 { (0x08, 0x8C) } else { (0x20, 0xA4) };
                out.cast::<i16>().write_unaligned(KIND.load(Ordering::SeqCst));
                Self::write_string(out.add(host_at).expose_provenance(), &HOST.lock().unwrap());
                out.add(port_at).cast::<i32>().write_unaligned(PORT.load(Ordering::SeqCst));
                out.add(size).write(1);
            }
        }
        out
    }

    #[inline(never)]
    pub extern "C" fn get_level(this: *mut c_void) -> *mut c_void {
        black_box(this);
        if LEVEL_PRESENT.load(Ordering::SeqCst) {
            std::ptr::with_exposed_provenance_mut(Self::level_address())
        } else {
            std::ptr::null_mut()
        }
    }

    #[inline(never)]
    pub extern "C" fn is_in_realm(this: *mut c_void) -> u8 {
        black_box(this);
        u8::from(REALM.load(Ordering::SeqCst))
    }

    #[inline(never)]
    extern "C" fn level_seed(this: *mut c_void, out: *mut u64) -> *mut u64 {
        black_box(this);
        // SAFETY: the loader passes a writable 8-byte result.
        unsafe { out.write(SEED.load(Ordering::SeqCst)) };
        out
    }

    #[inline(never)]
    extern "C" fn level_data(this: *mut c_void) -> usize {
        black_box(this);
        Self::level_data_address()
    }

    #[inline(never)]
    extern "C" fn connection_dtor(this: *mut u8) {
        black_box(this);
        DTORS.fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn storage_ctor(this: usize, config: usize, env: usize) -> usize {
        black_box((config, env));
        STORAGE_CALLS.fetch_add(black_box(1), Ordering::SeqCst);
        black_box(this)
    }

    #[inline(never)]
    extern "C" fn storage_shutdown(this: usize) -> u64 {
        black_box(this);
        STORAGE_CALLS.fetch_add(black_box(1), Ordering::SeqCst);
        black_box(1)
    }
}
