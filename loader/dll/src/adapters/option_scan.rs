use std::ffi::c_void;

use crate::{IntOptionLayout, OptionScanError};

/// `std::optional<Option*> IOptionRegistry::getIfValid(OptionID)`: rcx the registry, rdx the 16-byte optional
/// it fills (the `Option*` at +0, the engaged flag at +8), r8d the id; rax the same out pointer.
pub type GetIfValidFn = unsafe extern "C" fn(*mut c_void, *mut [usize; 2], i32) -> *mut [usize; 2];

/// Finds an option of the registry by its save tag: every id up to `MAX_ID` is asked through
/// `IOptionRegistry::getIfValid`, and exactly one option must carry the tag. Each option the scan reads must give back
/// its own id in `Option::Impl::mID`, so a build whose `Option::Impl` layout moved is refused, not misread.
pub struct OptionScan;

impl OptionScan {
    /// The highest id the scan asks for; the game has about 0x340 options.
    pub const MAX_ID: i32 = 0x800;
    // An MSVC `std::string` keeps up to 15 bytes in place; a larger capacity means a heap pointer at +0.
    const SMALL_CAPACITY: usize = 15;
    // The longest save tag `save_tag` reads.
    const MAX_TAG: usize = 64;

    /// The id of the one option whose save tag is `tag`.
    ///
    /// # Safety
    /// `options` is the live option registry of the primary client, used on the game thread, and `get` is its
    /// `getIfValid` function.
    pub unsafe fn find(options: usize, get: GetIfValidFn, tag: &str) -> Result<i32, OptionScanError> {
        let mut found: Vec<i32> = Vec::new();
        for id in 0..=Self::MAX_ID {
            // SAFETY: the caller guarantees `options` and `get`.
            let Some(option) = (unsafe { Self::option(options, get, id) }) else {
                continue;
            };
            // SAFETY: `option` is a live option that the registry gave.
            let Some(own_id) = (unsafe { IntOptionLayout::id(option) }) else {
                continue;
            };
            if own_id != id {
                return Err(OptionScanError::IdMismatch { id, found: own_id });
            }
            // SAFETY: the option has an Impl (its id was read), which holds `mSaveTag`.
            if unsafe { Self::has_tag(IntOptionLayout::implementation(option), tag) } {
                found.push(id);
            }
        }
        match found.as_slice() {
            [id] => Ok(*id),
            _ => Err(OptionScanError::Matches { matches: found.len() }),
        }
    }

    /// The option with `id`; `None` for an empty optional or a null pointer.
    ///
    /// # Safety
    /// `options` is the live option registry of the primary client, used on the game thread, and `get` is its
    /// `getIfValid` function.
    pub unsafe fn option(options: usize, get: GetIfValidFn, id: i32) -> Option<usize> {
        let mut out = [0usize; 2];
        // SAFETY: the caller guarantees `get` and `options`; `out` has room for the 16-byte optional.
        unsafe { get(std::ptr::with_exposed_provenance_mut(options), &raw mut out, id) };
        let engaged = out[1] & 0xFF != 0;
        (engaged && out[0] != 0).then_some(out[0])
    }

    /// The save tag of `option` as text (lossy), for a log line; empty when the string does not look like a string of
    /// at most `MAX_TAG` bytes.
    ///
    /// # Safety
    /// `option` is a live `Option` with an Impl.
    pub unsafe fn save_tag(option: usize) -> String {
        // SAFETY: the caller guarantees the option.
        let implementation = unsafe { IntOptionLayout::implementation(option) };
        if implementation == 0 {
            return String::new();
        }
        let tag = implementation + IntOptionLayout::SAVE_TAG_AT;
        // SAFETY: the Impl holds the 32-byte `mSaveTag` at `SAVE_TAG_AT`: the text or its pointer, size, capacity.
        let (text, size, capacity) = unsafe {
            (
                IntOptionLayout::read_at::<usize>(tag, 0),
                IntOptionLayout::read_at::<usize>(tag, 16),
                IntOptionLayout::read_at::<usize>(tag, 24),
            )
        };
        if size > Self::MAX_TAG || capacity < size {
            return String::new();
        }
        // SAFETY: the size and capacity are those of a string of `size` bytes, in place or on the heap.
        unsafe { Self::bytes(tag, text, size, capacity) }
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default()
    }

    // True when the `mSaveTag` of `implementation` is `tag`. The text pointer is read only when the size and capacity
    // are those of a heap string of that length.
    //
    // Safety: `implementation` is a live `Option::Impl`.
    unsafe fn has_tag(implementation: usize, tag: &str) -> bool {
        let at = implementation + IntOptionLayout::SAVE_TAG_AT;
        // SAFETY: the Impl holds the 32-byte `mSaveTag` at `SAVE_TAG_AT`: the text or its pointer, size, capacity.
        let (text, size, capacity) = unsafe {
            (
                IntOptionLayout::read_at::<usize>(at, 0),
                IntOptionLayout::read_at::<usize>(at, 16),
                IntOptionLayout::read_at::<usize>(at, 24),
            )
        };
        let wanted = tag.as_bytes();
        if size != wanted.len() || capacity < size {
            return false;
        }
        // SAFETY: the size and capacity are those of a string of `size` bytes.
        unsafe { Self::bytes(at, text, size, capacity) }.is_some_and(|bytes| bytes == wanted)
    }

    // The `size` bytes of the string at `at` whose heap pointer is `text`; `None` for a null heap pointer.
    //
    // Safety: `at` holds a string of `size` bytes and `capacity`, with `capacity >= size`.
    unsafe fn bytes(at: usize, text: usize, size: usize, capacity: usize) -> Option<Vec<u8>> {
        let start = if capacity > Self::SMALL_CAPACITY {
            if text == 0 {
                return None;
            }
            text
        } else {
            at
        };
        // SAFETY: a heap-mode string points to at least `size` bytes, and a small string holds them in place.
        Some(unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(start), size) }.to_vec())
    }
}
