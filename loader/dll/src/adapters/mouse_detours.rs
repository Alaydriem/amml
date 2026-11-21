use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, EightArgMouseFeed, SessionHook, SevenArgMouseFeed};

/// `void MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, bool
/// forceMotionlessPointer)` (26.6x): rcx the device, dl the action, r8b the data, r9w x; y, dx, dy and the
/// motionless flag in the stack slots at +28h, +30h, +38h and +40h. Every register and slot is passed on
/// whole.
type SevenArgFeedFn = unsafe extern "C" fn(*mut c_void, u64, u64, u64, u64, u64, u64, u64);
/// `void MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, int
/// controllerId, bool forceMotionlessPointer)` (70.x): as `SevenArgFeedFn`, with the 32-bit controller id
/// in the slot at +40h and the motionless flag moved to +48h.
type EightArgFeedFn = unsafe extern "C" fn(*mut c_void, u64, u64, u64, u64, u64, u64, u64, u64);

static SEVEN_ARG_FEED: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static EIGHT_ARG_FEED: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the game's mouse feed, and the loader's own calls past them.
///
/// Each detour runs the real mouse path inside `Boundary::run` (a panic there sends the feed on
/// unchanged) and then calls the original exactly once, with the registers and slots it received and the
/// listeners' x and y, unless the path holds the feed back. The loader's own calls (`send_seven_arg` and
/// `send_eight_arg`) call the original slot directly, so they raise no event and do not touch the button
/// table.
pub struct MouseDetours;

impl MouseDetours {
    /// The hook of the `seven_arg_feed` shape.
    pub fn seven_arg_hook() -> SessionHook {
        SessionHook {
            entry: SevenArgMouseFeed::FEED_ENTRY,
            vtable: None,
            detour: Self::seven_arg_feed as SevenArgFeedFn as *const c_void,
            original: &SEVEN_ARG_FEED,
        }
    }

    /// The hook of the `eight_arg_feed` shape.
    pub fn eight_arg_hook() -> SessionHook {
        SessionHook {
            entry: EightArgMouseFeed::FEED_ENTRY,
            vtable: None,
            detour: Self::eight_arg_feed as EightArgFeedFn as *const c_void,
            original: &EIGHT_ARG_FEED,
        }
    }

    /// Calls the seven-argument `feed(device, action, data, x, y, 0, 0, false)` past the loader's detour;
    /// false while the hook is not installed.
    ///
    /// # Safety
    /// `device` is the game's live `MouseDevice`, and the call runs on the thread of the game's own feed.
    pub(crate) unsafe fn send_seven_arg(device: *mut c_void, action: u8, data: i8, x: i16, y: i16) -> bool {
        let Some(original) = Self::installed(&SEVEN_ARG_FEED) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `SevenArgFeedFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, SevenArgFeedFn>(original) };
        let (action, data, x, y) = Self::low_bits(action, data, x, y);
        // SAFETY: the caller guarantees `device` and the thread; every value fits the width the game reads.
        unsafe { original(device, action, data, x, y, 0, 0, 0) };
        true
    }

    /// Calls the eight-argument `feed(device, action, data, x, y, 0, 0, controller, false)` past the loader's
    /// detour; false while the hook is not installed.
    ///
    /// # Safety
    /// As `send_seven_arg`.
    pub(crate) unsafe fn send_eight_arg(
        device: *mut c_void,
        action: u8,
        data: i8,
        x: i16,
        y: i16,
        controller: i32,
    ) -> bool {
        let Some(original) = Self::installed(&EIGHT_ARG_FEED) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `EightArgFeedFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, EightArgFeedFn>(original) };
        let (action, data, x, y) = Self::low_bits(action, data, x, y);
        let controller = u64::from(controller as u32);
        // SAFETY: as `send_seven_arg`.
        unsafe { original(device, action, data, x, y, 0, 0, controller, 0) };
        true
    }

    // The next hook or trampoline in `slot`; `None` while it is null.
    fn installed(slot: &AtomicPtr<c_void>) -> Option<*mut c_void> {
        Some(slot.load(Ordering::Acquire)).filter(|address| !address.is_null())
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the
        // caller guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    // The values as the zero-extended registers or slots the game reads them from.
    fn low_bits(action: u8, data: i8, x: i16, y: i16) -> (u64, u64, u64, u64) {
        (u64::from(action), u64::from(data as u8), u64::from(x as u16), u64::from(y as u16))
    }

    // The register or slot `whole` with its low 16 bits set to `value`.
    fn with_short(whole: u64, value: i16) -> u64 {
        (whole & !0xFFFF) | u64::from(value as u16)
    }

    // The real path's verdict on one feed: the x and y registers to pass on, or `None` to hold it back. A
    // panic in the path passes the feed on unchanged.
    fn verdict(
        real: impl FnOnce(usize, u8, i8, i16, i16, i16, i16) -> Option<(i16, i16)>,
        this: *mut c_void,
        regs: [u64; 6],
    ) -> Option<(u64, u64)> {
        let [action, data, x, y, dx, dy] = regs;
        Boundary::run(Some((x, y)), || {
            let moved = real(
                this.expose_provenance(),
                action as u8,
                data as u8 as i8,
                x as u16 as i16,
                y as u16 as i16,
                dx as u16 as i16,
                dy as u16 as i16,
            )?;
            Some((Self::with_short(x, moved.0), Self::with_short(y, moved.1)))
        })
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn seven_arg_feed(
        this: *mut c_void,
        action: u64,
        data: u64,
        x: u64,
        y: u64,
        dx: u64,
        dy: u64,
        motionless: u64,
    ) {
        let real = |this, action, data, x, y, dx, dy| {
            SevenArgMouseFeed::with(Some((x, y)), |feed| feed.real(this, action, data, x, y, dx, dy))
        };
        let Some((x, y)) = Self::verdict(real, this, [action, data, x, y, dx, dy]) else {
            return;
        };
        // SAFETY: the hook on the seven-argument feed is installed, so its slot holds a `SevenArgFeedFn`.
        let original = unsafe { Self::original::<SevenArgFeedFn>(&SEVEN_ARG_FEED) };
        // SAFETY: the game's own registers and slots, with x and y as the listeners left them.
        unsafe { original(this, action, data, x, y, dx, dy, motionless) };
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn eight_arg_feed(
        this: *mut c_void,
        action: u64,
        data: u64,
        x: u64,
        y: u64,
        dx: u64,
        dy: u64,
        controller: u64,
        motionless: u64,
    ) {
        let real = |this, action, data, x, y, dx, dy| {
            EightArgMouseFeed::with(Some((x, y)), |feed| feed.real(this, action, data, x, y, dx, dy))
        };
        let Some((x, y)) = Self::verdict(real, this, [action, data, x, y, dx, dy]) else {
            return;
        };
        // SAFETY: the hook on the eight-argument feed is installed, so its slot holds an `EightArgFeedFn`.
        let original = unsafe { Self::original::<EightArgFeedFn>(&EIGHT_ARG_FEED) };
        // SAFETY: the game's own registers and slots, with x and y as the listeners left them.
        unsafe { original(this, action, data, x, y, dx, dy, controller, motionless) };
    }
}
