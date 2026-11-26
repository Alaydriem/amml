use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_data::AdapterCatalog;

use crate::{Boundary, JoinEvents, SessionHook};

/// `void MinecraftGame::joinMultiplayerWithAddress(Social::GameConnectionInfo, bool isLocal, const std::string&,
/// const std::string&, NetworkType, std::shared_ptr<bool>, PlayerJoinWorldContext)`: rcx the MinecraftGame, rdx a pointer
/// to the by-value connection info, r8b the local flag, r9 the first string, then the stack arguments at entry+28h (the
/// second string), +30h (the network type), +38h (a pointer to the by-value shared_ptr) and +40h (a pointer to the
/// by-value context). The four register arguments are taken and passed on as whole 64-bit registers, and the four
/// stack arguments as the whole qwords at entry+28h..+40h (`signature` reads four stack slots on both builds), so the
/// detour changes no bit of the game's call.
type JoinMultiplayerFn = unsafe extern "C" fn(*mut c_void, usize, usize, usize, usize, usize, usize, usize);

static JOIN_MULTIPLAYER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour of `MinecraftGame.joinMultiplayerWithAddress`, the function the game's server joins reach (the
/// Servers tab, `World::ServerWorldJoiner::joinExternalServer`, Realms, the local world's own localhost join): raises
/// `ServerJoinStarting` Before the game's call for a join whose local flag is clear, and calls the original exactly once
/// with the arguments it received.
pub struct GameJoinDetours;

impl GameJoinDetours {
    pub const ENTRY: &'static str = "MinecraftGame.joinMultiplayerWithAddress";

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: Some(AdapterCatalog::MINECRAFT_GAME_VTABLE),
            detour: Self::join_multiplayer as JoinMultiplayerFn as *const c_void,
            original: &JOIN_MULTIPLAYER,
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn join_multiplayer(
        this: *mut c_void,
        info: usize,
        local: usize,
        first: usize,
        second: usize,
        network: usize,
        shared: usize,
        context: usize,
    ) {
        Boundary::run((), || JoinEvents::with(|events| events.game_join_starting(info, local, context)));
        // SAFETY: the hook on `MinecraftGame.joinMultiplayerWithAddress` is installed, so its slot holds a
        // `JoinMultiplayerFn`.
        let original =
            unsafe { std::mem::transmute::<*mut c_void, JoinMultiplayerFn>(JOIN_MULTIPLAYER.load(Ordering::Acquire)) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, info, local, first, second, network, shared, context) };
    }
}
