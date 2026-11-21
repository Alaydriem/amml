use std::ffi::c_void;
use std::sync::atomic::{AtomicI32, Ordering};

use amml_abi::AmmlStatus;

/// `std::optional<Option*> IOptionRegistry::getIfValid(OptionID)`: rcx the registry, rdx the 16-byte optional
/// it fills (the `Option*` at +0, the engaged flag at +8), r8d the id; rax the same out pointer.
type GetIfValidFn = unsafe extern "C" fn(*mut c_void, *mut [usize; 2], i32) -> *mut [usize; 2];

/// The game option `pause_menu_on_focus_lost`: `MinecraftGame::onAppFocusLost` opens the pause screen only
/// when it is on.
///
/// Its id is not the same on every build (0x32F on 26.63, 0x336 on 1.26.70.26), so the first read finds it by
/// its save tag: every id up to `MAX_ID` is asked through `IOptionRegistry::getIfValid`, and exactly one
/// option must carry the tag. Each option the scan reads must give back its own id in `Option::Impl::mID`, so a
/// build whose `Option::Impl` layout moved is refused, not misread. The id found is kept for the process.
///
/// The value is read the way the game reads it: from the option, follow the override chain
/// (`Impl + OVERRIDE_AT`) to its end, require a bool option (`mOptionType` 0) and read `BoolOption::mValue`.
pub struct PauseOption {
    id: AtomicI32,
}

impl PauseOption {
    pub const SAVE_TAG: &'static str = "pause_menu_on_focus_lost";
    const UNKNOWN: i32 = -1;
    // The highest id the scan asks for; the game has about 0x340 options.
    const MAX_ID: i32 = 0x800;
    const MAX_OVERRIDES: usize = 16;
    const BOOL_TYPE: i32 = 0;
    // Offsets in an `Option`.
    const IMPL_AT: usize = 0x8;
    const BOOL_VALUE_AT: usize = 0x10;
    // Offsets in its `Option::Impl`.
    const SAVE_TAG_AT: usize = 0x188;
    const ID_AT: usize = 0x1CC;
    const TYPE_AT: usize = 0x1D4;
    const OVERRIDE_AT: usize = 0x238;
    // An MSVC `std::string` keeps up to 15 bytes in place; a larger capacity means a heap pointer at +0.
    const SMALL_CAPACITY: usize = 15;

    pub fn new() -> Self {
        Self { id: AtomicI32::new(Self::UNKNOWN) }
    }

    /// The option's value on the registry `options`, through `get_if_valid` (the registry's `getIfValid`).
    /// `NOT_AVAILABLE`, with a log line, when no option or more than one carries the tag, when the layout check
    /// fails, or when the option is not a bool option.
    ///
    /// # Safety
    /// `options` is the live option registry of the primary client, used on the game thread, and
    /// `get_if_valid` is its `getIfValid` function.
    pub unsafe fn read(&self, options: usize, get_if_valid: usize) -> Result<bool, AmmlStatus> {
        // SAFETY: the caller guarantees that `get_if_valid` is `getIfValid`.
        let get = unsafe { std::mem::transmute::<usize, GetIfValidFn>(get_if_valid) };
        // SAFETY: the caller guarantees `options` and `get`.
        let id = unsafe { self.id(options, get) }?;
        // SAFETY: as above.
        let option = unsafe { Self::option(options, get, id) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `option` is a live option that the registry gave.
        let option = unsafe { Self::effective(option) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `effective` gave a live option with a non-null Impl.
        let kind = unsafe { Self::read_at::<i32>(Self::read_at::<usize>(option, Self::IMPL_AT), Self::TYPE_AT) };
        if kind != Self::BOOL_TYPE {
            curia::warn!("the pause option is not a bool option", { id: id, option_type: kind });
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        // SAFETY: a bool option is a `BoolOption`, which holds its value byte at `BOOL_VALUE_AT`.
        Ok(unsafe { Self::read_at::<u8>(option, Self::BOOL_VALUE_AT) } != 0)
    }

    // The id of the option, from the cache or a scan.
    //
    // Safety: as `read`.
    unsafe fn id(&self, options: usize, get: GetIfValidFn) -> Result<i32, AmmlStatus> {
        let known = self.id.load(Ordering::Acquire);
        if known != Self::UNKNOWN {
            return Ok(known);
        }
        let mut found: Vec<i32> = Vec::new();
        for id in 0..=Self::MAX_ID {
            // SAFETY: the caller guarantees `options` and `get`.
            let Some(option) = (unsafe { Self::option(options, get, id) }) else {
                continue;
            };
            // SAFETY: `option` is a live option that the registry gave.
            let implementation = unsafe { Self::read_at::<usize>(option, Self::IMPL_AT) };
            if implementation == 0 {
                continue;
            }
            // SAFETY: a non-null `mImpl` points to the option's `Option::Impl`.
            let own_id = unsafe { Self::read_at::<i32>(implementation, Self::ID_AT) };
            if own_id != id {
                curia::warn!("pause option not available: an option does not hold its own id", {
                    id: id,
                    found: own_id
                });
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            // SAFETY: as above; the Impl holds `mSaveTag`.
            if unsafe { Self::has_tag(implementation) } {
                found.push(id);
            }
        }
        match found.as_slice() {
            [id] => {
                self.id.store(*id, Ordering::Release);
                curia::info!("pause option found", { id: format!("{id:#x}"), save_tag: Self::SAVE_TAG });
                Ok(*id)
            }
            _ => {
                curia::warn!("pause option not available", { save_tag: Self::SAVE_TAG, matches: found.len() });
                Err(AmmlStatus::NOT_AVAILABLE)
            }
        }
    }

    // The option with `id`; `None` for an empty optional or a null pointer.
    //
    // Safety: as `read`.
    unsafe fn option(options: usize, get: GetIfValidFn, id: i32) -> Option<usize> {
        let mut out = [0usize; 2];
        // SAFETY: the caller guarantees `get` and `options`; `out` has room for the 16-byte optional.
        unsafe { get(std::ptr::with_exposed_provenance_mut(options), &raw mut out, id) };
        let engaged = out[1] & 0xFF != 0;
        (engaged && out[0] != 0).then_some(out[0])
    }

    // The end of the override chain of `option`; `None` for a null Impl or a chain longer than
    // `MAX_OVERRIDES`.
    //
    // Safety: `option` is a live option.
    unsafe fn effective(mut option: usize) -> Option<usize> {
        for _ in 0..=Self::MAX_OVERRIDES {
            // SAFETY: `option` is a live option (the first one, or an override the game linked).
            let implementation = unsafe { Self::read_at::<usize>(option, Self::IMPL_AT) };
            if implementation == 0 {
                return None;
            }
            // SAFETY: a non-null `mImpl` points to the option's `Option::Impl`.
            let next = unsafe { Self::read_at::<usize>(implementation, Self::OVERRIDE_AT) };
            if next == 0 {
                return Some(option);
            }
            option = next;
        }
        curia::warn!("the pause option override chain is too long", { max: Self::MAX_OVERRIDES });
        None
    }

    // True when the `mSaveTag` of `implementation` is `SAVE_TAG`. The text pointer is read only when the size
    // and capacity are those of a heap string of that length.
    //
    // Safety: `implementation` is a live `Option::Impl`.
    unsafe fn has_tag(implementation: usize) -> bool {
        let tag = implementation + Self::SAVE_TAG_AT;
        // SAFETY: the Impl holds the 32-byte `mSaveTag` at `SAVE_TAG_AT`: the text or its pointer, size, capacity.
        let (text, size, capacity) = unsafe {
            (Self::read_at::<usize>(tag, 0), Self::read_at::<usize>(tag, 16), Self::read_at::<usize>(tag, 24))
        };
        let wanted = Self::SAVE_TAG.as_bytes();
        if size != wanted.len() || capacity < size {
            return false;
        }
        let bytes = if capacity > Self::SMALL_CAPACITY {
            if text == 0 {
                return false;
            }
            // SAFETY: a heap-mode string points to at least `size` bytes.
            unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(text), size) }
        } else {
            // SAFETY: a small string holds its `size` bytes in place.
            unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(tag), size) }
        };
        bytes == wanted
    }

    // Safety: `base + offset` is readable game memory that holds a `T`.
    unsafe fn read_at<T: Copy>(base: usize, offset: usize) -> T {
        // SAFETY: the caller guarantees the address.
        unsafe { std::ptr::with_exposed_provenance::<T>(base.wrapping_add(offset)).read_unaligned() }
    }
}

impl Default for PauseOption {
    fn default() -> Self {
        Self::new()
    }
}
