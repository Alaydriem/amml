use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU8, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, FakeVtable};

// The control block of the stand-in input handler, laid out as the MSVC `_Ref_count_base`: vtable, uses, weaks. The
// uses start high, so a release never reaches zero and the vtable is never called.
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
// A stand-in MinecraftInputHandler of 40h bytes: `mInputHandler` at `HANDLER_AT`.
static HANDLER: [AtomicUsize; 8] = [const { AtomicUsize::new(0) }; 8];
// A stand-in InputHandler of 180h bytes: the `mAnyInputHandler` std::function at `ANY_INPUT_AT`.
static INPUT_HANDLER: [AtomicUsize; 48] = [const { AtomicUsize::new(0) }; 48];
// The callable of the std::function: only its vtable pointer.
static CALLABLE: AtomicUsize = AtomicUsize::new(0);
// The callable's vtable: 2 `_Do_call`.
static CALLABLE_VTABLE: FakeVtable<3> = FakeVtable([
    ActivityGame::unused as *const (),
    ActivityGame::unused as *const (),
    ActivityGame::do_call as *const (),
]);
// The client each `_Do_call` got.
static CALLS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// Stand-ins for the game's any-input chain that `note_activity` walks: `getMinecraftInput` (slot 1 of the stand-in
/// client vtable; it takes one use of the control block, as the game does), a MinecraftInputHandler with its
/// `mInputHandler`, and an InputHandler whose `mAnyInputHandler` records each call.
pub struct ActivityGame;

impl ActivityGame {
    /// The derived `MinecraftInputHandler.mInputHandler` and `InputHandler.mAnyInputHandler` of the stand-ins (the
    /// values of 26.63 and 70.26).
    pub const HANDLER_AT: usize = 0x20;
    pub const ANY_INPUT_AT: usize = 0xC0;

    /// The client entries and the three any-input entries; `without` names entries to leave out. The chain starts
    /// valid: the flag is 1, the InputHandler is set and its handler is set.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::set_valid(true);
        Self::set_input_handler(true);
        Self::set_handler(true);
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 1 };
        let mut entries = ClientGame::entries(&[]);
        entries.extend([
            ("ClientInstance.getMinecraftInput", Resolved { kind: EntryKind::VSlot, value: 1, recipes_agreeing: 2 }),
            ("MinecraftInputHandler.mInputHandler", field(Self::HANDLER_AT)),
            ("InputHandler.mAnyInputHandler", field(Self::ANY_INPUT_AT)),
        ]);
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    /// The flag byte of the control block: 1 while the input handler lives.
    pub fn set_valid(valid: bool) {
        FLAG.store(u8::from(valid), Ordering::SeqCst);
    }

    pub fn set_input_handler(set: bool) {
        let address = if set { std::ptr::from_ref(&INPUT_HANDLER) as usize } else { 0 };
        HANDLER[Self::HANDLER_AT / 8].store(address, Ordering::SeqCst);
    }

    /// Whether the std::function holds a callable (a handler is registered).
    pub fn set_handler(set: bool) {
        CALLABLE.store(std::ptr::from_ref(&CALLABLE_VTABLE) as usize, Ordering::SeqCst);
        let callable = if set { std::ptr::from_ref(&CALLABLE) as usize } else { 0 };
        INPUT_HANDLER[(Self::ANY_INPUT_AT + 0x38) / 8].store(callable, Ordering::SeqCst);
    }

    /// The clients the any-input handler was called with, in order.
    pub fn calls() -> Vec<usize> {
        CALLS.lock().unwrap().clone()
    }

    /// The use count of the control block.
    pub fn uses() -> i32 {
        CONTROL.uses.load(Ordering::SeqCst)
    }

    /// `NonOwnerPointer<MinecraftInputHandler> getMinecraftInput() const`: rcx the client, rdx the result buffer;
    /// writes the flag pointer, the control block and the handler, and takes one use.
    #[inline(never)]
    pub extern "C" fn minecraft_input(this: *mut c_void, out: *mut [usize; 3]) -> *mut [usize; 3] {
        black_box(this);
        CONTROL.uses.fetch_add(1, Ordering::SeqCst);
        let value = [
            std::ptr::from_ref(&FLAG) as usize,
            std::ptr::from_ref(&CONTROL) as usize,
            std::ptr::from_ref(&HANDLER) as usize,
        ];
        // SAFETY: the caller passes a writable 24-byte buffer.
        unsafe { out.write(value) };
        out
    }

    #[inline(never)]
    extern "C" fn do_call(this: *mut c_void, client: *mut c_void) {
        black_box(this);
        CALLS.lock().unwrap().push(client as usize);
    }

    extern "C" fn unused(this: *mut c_void) {
        black_box(this);
    }
}
