use amml::ImportHook;

use crate::support::FolderProbe;

// SAFETY (helper): every base passed in is a module the test keeps mapped, or a zeroed buffer
// that the call refuses before reading anything but its first bytes.
struct Patch;

impl Patch {
    fn at(base: usize, function: &'static str, replacement: usize) -> Result<Vec<amml::ImportSlot>, amml::HookError> {
        // SAFETY: see the helper note above; the replacements are plain `extern "system"` functions.
        unsafe { ImportHook::patch(base, function, replacement) }
    }
}

struct Replacements;

impl Replacements {
    const TEMP: &'static str = r"X:\temp\";

    extern "system" fn temp_path(size: u32, buffer: *mut u16) -> u32 {
        let wide: Vec<u16> = Self::TEMP.encode_utf16().chain(std::iter::once(0)).collect();
        if (size as usize) < wide.len() {
            return wide.len() as u32;
        }
        // SAFETY: the probe passes a buffer of `size` units, and `wide` fits in it.
        unsafe { std::ptr::copy_nonoverlapping(wide.as_ptr(), buffer, wide.len()) };
        (wide.len() - 1) as u32
    }

    extern "system" fn env(_name: *const u16, _buffer: *mut u16, _size: u32) -> u32 {
        0
    }
}

#[test]
fn patch_redirects_only_the_named_import() {
    let probe = FolderProbe::load();
    let real_appdata = probe.env_appdata();
    let real_temp = probe.temp_path();

    let slots = Patch::at(probe.base(), "GetTempPathW", Replacements::temp_path as *const () as usize).unwrap();

    assert!(!slots.is_empty());
    assert_eq!(probe.temp_path(), Replacements::TEMP);
    assert_ne!(real_temp, Replacements::TEMP);
    assert_eq!(probe.env_appdata(), real_appdata);
}

#[test]
fn restore_puts_the_original_back() {
    let probe = FolderProbe::load();
    let real_temp = probe.temp_path();
    let slots = Patch::at(probe.base(), "GetTempPathW", Replacements::temp_path as *const () as usize).unwrap();
    assert_eq!(probe.temp_path(), Replacements::TEMP);

    for slot in &slots {
        slot.restore().unwrap();
    }

    assert_eq!(probe.temp_path(), real_temp);
}

#[test]
fn unknown_function_patches_nothing() {
    let probe = FolderProbe::load();

    let slots = Patch::at(probe.base(), "NoSuchFunction", Replacements::temp_path as *const () as usize).unwrap();

    assert!(slots.is_empty());
}

#[test]
fn api_set_import_is_found() {
    let probe = FolderProbe::load();

    let slots = Patch::at(probe.base(), "GetEnvironmentVariableW", Replacements::env as *const () as usize).unwrap();

    assert!(!slots.is_empty());
    assert_eq!(probe.env_appdata(), "");
}

#[test]
fn a_base_that_is_not_an_image_is_refused() {
    let not_an_image = [0u8; 4096];

    let result =
        Patch::at(not_an_image.as_ptr() as usize, "GetTempPathW", Replacements::temp_path as *const () as usize);

    assert!(result.is_err());
}
