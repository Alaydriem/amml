use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{Boundary, ScreenDetours, SessionHook, TimeAdapter, TimerLayout};

/// `void Timer::advanceTime(float preferredFrameStep, bool forceOnlySingleTick)`: rcx the timer, xmm1 the step, r8b the
/// flag; the value left in xmm0 is passed back (survey section 8, row T1).
type AdvanceFn = unsafe extern "C" fn(*mut c_void, f32, u8) -> f32;

/// `void ParticleSystemEngine::tick(float alpha, bool paused)`: rcx the engine, xmm1 the frame alpha, r8 the paused flag
/// (the game reads r8d; the loader passes the register on as it came), no result (row T5).
type ParticleFn = unsafe extern "C" fn(*mut c_void, f32, u32);

/// `void PredictedMovementSystem::_tickPredictedMovement(...)`: ten arguments, the first four in rcx, rdx, r8 and r9 and
/// the other six in the stack slots 28h to 50h; each is a pointer or a value of at most eight bytes in one integer
/// slot, so the detour takes and passes each as one word. The tenth is the `SystemUniforms const&` (row T4, and the
/// signature of the entry); no result.
type MovementFn = unsafe extern "C" fn(usize, usize, usize, usize, usize, usize, usize, usize, usize, usize);

/// `void MinecraftGame::endFrame()`: rcx the game, no result (row T6).
type EndFrameFn = unsafe extern "C" fn(*mut c_void);

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PARTICLE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static MOVEMENT_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static END_FRAME_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the time detours have work for besides the game's own call. Zero: each detour calls the original and nothing
// else.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The loader's detours on `Timer::advanceTime`, `ParticleSystemEngine::tick`,
/// `PredictedMovementSystem::_tickPredictedMovement` and `MinecraftGame::endFrame`, and their flags word.
///
/// With no flag set each detour calls the original and does nothing else: one relaxed load, no lock, no allocation, no
/// log. `HELD` is set while a mod holds the game clock. The advance detour then keeps the timer's overflow time from
/// before the advance and hands the timer to `TimeAdapter` after it, which writes the held clock into the four client
/// timers and leaves every other timer (a server timer) alone. The particle detour moves the engine's last tick time
/// Before the game's tick so the step it measures is the game time stepped since the last tick, and notes the particle
/// count After. The movement detour puts the virtual `mNow` into the uniforms for the call and the wall value back after
/// it, on every path. `FRAME_END` is set while a mod listens to `FrameEnd`: the end frame detour then counts the call
/// and raises the event Before the game's call. A panic inside the loader's work is caught at the boundary; the game's
/// own call runs once with its own arguments.
pub struct TimeDetours;

impl TimeDetours {
    pub const ADVANCE: &'static str = "Timer.advanceTime";
    pub const PARTICLE: &'static str = "ParticleSystemEngine.tick";
    pub const MOVEMENT: &'static str = "PredictedMovementSystem.tickPredictedMovement";
    pub const END_FRAME: &'static str = "MinecraftGame.endFrame";
    /// A mod holds the game clock.
    pub const HELD: u32 = 1;
    /// A mod listens to `FrameEnd`.
    pub const FRAME_END: u32 = 2;

    /// The hook on `Timer::advanceTime`.
    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ADVANCE,
            vtable: None,
            detour: Self::advance as AdvanceFn as *const c_void,
            original: &ORIGINAL,
        }
    }

    /// The hook on `ParticleSystemEngine::tick`.
    pub fn particle_hook() -> SessionHook {
        SessionHook {
            entry: Self::PARTICLE,
            vtable: None,
            detour: Self::particle as ParticleFn as *const c_void,
            original: &PARTICLE_ORIGINAL,
        }
    }

    /// The hook on `PredictedMovementSystem::_tickPredictedMovement`.
    pub fn movement_hook() -> SessionHook {
        SessionHook {
            entry: Self::MOVEMENT,
            vtable: None,
            detour: Self::movement as MovementFn as *const c_void,
            original: &MOVEMENT_ORIGINAL,
        }
    }

    /// The hook on `MinecraftGame::endFrame`.
    pub fn end_frame_hook() -> SessionHook {
        SessionHook {
            entry: Self::END_FRAME,
            vtable: None,
            detour: Self::end_frame as EndFrameFn as *const c_void,
            original: &END_FRAME_ORIGINAL,
        }
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// The flags word: one relaxed load.
    pub fn flags() -> u32 {
        WANTED.load(Ordering::Relaxed)
    }

    unsafe extern "C" fn advance(this: *mut c_void, step: f32, single: u8) -> f32 {
        // SAFETY: the hook on `advanceTime` is installed, so its original slot holds an `AdvanceFn`.
        let original = unsafe { ScreenDetours::original::<AdvanceFn>(&ORIGINAL) };
        if Self::flags() & Self::HELD == 0 {
            // SAFETY: the game's own arguments, passed on unchanged.
            return unsafe { original(this, step, single) };
        }
        let timer = this.expose_provenance();
        // SAFETY: `this` is the live Timer the game advances; the mirror covers its prefix on every supported build.
        let overflow = unsafe { TimerLayout::overflow_time(timer) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, step, single) };
        Boundary::run((), || TimeAdapter::with((), |adapter| adapter.advanced(timer, overflow)));
        result
    }

    unsafe extern "C" fn particle(this: *mut c_void, alpha: f32, paused: u32) {
        // SAFETY: the hook on the particle tick is installed, so its original slot holds a `ParticleFn`.
        let original = unsafe { ScreenDetours::original::<ParticleFn>(&PARTICLE_ORIGINAL) };
        if Self::flags() & Self::HELD == 0 {
            // SAFETY: the game's own arguments, passed on unchanged.
            return unsafe { original(this, alpha, paused) };
        }
        let engine = this.expose_provenance();
        Boundary::run((), || TimeAdapter::with((), |adapter| adapter.particles_before(engine)));
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, alpha, paused) };
        Boundary::run((), || TimeAdapter::with((), |adapter| adapter.particles_after(engine)));
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn movement(
        a1: usize,
        a2: usize,
        a3: usize,
        a4: usize,
        a5: usize,
        a6: usize,
        a7: usize,
        a8: usize,
        a9: usize,
        uniforms: usize,
    ) {
        // SAFETY: the hook on the predicted movement is installed, so its original slot holds a `MovementFn`.
        let original = unsafe { ScreenDetours::original::<MovementFn>(&MOVEMENT_ORIGINAL) };
        // The guard puts the wall value back when it drops, whichever way the detour leaves.
        let _restore = if Self::flags() & Self::HELD == 0 || uniforms == 0 {
            None
        } else {
            Boundary::run(None, || TimeAdapter::with(None, |adapter| adapter.swapped_now(uniforms)))
        };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(a1, a2, a3, a4, a5, a6, a7, a8, a9, uniforms) };
    }

    unsafe extern "C" fn end_frame(this: *mut c_void) {
        if Self::flags() & (Self::HELD | Self::FRAME_END) != 0 {
            Boundary::run((), || TimeAdapter::with((), TimeAdapter::frame_ending));
        }
        // SAFETY: the hook on `endFrame` is installed, so its original slot holds an `EndFrameFn`.
        let original = unsafe { ScreenDetours::original::<EndFrameFn>(&END_FRAME_ORIGINAL) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
    }
}
