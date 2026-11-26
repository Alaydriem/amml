use std::sync::atomic::{AtomicI32, Ordering};

use amml_abi::AmmlStatus;

use crate::{GetIfValidFn, IntOptionLayout, OptionScan, OptionScanError};

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
    const MAX_OVERRIDES: usize = 16;
    const BOOL_TYPE: i32 = 0;
    // Offset of the value in a `BoolOption`.
    const BOOL_VALUE_AT: usize = 0x10;
    // Offset of the override pointer in an `Option::Impl`.
    const OVERRIDE_AT: usize = 0x238;

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
        let kind = unsafe {
            Self::read_at::<i32>(Self::read_at::<usize>(option, IntOptionLayout::IMPL_AT), IntOptionLayout::TYPE_AT)
        };
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
        // SAFETY: the caller guarantees `options` and `get`.
        match unsafe { OptionScan::find(options, get, Self::SAVE_TAG) } {
            Ok(id) => {
                self.id.store(id, Ordering::Release);
                curia::info!("pause option found", { id: format!("{id:#x}"), save_tag: Self::SAVE_TAG });
                Ok(id)
            }
            Err(OptionScanError::IdMismatch { id, found }) => {
                curia::warn!("pause option not available: an option does not hold its own id", {
                    id: id,
                    found: found
                });
                Err(AmmlStatus::NOT_AVAILABLE)
            }
            Err(OptionScanError::Matches { matches }) => {
                curia::warn!("pause option not available", { save_tag: Self::SAVE_TAG, matches: matches });
                Err(AmmlStatus::NOT_AVAILABLE)
            }
        }
    }

    // The option with `id`; `None` for an empty optional or a null pointer.
    //
    // Safety: as `read`.
    unsafe fn option(options: usize, get: GetIfValidFn, id: i32) -> Option<usize> {
        // SAFETY: the caller's guarantee is passed on.
        unsafe { OptionScan::option(options, get, id) }
    }

    // The end of the override chain of `option`; `None` for a null Impl or a chain longer than
    // `MAX_OVERRIDES`.
    //
    // Safety: `option` is a live option.
    unsafe fn effective(mut option: usize) -> Option<usize> {
        for _ in 0..=Self::MAX_OVERRIDES {
            // SAFETY: `option` is a live option (the first one, or an override the game linked).
            let implementation = unsafe { Self::read_at::<usize>(option, IntOptionLayout::IMPL_AT) };
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
