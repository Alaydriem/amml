use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};
use amml_msvc::{MsvcHeap, MsvcString};

use super::{FakeVtable, Harness, HeapProbe, SkinSpec};

// What the stand-in `Json::Value::toStyledString` writes, and how often it ran.
static GEOMETRY: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static STYLED_CALLS: AtomicUsize = AtomicUsize::new(0);
// The handlers the original game ran so far.
static HANDLED: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
// The byte at `NetworkGame::ENABLED_AT` of each packet the original `NetworkSystem::send` got, in order.
static SENT_BYTES: Mutex<Vec<u8>> = Mutex::new(Vec::new());

#[inline(never)]
extern "C" fn list_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    std::hint::black_box((this, source, packet));
    HANDLED.lock().unwrap().push(std::hint::black_box("list"));
}

#[inline(never)]
extern "C" fn skin_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    std::hint::black_box((this, source, packet));
    HANDLED.lock().unwrap().push(std::hint::black_box("skin"));
}

#[inline(never)]
extern "C" fn send_stand_in(system: *mut c_void, id: *const c_void, packet: *const c_void, sub: u32) {
    std::hint::black_box((system, id, sub));
    // SAFETY: `packet` is a stand-in packet of `NetworkGame::packet`, at least `PACKET_SIZE` bytes.
    let byte = unsafe { packet.cast::<u8>().add(NetworkGame::ENABLED_AT).read() };
    SENT_BYTES.lock().unwrap().push(std::hint::black_box(byte));
}

#[inline(never)]
extern "C" fn styled_stand_in(value: usize, out: *mut MsvcString) -> *mut MsvcString {
    std::hint::black_box(value);
    STYLED_CALLS.fetch_add(1, Ordering::SeqCst);
    let text = GEOMETRY.lock().unwrap().clone();
    let text = String::from_utf8(text).unwrap();
    // SAFETY: `HeapProbe` is a heap of `operator new` and a sized `operator delete`.
    let string = unsafe { MsvcString::new_in(&NetworkGame::heap(), &text) }.unwrap();
    // SAFETY: `out` is the uninitialised string the loader gives, with the room of one `MsvcString`.
    unsafe { out.write(string) };
    out
}

unsafe extern "C" fn probe_alloc(size: usize) -> *mut u8 {
    // SAFETY: the same contract as `HeapProbe::alloc`.
    unsafe { HeapProbe::alloc(size) }
}

static CACHE_PACKET: FakeVtable<2> = FakeVtable([std::ptr::null(), std::ptr::null()]);
static OTHER_PACKET: FakeVtable<2> = FakeVtable([std::ptr::null(), std::ptr::dangling()]);

static HANDLERS: FakeVtable<2> = FakeVtable([list_stand_in as *const (), skin_stand_in as *const ()]);

/// Stand-in game memory for the network adapter: the packets, the player list entries and the skin objects, laid out
/// as the 26.63 derived entries say, and the entry table that resolves them. Everything is leaked; the address of a
/// built packet stays valid for the test process.
pub struct NetworkGame;

impl NetworkGame {
    pub const LIST: usize = 0;
    pub const SKIN: usize = 1;

    pub const ENTRIES_AT: usize = 0x30;
    pub const ENTRY_SIZE: usize = 0xd8;
    pub const VARIANT_AT: usize = 0xd0;
    pub const UUID_AT: usize = 0x08;
    pub const ID_AT: usize = 0x18;
    pub const NAME_AT: usize = 0x20;
    pub const XUID_AT: usize = 0x40;
    pub const SKIN_REF_AT: usize = 0xa8;
    pub const SKIN_PACKET_UUID_AT: usize = 0x30;
    pub const SKIN_PACKET_SKIN_AT: usize = 0x40;
    pub const SKIN_SIZE: usize = 0x200;
    pub const IMAGE_AT: usize = 0x80;
    pub const ANIMATED_AT: usize = 0xe0;
    pub const ANIMATED_STRIDE: usize = 0x40;
    pub const ANIMATED_IMAGE_AT: usize = 0x08;
    pub const GEOMETRY_AT: usize = 0xf8;
    pub const GEOMETRY_NAME_AT: usize = 0x60;
    pub const PREMIUM_AT: usize = 0x1e9;
    pub const PERSONA_AT: usize = 0x1ea;
    pub const PATCH_AT: usize = 0x40;
    pub const ENABLED_AT: usize = 0x30;
    pub const PACKET_SIZE: usize = 0x38;
    pub const PROTOCOL: u64 = 2225;

    /// The heap the stand-in `toStyledString` builds on; the loader frees on the same one.
    pub fn heap() -> MsvcHeap {
        MsvcHeap::new(probe_alloc, HeapProbe::free)
    }

    /// The entries of the table of a build that has every network row.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let global = |address: usize| Resolved {
            kind: EntryKind::Global,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
        vec![
            ("LegacyClientNetworkHandler.vtable", global(std::ptr::from_ref(&HANDLERS) as usize)),
            ("ClientNetworkHandler.handle.PlayerListPacket", slot(Self::LIST as u64)),
            ("ClientNetworkHandler.handle.PlayerSkinPacket", slot(Self::SKIN as u64)),
            ("PlayerListPacket.mEntries", field(Self::ENTRIES_AT)),
            ("PlayerListAddEntry.size", field(Self::ENTRY_SIZE)),
            ("PlayerListPacket.variantIndex", field(Self::VARIANT_AT)),
            ("PlayerListAddEntry.mUUID", field(Self::UUID_AT)),
            ("PlayerListAddEntry.mId", field(Self::ID_AT)),
            ("PlayerListAddEntry.mSkin", field(Self::SKIN_REF_AT)),
            ("PlayerListAddEntry.mName", field(Self::NAME_AT)),
            ("PlayerListAddEntry.mXUID", field(Self::XUID_AT)),
            ("PlayerSkinPacket.mUUID", field(Self::SKIN_PACKET_UUID_AT)),
            ("PlayerSkinPacket.mSkin", field(Self::SKIN_PACKET_SKIN_AT)),
            ("SerializedSkinImpl.mSkinImage", field(Self::IMAGE_AT)),
            ("SerializedSkinImpl.mSkinAnimatedImages", field(Self::ANIMATED_AT)),
            ("AnimatedImageData.size", field(Self::ANIMATED_STRIDE)),
            ("AnimatedImageData.mType", field(0)),
            ("AnimatedImageData.mImage", field(Self::ANIMATED_IMAGE_AT)),
            ("SerializedSkinImpl.mGeometryData", field(Self::GEOMETRY_AT)),
            ("SerializedSkinImpl.mDefaultGeometryName", field(Self::GEOMETRY_NAME_AT)),
            ("JsonValue.toStyledString", function(styled_stand_in as *const () as usize)),
            ("SerializedSkinImpl.mIsPersona", field(Self::PERSONA_AT)),
            ("SerializedSkinImpl.mIsPremium", field(Self::PREMIUM_AT)),
            ("SerializedSkinImpl.mResourcePatch", field(Self::PATCH_AT)),
            ("NetworkSystem.send", function(send_stand_in as *const () as usize)),
            ("ClientCacheStatusPacket.vtable", global(std::ptr::from_ref(&CACHE_PACKET) as usize)),
            ("ClientCacheStatusPacket.mEnabled", field(Self::ENABLED_AT)),
            ("SharedConstants.NetworkProtocolVersion", field(Self::PROTOCOL as usize)),
        ]
    }

    /// Sets the text the stand-in `toStyledString` writes.
    pub fn set_geometry(text: &[u8]) {
        *GEOMETRY.lock().unwrap() = text.to_vec();
    }

    /// How often the stand-in `toStyledString` ran.
    pub fn styled_calls() -> usize {
        STYLED_CALLS.load(Ordering::SeqCst)
    }

    /// The handlers the original game ran so far, in order.
    pub fn handled() -> Vec<&'static str> {
        HANDLED.lock().unwrap().clone()
    }

    /// The hooked game handler of `slot`, as the game calls it through its vtable, with `packet`.
    pub fn handle(slot: usize, packet: usize) {
        // SAFETY: the vtable holds the two stand-in handlers, which have the signature of the game's handler.
        let function = unsafe {
            std::mem::transmute::<*const (), extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>(HANDLERS.0[slot])
        };
        let function = std::hint::black_box(function);
        function(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::with_exposed_provenance_mut(packet));
    }

    /// A packet of 38h bytes whose byte at `ENABLED_AT` is `byte`: a `ClientCacheStatusPacket` when `cache_status`, any
    /// other packet otherwise. Gives its address.
    pub fn packet(cache_status: bool, byte: u8) -> usize {
        let mut packet = vec![0u8; Self::PACKET_SIZE];
        let vtable = if cache_status { std::ptr::from_ref(&CACHE_PACKET) } else { std::ptr::from_ref(&OTHER_PACKET) };
        packet[..8].copy_from_slice(&vtable.expose_provenance().to_le_bytes());
        packet[Self::ENABLED_AT] = byte;
        let packet: &'static mut [u8] = Box::leak(packet.into_boxed_slice());
        packet.as_ptr().expose_provenance()
    }

    /// The game sends `packet` through the hooked `NetworkSystem::send`; gives the byte at `ENABLED_AT` the original
    /// function saw.
    pub fn send(packet: usize) -> u8 {
        let function =
            std::hint::black_box(send_stand_in as extern "C" fn(*mut c_void, *const c_void, *const c_void, u32));
        function(std::ptr::null_mut(), std::ptr::null(), std::ptr::with_exposed_provenance(packet), 0);
        SENT_BYTES.lock().unwrap().last().copied().unwrap()
    }

    /// How many packets the original `NetworkSystem::send` got.
    pub fn sent() -> usize {
        SENT_BYTES.lock().unwrap().len()
    }

    /// A `std::string`: inline up to 15 bytes, on the heap above.
    pub fn string(bytes: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        if bytes.len() <= 15 {
            out[..bytes.len()].copy_from_slice(bytes);
            out[0x10..0x18].copy_from_slice(&bytes.len().to_le_bytes());
            out[0x18..0x20].copy_from_slice(&15usize.to_le_bytes());
        } else {
            let heap: &'static mut [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
            out[..8].copy_from_slice(&heap.as_ptr().expose_provenance().to_le_bytes());
            out[0x10..0x18].copy_from_slice(&bytes.len().to_le_bytes());
            out[0x18..0x20].copy_from_slice(&bytes.len().to_le_bytes());
        }
        out
    }

    /// A skin object of `spec` and the `shared_ptr` that points to it, as 16 bytes.
    pub fn skin_ref(spec: &SkinSpec) -> [u8; 16] {
        let mut skin = vec![0u8; Self::SKIN_SIZE];
        Self::image(&mut skin, Self::IMAGE_AT, spec.size, spec.blob_len);
        let mut animated = Vec::new();
        for (kind, width, height) in &spec.animated {
            let mut element = vec![0u8; Self::ANIMATED_STRIDE];
            element[..4].copy_from_slice(&kind.to_le_bytes());
            Self::image(&mut element, Self::ANIMATED_IMAGE_AT, (*width, *height), None);
            animated.extend(element);
        }
        let animated: &'static mut [u8] = Box::leak(animated.into_boxed_slice());
        let begin = animated.as_ptr().expose_provenance();
        let end = begin + animated.len();
        skin[Self::ANIMATED_AT..Self::ANIMATED_AT + 8].copy_from_slice(&begin.to_le_bytes());
        skin[Self::ANIMATED_AT + 8..Self::ANIMATED_AT + 16].copy_from_slice(&end.to_le_bytes());
        skin[Self::PERSONA_AT] = u8::from(spec.persona);
        skin[Self::PREMIUM_AT] = u8::from(spec.premium);
        skin[Self::GEOMETRY_NAME_AT..Self::GEOMETRY_NAME_AT + 32].copy_from_slice(&Self::string(&spec.geometry_name));
        skin[Self::PATCH_AT..Self::PATCH_AT + 32].copy_from_slice(&Self::string(&spec.resource_patch));
        let skin: &'static mut [u8] = Box::leak(skin.into_boxed_slice());
        let mut reference = [0u8; 16];
        reference[..8].copy_from_slice(&skin.as_ptr().expose_provenance().to_le_bytes());
        reference
    }

    // Writes an `mce::Image` of `size` at `at` in `memory`, with a leaked pixel blob whose bytes count up from 1.
    fn image(memory: &mut [u8], at: usize, size: (u32, u32), blob_len: Option<usize>) {
        let length = blob_len.unwrap_or(size.0 as usize * size.1 as usize * 4);
        let blob: Vec<u8> = (0..length).map(|index| (index % 251) as u8 + 1).collect();
        let blob: &'static mut [u8] = Box::leak(blob.into_boxed_slice());
        let address = blob.as_ptr().expose_provenance();
        memory[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
        memory[at + 4..at + 8].copy_from_slice(&size.0.to_le_bytes());
        memory[at + 8..at + 12].copy_from_slice(&size.1.to_le_bytes());
        memory[at + 0x20..at + 0x28].copy_from_slice(&address.to_le_bytes());
        memory[at + 0x28..at + 0x30].copy_from_slice(&length.to_le_bytes());
    }

    /// One Add entry: the uuid, the actor id, the name and xuid texts and the skin ref (zeros for no skin).
    pub fn add_entry(uuid: [u8; 16], actor: i64, name: &[u8], xuid: &[u8], skin: Option<[u8; 16]>) -> Vec<u8> {
        let mut entry = Self::remove_entry(uuid);
        entry[Self::VARIANT_AT] = 1;
        entry[Self::ID_AT..Self::ID_AT + 8].copy_from_slice(&actor.to_le_bytes());
        entry[Self::NAME_AT..Self::NAME_AT + 32].copy_from_slice(&Self::string(name));
        entry[Self::XUID_AT..Self::XUID_AT + 32].copy_from_slice(&Self::string(xuid));
        if let Some(skin) = skin {
            entry[Self::SKIN_REF_AT..Self::SKIN_REF_AT + 16].copy_from_slice(&skin);
        }
        entry
    }

    /// One Remove entry.
    pub fn remove_entry(uuid: [u8; 16]) -> Vec<u8> {
        let mut entry = vec![0u8; Self::ENTRY_SIZE];
        entry[Self::UUID_AT..Self::UUID_AT + 16].copy_from_slice(&uuid);
        entry
    }

    /// A `PlayerListPacket` that holds `entries`; gives its address.
    pub fn list_packet(entries: Vec<Vec<u8>>) -> usize {
        let all: Vec<u8> = entries.into_iter().flatten().collect();
        let all: &'static mut [u8] = Box::leak(all.into_boxed_slice());
        let begin = all.as_ptr().expose_provenance();
        let mut packet = vec![0u8; 0x50];
        packet[Self::ENTRIES_AT..Self::ENTRIES_AT + 8].copy_from_slice(&begin.to_le_bytes());
        packet[Self::ENTRIES_AT + 8..Self::ENTRIES_AT + 16].copy_from_slice(&(begin + all.len()).to_le_bytes());
        let packet: &'static mut [u8] = Box::leak(packet.into_boxed_slice());
        packet.as_ptr().expose_provenance()
    }

    /// A `PlayerSkinPacket` of `uuid` and `skin`; gives its address.
    pub fn skin_packet(uuid: [u8; 16], skin: Option<[u8; 16]>) -> usize {
        let mut packet = vec![0u8; 0x98];
        packet[Self::SKIN_PACKET_UUID_AT..Self::SKIN_PACKET_UUID_AT + 16].copy_from_slice(&uuid);
        if let Some(skin) = skin {
            packet[Self::SKIN_PACKET_SKIN_AT..Self::SKIN_PACKET_SKIN_AT + 16].copy_from_slice(&skin);
        }
        let packet: &'static mut [u8] = Box::leak(packet.into_boxed_slice());
        packet.as_ptr().expose_provenance()
    }
}
