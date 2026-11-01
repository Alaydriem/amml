use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use amml::HookEngine;
use amml_abi::AmmlStatus;

type Signature = extern "C" fn(i32) -> i32;

static LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static ORIGINAL_A: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_B: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_C: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// The function under hook and three hooks on it. Chain order A, B, C:
// A(x) = B(x) * 10, B(x) = C(x) + 100, C(x) = target(x) + 1000, target(x) = x + 1.
struct Probe;

impl Probe {
    #[inline(never)]
    extern "C" fn target(x: i32) -> i32 {
        black_box(x) + black_box(1)
    }

    extern "C" fn hook_a(x: i32) -> i32 {
        Self::note("A");
        Self::forward(&ORIGINAL_A, x) * 10
    }

    extern "C" fn hook_b(x: i32) -> i32 {
        Self::note("B");
        Self::forward(&ORIGINAL_B, x) + 100
    }

    extern "C" fn hook_c(x: i32) -> i32 {
        Self::note("C");
        Self::forward(&ORIGINAL_C, x) + 1000
    }

    fn note(name: &'static str) {
        if let Ok(mut log) = LOG.lock() {
            log.push(name);
        }
    }

    fn forward(slot: &AtomicPtr<c_void>, x: i32) -> i32 {
        let next = slot.load(Ordering::Acquire);
        // SAFETY: the engine stored the next hook or the trampoline here before the hook ran, and
        // both have the signature of `target`.
        let next = unsafe { std::mem::transmute::<*mut c_void, Signature>(next) };
        next(x)
    }

    fn call(x: i32) -> i32 {
        black_box(Self::target as Signature)(black_box(x))
    }

    fn target_ptr() -> *const c_void {
        black_box(Self::target as Signature) as *const c_void
    }

    fn take_log() -> Vec<&'static str> {
        LOG.lock().map(|mut log| std::mem::take(&mut *log)).unwrap_or_default()
    }

    fn hook(engine: &mut HookEngine, detour: Signature, priority: i32, slot: &'static AtomicPtr<c_void>) -> u64 {
        // SAFETY: `target` is a plain function of the test binary, `detour` has its signature, and
        // `slot` is a static AtomicPtr that outlives the engine.
        unsafe { engine.hook(Self::target_ptr(), detour as *const c_void, priority, slot.as_ptr().cast()).unwrap() }
    }
}

#[test]
fn two_hooks_on_one_function_both_run_in_priority_order() {
    let mut engine = HookEngine::new(8);
    // registered low priority first, so the order comes from the priority
    Probe::hook(&mut engine, Probe::hook_b, 0, &ORIGINAL_B);
    Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);

    // target(1) = 2; B adds 100; A multiplies by 10
    assert_eq!(Probe::call(1), 1020);
    assert_eq!(Probe::take_log(), vec!["A", "B"]);
}

#[test]
fn removing_the_middle_hook_keeps_the_chain_intact() {
    let mut engine = HookEngine::new(8);
    Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
    let middle = Probe::hook(&mut engine, Probe::hook_b, 5, &ORIGINAL_B);
    Probe::hook(&mut engine, Probe::hook_c, 0, &ORIGINAL_C);
    assert_eq!(Probe::call(1), 11020);
    Probe::take_log();

    engine.unhook(middle).unwrap();

    // target(1) = 2; C adds 1000; A multiplies by 10
    assert_eq!(Probe::call(1), 10020);
    assert_eq!(Probe::take_log(), vec!["A", "C"]);
}

#[test]
fn a_removed_first_hook_still_reaches_the_rest_of_the_chain() {
    let mut engine = HookEngine::new(8);
    let first = Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
    Probe::hook(&mut engine, Probe::hook_b, 0, &ORIGINAL_B);

    engine.unhook(first).unwrap();

    // the target now enters at B
    assert_eq!(Probe::call(1), 102);
    assert_eq!(Probe::take_log(), vec!["B"]);
    // a thread that was inside A when it was removed still gets through B to the target
    assert_eq!(Probe::hook_a(1), 1020);
}

#[test]
fn unhooking_all_restores_the_original() {
    let mut engine = HookEngine::new(8);
    let a = Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
    let b = Probe::hook(&mut engine, Probe::hook_b, 0, &ORIGINAL_B);

    engine.unhook(a).unwrap();
    engine.unhook(b).unwrap();

    assert_eq!(Probe::call(1), 2);
    assert!(Probe::take_log().is_empty());
    assert!(engine.is_empty());
    // a thread that was inside B when it was removed still reaches the original
    assert_eq!(Probe::hook_b(1), 102);
}

#[test]
fn a_target_can_be_hooked_again_after_all_hooks_were_removed() {
    let mut engine = HookEngine::new(8);
    let a = Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
    engine.unhook(a).unwrap();

    Probe::hook(&mut engine, Probe::hook_b, 0, &ORIGINAL_B);

    assert_eq!(Probe::call(1), 102);
}

#[test]
fn a_bad_slot_or_handle_is_refused_without_touching_the_target() {
    let mut engine = HookEngine::new(8);

    // SAFETY: the null slot is refused before any pointer is used.
    let null_slot = unsafe {
        engine.hook(Probe::target_ptr(), Probe::hook_a as Signature as *const c_void, 0, std::ptr::null_mut())
    };
    assert_eq!(null_slot.unwrap_err().status(), AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(engine.unhook(99).unwrap_err().status(), AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(Probe::call(1), 2);
}

#[test]
fn the_hook_limit_is_enforced() {
    let mut engine = HookEngine::new(1);
    Probe::hook(&mut engine, Probe::hook_a, 0, &ORIGINAL_A);

    // SAFETY: ORIGINAL_B is a static AtomicPtr; the limit refuses the hook before it is used.
    let second = unsafe {
        engine.hook(Probe::target_ptr(), Probe::hook_b as Signature as *const c_void, 0, ORIGINAL_B.as_ptr().cast())
    };

    assert_eq!(second.unwrap_err().status(), AmmlStatus::FAILED);
}

#[test]
fn calls_on_another_thread_see_a_complete_chain_while_hooks_come_and_go() {
    static STOP: AtomicBool = AtomicBool::new(false);
    let mut engine = HookEngine::new(8);
    static STARTED: AtomicBool = AtomicBool::new(false);
    let caller = std::thread::spawn(|| {
        let mut seen = Vec::new();
        while !STOP.load(Ordering::Acquire) {
            seen.push(Probe::call(1));
            STARTED.store(true, Ordering::Release);
        }
        seen
    });
    while !STARTED.load(Ordering::Acquire) {
        std::thread::yield_now();
    }

    for _ in 0..500 {
        let a = Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
        let b = Probe::hook(&mut engine, Probe::hook_b, 0, &ORIGINAL_B);
        engine.unhook(a).unwrap();
        engine.unhook(b).unwrap();
    }
    STOP.store(true, Ordering::Release);
    let seen = caller.join().unwrap();

    // original, A alone, B alone, or both: never a torn chain
    assert!(seen.iter().any(|value| *value != 2), "no hooked value was seen in {seen:?}");
    assert!(seen.iter().all(|value| [2, 20, 102, 1020].contains(value)), "unexpected result in {seen:?}");
}

#[test]
fn a_slot_or_detour_already_in_use_is_refused_and_the_live_slot_is_left_alone() {
    let mut engine = HookEngine::new(8);
    Probe::hook(&mut engine, Probe::hook_a, 10, &ORIGINAL_A);
    let live = ORIGINAL_A.load(Ordering::Acquire);

    // SAFETY: both slots are static AtomicPtrs; the requests are refused before any slot is written.
    let (same_slot, same_detour) = unsafe {
        (
            engine.hook(
                Probe::target_ptr(),
                Probe::hook_b as Signature as *const c_void,
                0,
                ORIGINAL_A.as_ptr().cast(),
            ),
            engine.hook(
                Probe::target_ptr(),
                Probe::hook_a as Signature as *const c_void,
                0,
                ORIGINAL_B.as_ptr().cast(),
            ),
        )
    };

    assert_eq!(same_slot.unwrap_err().status(), AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(same_detour.unwrap_err().status(), AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(ORIGINAL_A.load(Ordering::Acquire), live);
    assert_eq!(Probe::call(1), 20);
}
