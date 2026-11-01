//! The detour on `ClientInstance::update`.

use amml_game::hooks::{ClientUpdate, ClientUpdateFn};
use amml_sdk::{DetourGuard, ModRuntime};

use crate::{PROBE, TICKS};

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("client_update");

// bool ClientInstance::update(bool isInitFinished) (26.60: update(bool, bool)): once per frame on the game thread.
// 26.60 added a bool third argument (r8). It is carried as a raw u64 so the register reaches the original bit for
// bit; 26.51 ignores r8, so one declaration works on both builds.
pub extern "C" fn detour(this: usize, init_finished: u8, extra: u64) -> u8 {
    let original: ClientUpdateFn = GUARD.original_fn();
    ClientUpdate::run(
        // SAFETY: `original` is the next function in the chain, with the hooked signature.
        #[allow(unsafe_code)]
        |this, init_finished, extra| unsafe { original(this, init_finished, extra) },
        this,
        init_finished,
        extra,
        PROBE.latch(),
        |client| {
            if TICKS.tick()
                && let Some(runtime) = ModRuntime::get()
            {
                runtime.logger().info(&format!("amml-testmod: ticks={}", TICKS.target()));
            }
            PROBE.on_tick(client);
        },
    )
}
