use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_data::AdapterCatalog;

use crate::{Boundary, JoinEvents, SessionHook};

/// `void ClientInstance::startExternalNetworkWorld(Social::GameConnectionInfo, std::string const& serverName,
/// PlayerJoinWorldContext)`: rcx the ClientInstance, rdx a pointer to the by-value connection info, r8 the server name,
/// r9 a pointer to the by-value context. The callee destroys both by-value arguments, so the detour passes the
/// pointers on unchanged and never destroys them.
type ExternalFn = unsafe extern "C" fn(*mut c_void, usize, usize, usize);

static EXTERNAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour of `ClientInstance.startExternalNetworkWorld`, which raises `ServerJoinStarting` Before the
/// game's call and calls the original exactly once with the arguments it received.
pub struct ExternalJoinDetours;

impl ExternalJoinDetours {
    pub const ENTRY: &'static str = "ClientInstance.startExternalNetworkWorld";

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: Some(AdapterCatalog::CLIENT_INSTANCE_VTABLE),
            detour: Self::external as ExternalFn as *const c_void,
            original: &EXTERNAL,
        }
    }

    unsafe extern "C" fn external(this: *mut c_void, info: usize, name: usize, context: usize) {
        Boundary::run((), || JoinEvents::with(|events| events.server_join_starting(info, name, context)));
        // SAFETY: the hook on `ClientInstance.startExternalNetworkWorld` is installed, so its slot holds an
        // `ExternalFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, ExternalFn>(EXTERNAL.load(Ordering::Acquire)) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, info, name, context) };
    }
}
