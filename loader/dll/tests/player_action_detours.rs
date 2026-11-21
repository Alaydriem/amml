use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml::{ModListener, PlayerActionAdapter, SessionAdapter};
use amml_abi::{AmmlBand, AmmlBlockPos, AmmlEventContext, AmmlEventHeader, AmmlStatus, BlockBreakContinue};
use amml_data::{EntryKind, Resolved};

use crate::support::{ClientGame, FakeVtable, Harness};

// What the stand-in `continueDestroyBlock` received and what the listener saw, in order.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

// The stand-in `GameMode` vtable: slot 3 is `continueDestroyBlock`, the others never run.
static GAME_MODE_VTABLE: FakeVtable<4> =
    FakeVtable([unused as *const (), unused as *const (), unused as *const (), fake_continue as *const ()]);

const PLAYER_OFFSET: u64 = 0x8;

extern "C" fn unused() {}

/// The 70.26 shape: `bool continueDestroyBlock(GameMode*, BlockPos const&, face, Vec3 const&, bool&)` with the face in
/// the whole of r8.
#[inline(never)]
extern "C" fn fake_continue(
    _this: *mut c_void,
    pos: *const AmmlBlockPos,
    face: u64,
    player_pos: *const c_void,
    destroyed: *mut u8,
) -> u8 {
    // SAFETY: the test passes a live position.
    let pos = unsafe { *black_box(pos) };
    CALLS.lock().unwrap().push(format!(
        "game pos={},{},{} face={face:#x} player_pos={:#x} destroyed={:#x}",
        pos.x,
        pos.y,
        pos.z,
        black_box(player_pos) as usize,
        black_box(destroyed) as usize
    ));
    1
}

unsafe extern "C" fn on_continue(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `player.block_break_continue` with a `BlockBreakContinue`.
    let event = unsafe { &*event.cast::<BlockBreakContinue>() };
    CALLS.lock().unwrap().push(format!("event face={}", event.face.0));
    AmmlStatus::OK
}

fn entries() -> Vec<(&'static str, Resolved)> {
    let global = |address: usize| Resolved {
        kind: EntryKind::Global,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let mut all = ClientGame::entries(&[]);
    all.extend([
        ("GameMode.vtable", global(std::ptr::from_ref(&GAME_MODE_VTABLE) as usize)),
        ("GameMode.continueDestroyBlock", Resolved { kind: EntryKind::VSlot, value: 3, recipes_agreeing: 2 }),
        ("GameMode.mPlayer", Resolved { kind: EntryKind::Field, value: PLAYER_OFFSET, recipes_agreeing: 2 }),
    ]);
    all
}

#[test]
fn continue_destroy_passes_a_face_with_upper_bytes_on_bit_for_bit_and_reads_the_face_from_the_low_byte() {
    let harness = Harness::start_with_entries(&entries());
    SessionAdapter::install(harness.state);
    let rows = PlayerActionAdapter::install(harness.state);
    assert!(rows.iter().any(|row| row.id == "player.block_break_continue" && row.shape.is_some()), "{rows:?}");
    let listener = ModListener { owner: 0, callback: on_continue, user: 0 };
    assert!(harness.state.events().listen("player.block_break_continue", 1, AmmlBand::NORMAL, listener).is_ok());
    ClientGame::frame();
    let game_mode = [0usize; 4];
    let pos = AmmlBlockPos { x: 1, y: 64, z: -3 };
    let mut destroyed = 0u8;

    let function =
        black_box(fake_continue as extern "C" fn(*mut c_void, *const AmmlBlockPos, u64, *const c_void, *mut u8) -> u8);
    let result = function(
        black_box(game_mode.as_ptr().cast_mut().cast()),
        black_box(&raw const pos),
        black_box(0xA5A5_5A5A_DEAD_BE03),
        black_box(0x5150 as *const c_void),
        black_box(&raw mut destroyed),
    );

    assert_eq!(result, 1);
    let destroyed_at = (&raw mut destroyed) as usize;
    assert_eq!(
        *CALLS.lock().unwrap(),
        [
            "event face=3".to_string(),
            format!("game pos=1,64,-3 face=0xa5a55a5adeadbe03 player_pos=0x5150 destroyed={destroyed_at:#x}"),
        ]
    );
}
