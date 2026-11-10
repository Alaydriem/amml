use std::ffi::c_void;
use std::sync::Mutex;

use amml_abi::{AMML_API_VERSION, AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr};

pub static LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// A loader table whose functions are fakes: `resolve` knows the id `known`, `log` records lines.
pub struct FakeLoader;

impl FakeLoader {
    unsafe extern "C" fn resolve(id: AmmlStr, out: *mut u64) -> AmmlStatus {
        // SAFETY: the SDK passes a live string view.
        let id = unsafe { id.as_str() }.unwrap();
        if id == "known" {
            // SAFETY: the SDK passes a live u64.
            unsafe { out.write(0x1234) };
            AmmlStatus::OK
        } else {
            AmmlStatus::MISSING_ENTRY
        }
    }

    unsafe extern "C" fn hook(
        _: *const c_void,
        _: *const c_void,
        _: i32,
        _: *mut *const c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn unhook(_: u64) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn log(module: AmmlStr, level: AmmlLogLevel, message: AmmlStr) {
        // SAFETY: the SDK passes live string views.
        let (module, message) = unsafe { (module.as_str().unwrap(), message.as_str().unwrap()) };
        LINES.lock().unwrap().push(format!("{module} {} {message}", level.0));
    }

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    // The module name picks the answer: `shared` has a shared data folder, `old` is a loader that
    // refuses kind 3, `broken` fails kind 3 another way, `rootless` answers kind 3 with an empty folder as a loader
    // with no shared data root does. Any other module fails every kind.
    unsafe extern "C" fn mod_dir(module: AmmlStr, kind: u8, out: *mut AmmlStr) -> AmmlStatus {
        // SAFETY: the SDK passes a live string view.
        let module = unsafe { module.as_str() }.unwrap();
        let text = match (module, kind) {
            ("shared" | "old" | "broken" | "rootless", 0..=2) => r"C:\m",
            ("shared", 3) => r"C:\s",
            ("rootless", 3) => "",
            ("old", 3) => return AmmlStatus::INVALID_ARGUMENT,
            _ => return AmmlStatus::FAILED,
        };
        // SAFETY: the SDK passes a writable AmmlStr and the text is 'static.
        unsafe { out.write(AmmlStr::from_str(text)) };
        AmmlStatus::OK
    }

    pub fn table(size: u32, version: u32) -> AmmlApi {
        AmmlApi {
            size,
            version,
            module_base: 0x1000,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
        }
    }

    pub fn current() -> AmmlApi {
        Self::table(AmmlApi::SIZE_V1, AMML_API_VERSION)
    }
}
