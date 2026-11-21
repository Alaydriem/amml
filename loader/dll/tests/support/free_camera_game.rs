use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{FakeVtable, Harness, HeapProbe, SessionGame};

// The control block every made packet shares, laid out as the MSVC `_Ref_count_base`: vtable, uses, weaks. The uses
// start high, so a release never reaches zero and the vtable is never called; each made packet adds one use.
#[repr(C)]
struct ControlBlock {
    vtable: AtomicUsize,
    uses: AtomicI32,
    weaks: AtomicI32,
}

// The control block of one shapes packet when `OWN_BLOCKS` is set: the MSVC `_Ref_count_base` (vtable, uses, weaks)
// and the packet. The last use destroys the packet as the game does: its shape vector frees its block.
#[repr(C)]
struct PacketControl {
    vtable: &'static FakeVtable<4>,
    uses: AtomicI32,
    weaks: AtomicI32,
    packet: usize,
}

static PACKET_CONTROL_VTABLE: FakeVtable<4> = FakeVtable([
    FreeCameraGame::destroy_packet as *const (),
    FreeCameraGame::delete_control as *const (),
    FreeCameraGame::destroy_packet as *const (),
    FreeCameraGame::destroy_packet as *const (),
]);
// While true, each made shapes packet gets a control block of its own whose last use destroys it.
static OWN_BLOCKS: AtomicBool = AtomicBool::new(false);
// The packets the stand-in destroyed, in order.
static DESTROYED: Mutex<Vec<usize>> = Mutex::new(Vec::new());

// A stand-in game object: its first qword points to its vtable.
#[repr(C)]
struct FakeObject(&'static FakeVtable<{ FreeCameraGame::LEVEL_SLOTS }>);

#[repr(C)]
struct FakeHandler(&'static FakeVtable<3>);

static CONTROL: ControlBlock =
    ControlBlock { vtable: AtomicUsize::new(0), uses: AtomicI32::new(FreeCameraGame::USES), weaks: AtomicI32::new(1) };
static HANDLERS: FakeVtable<3> = FakeVtable([
    FreeCameraGame::camera_stand_in as *const (),
    FreeCameraGame::chunk_stand_in as *const (),
    FreeCameraGame::shapes_stand_in as *const (),
]);
static HANDLER: FakeHandler = FakeHandler(&HANDLERS);
static LEVEL_VTABLE: FakeVtable<{ FreeCameraGame::LEVEL_SLOTS }> = FreeCameraGame::level_vtable();
static LEVEL: FakeObject = FakeObject(&LEVEL_VTABLE);
// The `CameraPresets` the stand-in `getCameraPresets` gives, and the stand-in local player.
static PRESETS: AtomicUsize = AtomicUsize::new(0);
static PLAYER: AtomicUsize = AtomicUsize::new(0);
// What the stand-ins saw, one line each, and the ids `createPacket` was asked for.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CREATED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
// The address of each made packet, in order.
static PACKETS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// Stand-ins for the free camera path, laid out as the derived entries of 26.63 and 70.26 say: a client network
/// handler whose vtable holds the `CameraInstructionPacket` handler (slot 0), the
/// `NetworkChunkPublisherUpdatePacket` handler (slot 1) and the `PrimitiveShapesPacket` handler (slot 2),
/// `MinecraftPackets::createPacket` (a zeroed 188h-byte packet with a shared control block), a local player whose
/// `mLevel` (200h) is a level whose `getCameraPresets` (slot 1Dh) gives a preset list of 130h-byte presets named by a
/// small MSVC string. The camera handler records what the instruction of the packet asks; the shapes handler records
/// each B8h-byte payload of the shape vector at 30h.
pub struct FreeCameraGame;

impl FreeCameraGame {
    /// The network identifier the tests pass with each server handler call; nothing reads it.
    pub const ID: usize = 0x4400;
    /// The packet id of a `PrimitiveShapesPacket`.
    pub const SHAPES_ID: u32 = 328;
    /// The offset of the shape vector in a `PrimitiveShapesPacket`.
    pub const SHAPES_AT: usize = 0x30;
    /// The size of one shape payload.
    pub const PAYLOAD: usize = 0xB8;
    // The size of the game's `NetworkIdentifier`.
    const IDENTIFIER: usize = 0xB0;
    /// The uses of the shared control block with no packet held.
    pub const USES: i32 = 100;
    const LEVEL_SLOTS: usize = 0x1E;
    const PRESETS_SLOT: usize = 0x1D;
    const PACKET_WORDS: usize = 0x188 / 8;
    const PRESET_WORDS: usize = 0x130 / 8;
    const INSTRUCTION: usize = 0x30;
    const LEVEL_AT: usize = 0x200;

    /// The session entries and every entry of the `camera.free` and `camera.instruction_received` rows.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
        let slot = |value: usize| Resolved { kind: EntryKind::VSlot, value: value as u64, recipes_agreeing: 2 };
        let address =
            |kind, address: usize| Resolved { kind, value: address as u64 - Harness::BASE, recipes_agreeing: 2 };
        let mut entries = SessionGame::entries(&[]);
        entries.extend([
            ("LegacyClientNetworkHandler.vtable", address(EntryKind::Global, std::ptr::from_ref(&HANDLERS) as usize)),
            ("ClientNetworkHandler.handle.CameraInstructionPacket", slot(0)),
            ("ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket", slot(1)),
            (
                "MinecraftPackets.createPacket",
                address(EntryKind::Function, Self::create_stand_in as *const () as usize),
            ),
            ("CameraInstructionPacket.size", field(0x188)),
            ("CameraInstructionPacket.mCameraInstruction", field(Self::INSTRUCTION)),
            ("CameraInstruction.mSpline", field(0xB0)),
            ("Level.getCameraPresets", slot(Self::PRESETS_SLOT)),
            ("CameraPreset.size", field(0x130)),
            ("Actor.mLevel", field(Self::LEVEL_AT)),
        ]);
        entries
    }

    /// `entries` and every entry of the `world.shapes` row.
    pub fn shape_entries() -> Vec<(&'static str, Resolved)> {
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
        let slot = Resolved { kind: EntryKind::VSlot, value: 2, recipes_agreeing: 2 };
        let mut entries = Self::entries();
        entries.extend([
            ("ClientNetworkHandler.handle.PrimitiveShapesPacket", slot),
            ("PrimitiveShapesPacket.mShapes", field(Self::SHAPES_AT)),
            ("PrimitiveShapeDataPayload.size", field(Self::PAYLOAD)),
        ]);
        entries
    }

    /// A new local player on the stand-in level, whose preset list holds `names` in order (each at most 15 bytes).
    pub fn reset(names: &[&str]) {
        let presets: &'static mut [[u64; Self::PRESET_WORDS]] =
            Box::leak(names.iter().map(|name| Self::preset(name)).collect::<Vec<_>>().into_boxed_slice());
        let begin = presets.as_mut_ptr() as usize;
        let end = begin + presets.len() * Self::PRESET_WORDS * 8;
        let list: &'static mut [usize; 6] = Box::leak(Box::new([begin, end, end, 0, 0, 0]));
        PRESETS.store(list.as_ptr() as usize, Ordering::SeqCst);
        let player: &'static mut [usize; Self::LEVEL_AT / 8 + 1] = Box::leak(Box::new([0; Self::LEVEL_AT / 8 + 1]));
        player[Self::LEVEL_AT / 8] = std::ptr::from_ref(&LEVEL) as usize;
        PLAYER.store(player.as_ptr() as usize, Ordering::SeqCst);
        CALLS.lock().unwrap().clear();
        CREATED.lock().unwrap().clear();
        PACKETS.lock().unwrap().clear();
    }

    /// The stand-in local player of the last `reset`.
    pub fn player() -> usize {
        PLAYER.load(Ordering::SeqCst)
    }

    /// The stand-in client network handler.
    pub fn handler() -> usize {
        std::ptr::from_ref(&HANDLER) as usize
    }

    /// Calls (through its address) the chunk publisher handler of the stand-in handler, as the server's packet does.
    pub fn chunk_publisher() {
        let handle = black_box(Self::chunk_stand_in as extern "C" fn(*mut c_void, *mut c_void, *mut c_void));
        let packet: &'static mut [u64; 12] = Box::leak(Box::new([0; 12]));
        handle(
            std::ptr::with_exposed_provenance_mut(Self::handler()),
            std::ptr::with_exposed_provenance_mut(Self::ID),
            packet.as_mut_ptr().cast(),
        );
    }

    /// Calls (through its address) the camera instruction handler with a server packet whose set part is present when
    /// `set`, and whose clear part is present and true when `clear`.
    pub fn server_instruction(set: bool, clear: bool) {
        let packet: &'static mut [u64; Self::PACKET_WORDS] = Box::leak(Box::new([0; Self::PACKET_WORDS]));
        let bytes = packet.as_mut_ptr().cast::<u8>();
        // SAFETY: both bytes lie inside the leaked 188h-byte packet.
        unsafe {
            bytes.add(Self::INSTRUCTION + 0xAC).write(u8::from(set));
            bytes.add(Self::INSTRUCTION + 0x148).write(u8::from(clear));
            bytes.add(Self::INSTRUCTION + 0x149).write(u8::from(clear));
        }
        let handle = black_box(Self::camera_stand_in as extern "C" fn(*mut c_void, *mut c_void, *mut c_void));
        handle(
            std::ptr::with_exposed_provenance_mut(Self::handler()),
            std::ptr::with_exposed_provenance_mut(Self::ID),
            bytes.cast(),
        );
    }

    /// The lines of the stand-in handlers so far.
    pub fn log() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    /// The packet ids `createPacket` made so far.
    pub fn created() -> Vec<u32> {
        CREATED.lock().unwrap().clone()
    }

    /// The addresses of the packets `createPacket` made so far.
    pub fn packets() -> Vec<usize> {
        PACKETS.lock().unwrap().clone()
    }

    /// The shape vector (first, last, end) of the packet at `packet`.
    pub fn shape_vector(packet: usize) -> [usize; 3] {
        // SAFETY: `packet` is a packet the stand-in factory made (188h bytes, leaked); the vector lies inside it.
        unsafe { std::ptr::with_exposed_provenance::<[usize; 3]>(packet + Self::SHAPES_AT).read() }
    }

    /// From now on each made shapes packet has a control block of its own (one use): the loader's release destroys the
    /// packet as the game does, and its shape vector frees its block through `HeapProbe::free` with the MSVC
    /// `std::allocator` rule (a block of 4096 bytes or more is a big allocation: the raw pointer before the block, and
    /// 27h more bytes).
    pub fn destroy_shapes_packets() {
        OWN_BLOCKS.store(true, Ordering::SeqCst);
    }

    /// The packets the stand-in destroyed so far.
    pub fn destroyed() -> Vec<usize> {
        DESTROYED.lock().unwrap().clone()
    }

    /// The uses of the shared control block now: `USES` plus one per made packet still held.
    pub fn uses() -> i32 {
        CONTROL.uses.load(Ordering::SeqCst)
    }

    const fn level_vtable() -> FakeVtable<{ Self::LEVEL_SLOTS }> {
        let mut slots = [Self::unused as *const (); Self::LEVEL_SLOTS];
        slots[Self::PRESETS_SLOT] = Self::presets_stand_in as *const ();
        FakeVtable(slots)
    }

    // A preset named `name` in a small MSVC string: the bytes inline, then the size and the capacity 15.
    fn preset(name: &str) -> [u64; Self::PRESET_WORDS] {
        assert!(name.len() <= 15);
        let mut text = [0u8; 16];
        text[..name.len()].copy_from_slice(name.as_bytes());
        let mut preset = [0u64; Self::PRESET_WORDS];
        preset[0] = u64::from_le_bytes(text[..8].try_into().unwrap());
        preset[1] = u64::from_le_bytes(text[8..].try_into().unwrap());
        preset[2] = name.len() as u64;
        preset[3] = 15;
        preset
    }

    #[inline(never)]
    extern "C" fn unused(this: *mut c_void) -> usize {
        black_box(this);
        0
    }

    #[inline(never)]
    extern "C" fn presets_stand_in(this: *mut c_void) -> usize {
        black_box(this);
        PRESETS.load(Ordering::SeqCst)
    }

    #[inline(never)]
    extern "C" fn create_stand_in(out: *mut [usize; 2], id: u32) -> *mut [usize; 2] {
        CREATED.lock().unwrap().push(black_box(id));
        let packet: &'static mut [u64; Self::PACKET_WORDS] = Box::leak(Box::new([0; Self::PACKET_WORDS]));
        PACKETS.lock().unwrap().push(packet.as_ptr() as usize);
        let control = if id == Self::SHAPES_ID && OWN_BLOCKS.load(Ordering::SeqCst) {
            let control: &'static PacketControl = Box::leak(Box::new(PacketControl {
                vtable: &PACKET_CONTROL_VTABLE,
                uses: AtomicI32::new(1),
                weaks: AtomicI32::new(1),
                packet: packet.as_ptr() as usize,
            }));
            std::ptr::from_ref(control) as usize
        } else {
            CONTROL.uses.fetch_add(1, Ordering::SeqCst);
            std::ptr::from_ref(&CONTROL) as usize
        };
        // SAFETY: `out` is the 16-byte buffer of the shared_ptr the loader gives.
        unsafe { out.write([packet.as_mut_ptr() as usize, control]) };
        out
    }

    // `_Destroy` of a shapes packet's control block: the packet's shape vector frees its block as
    // `std::allocator::deallocate` does, with the bytes of its capacity.
    #[inline(never)]
    extern "C" fn destroy_packet(control: *mut PacketControl) {
        // SAFETY: the loader calls this through the control block the stand-in factory made.
        let packet = unsafe { (*control).packet };
        DESTROYED.lock().unwrap().push(packet);
        let [first, _, end] = Self::shape_vector(packet);
        if first == 0 {
            return;
        }
        let bytes = end - first;
        // SAFETY: the vector's block came from the probe through the loader's `MsvcHeap`; a big allocation keeps the
        // raw pointer in the 8 bytes before the block.
        unsafe {
            if bytes >= 4096 {
                let raw = std::ptr::with_exposed_provenance::<usize>(first - 8).read_unaligned();
                HeapProbe::free(std::ptr::with_exposed_provenance_mut(raw), bytes + 0x27);
            } else {
                HeapProbe::free(std::ptr::with_exposed_provenance_mut(first), bytes);
            }
        }
    }

    // `_Delete_this`: the stand-in control block is leaked.
    #[inline(never)]
    extern "C" fn delete_control(control: *mut PacketControl) {
        black_box(control);
    }

    #[inline(never)]
    extern "C" fn chunk_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        black_box(packet);
        CALLS.lock().unwrap().push(format!("chunk handler={:#x} id={:#x}", this.addr(), source.addr()));
    }

    #[inline(never)]
    extern "C" fn camera_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        let instruction = packet.cast::<u8>().wrapping_add(Self::INSTRUCTION);
        // SAFETY: every read lies inside the 188h-byte packet the caller passes.
        let line = unsafe {
            let byte = |at: usize| instruction.add(at).read();
            let float = |at: usize| instruction.add(at).cast::<f32>().read_unaligned();
            let index = instruction.add(0x50).cast::<u32>().read_unaligned();
            format!(
                "camera handler={:#x} id={} set={} index={} pos={},{},{}/{} rot={},{}/{} clear={}/{}",
                this.addr(),
                Self::identifier(source),
                byte(0xAC),
                index,
                float(0x60),
                float(0x64),
                float(0x68),
                byte(0x6C),
                float(0x70),
                float(0x74),
                byte(0x78),
                byte(0x148),
                byte(0x149)
            )
        };
        CALLS.lock().unwrap().push(black_box(line));
    }

    // How the handler sees its identifier: `server` for the address the tests pass, `invalid` for a readable identifier
    // whose bytes are zero but the type `mType` (A8h, i32) of 4, `null`, or `dirty`.
    fn identifier(source: *mut c_void) -> String {
        if source.addr() == Self::ID {
            return "server".to_string();
        }
        if source.is_null() {
            return "null".to_string();
        }
        // SAFETY: any other identifier is the loader's own buffer of the identifier's size.
        let bytes = unsafe { std::slice::from_raw_parts(source.cast::<u8>(), Self::IDENTIFIER) };
        let kind = i32::from_le_bytes(bytes[0xA8..0xAC].try_into().unwrap());
        let rest_zero = bytes.iter().enumerate().all(|(at, byte)| (0xA8..0xAC).contains(&at) || *byte == 0);
        if kind == 4 && rest_zero { "invalid".to_string() } else { "dirty".to_string() }
    }

    // One payload: the network id, then each option as value/engaged, then the extra data index and data.
    fn payload_line(payload: *const u8) -> String {
        // SAFETY: the caller passes a payload inside the shape vector of a live packet (B8h bytes).
        let (byte, float) =
            unsafe { (|at: usize| payload.add(at).read(), |at: usize| payload.add(at).cast::<f32>().read_unaligned()) };
        let vec3 = |at: usize| format!("{},{},{}", float(at), float(at + 4), float(at + 8));
        let index = byte(0xB0);
        let data = match index {
            1 => format!(
                " end={}/{} head={}/{},{}/{} segments={}/{}",
                vec3(0x70),
                byte(0x7C),
                float(0x80),
                byte(0x84),
                float(0x88),
                byte(0x8C),
                byte(0x90),
                byte(0x91)
            ),
            3 | 4 => format!(" end={}", vec3(0x70)),
            _ => String::new(),
        };
        // SAFETY: as above.
        let (network_id, dimension) =
            unsafe { (payload.cast::<u64>().read_unaligned(), payload.add(0x50).cast::<i32>().read_unaligned()) };
        format!(
            "shape net={network_id:#x} type={}/{} loc={}/{} rot={}/{} scale={}/{} color={:.3},{:.3},{:.3},{:.3}/{} \
             dim={dimension}/{} time={} attached={} distance={} extra={index}{data}",
            byte(0x08),
            byte(0x09),
            vec3(0x0C),
            byte(0x18),
            vec3(0x1C),
            byte(0x28),
            float(0x2C),
            byte(0x30),
            float(0x34),
            float(0x38),
            float(0x3C),
            float(0x40),
            byte(0x44),
            byte(0x54),
            byte(0x4C),
            byte(0x60),
            byte(0x6C),
        )
    }

    #[inline(never)]
    extern "C" fn shapes_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
        let vector = Self::shape_vector(packet.addr());
        let count = (vector[1] - vector[0]) / Self::PAYLOAD;
        let mut lines =
            vec![format!("shapes handler={:#x} id={} count={count}", this.addr(), Self::identifier(source))];
        for index in 0..count {
            lines.push(Self::payload_line(std::ptr::with_exposed_provenance(vector[0] + index * Self::PAYLOAD)));
        }
        CALLS.lock().unwrap().extend(black_box(lines));
    }
}
