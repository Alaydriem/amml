use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use amml_patch::ImportPatcher;
use windows_sys::Win32::Foundation::FreeLibrary;
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};

use crate::support::PeFixture;

/// Loads a DLL through the OS loader in the test process. Dependencies are searched only beside the
/// DLL and in System32, so an `amml.dll` on `PATH` (the test runner adds the target folder) is not found.
struct LoadedLibrary(*mut c_void);

impl LoadedLibrary {
    fn open(path: &Path) -> Option<Self> {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 path that lives for the call, and the file handle
        // argument must be null.
        let handle = unsafe {
            LoadLibraryExW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        };
        (!handle.is_null()).then_some(Self(handle))
    }

    fn has_export(&self, name: &[u8]) -> bool {
        assert_eq!(name.last(), Some(&0));
        // SAFETY: `self.0` is a module handle from `LoadLibraryExW` that is still loaded, and `name` is
        // NUL-terminated.
        unsafe { GetProcAddress(self.0, name.as_ptr()) }.is_some()
    }
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        // SAFETY: `self.0` came from `LoadLibraryExW` and is released exactly once, here.
        unsafe { FreeLibrary(self.0) };
    }
}

/// Builds the `anchor-stub` fixture, a cdylib workspace member that exports `amml_anchor`.
struct AnchorStub;

impl AnchorStub {
    fn build() -> PathBuf {
        let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("anchor-stub");
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args(["build", "-p", "anchor-stub", "--target-dir"])
            .arg(&target)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .unwrap();
        assert!(status.success(), "building the anchor-stub fixture failed");
        target.join("debug").join("anchor_stub.dll")
    }
}

/// Loads patched files in the test process with a stub `amml.dll` beside them.
pub struct StubLoad;

impl StubLoad {
    pub fn assert_loads_with_stub(patched: &[u8]) {
        let work = tempfile::tempdir().unwrap();
        let target = work.path().join("small.dll");
        std::fs::write(&target, patched).unwrap();
        std::fs::copy(AnchorStub::build(), work.path().join("amml.dll")).unwrap();
        let Some(library) = LoadedLibrary::open(&target) else {
            panic!("the patched dll must load with a stub amml.dll beside it");
        };
        assert!(library.has_export(b"amml_probe\0"));
    }
}

#[test]
fn a_patched_dll_loads_when_amml_dll_is_beside_it_and_does_not_load_without() {
    let patched = ImportPatcher::add_import(&PeFixture::small(), "amml.dll", "amml_anchor").unwrap();
    let work = tempfile::tempdir().unwrap();
    let target = work.path().join("small.dll");
    std::fs::write(&target, patched).unwrap();

    assert!(LoadedLibrary::open(&target).is_none(), "the dependency is missing, the load must fail");

    std::fs::copy(AnchorStub::build(), work.path().join("amml.dll")).unwrap();
    let Some(library) = LoadedLibrary::open(&target) else {
        panic!("the patched dll must load with a stub amml.dll beside it");
    };
    assert!(library.has_export(b"amml_probe\0"));
}
