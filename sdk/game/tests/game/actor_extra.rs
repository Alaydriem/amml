use std::cell::Cell;
use std::ffi::c_void;

use amml_game::{Actor, EntryId, GameEntries};

use crate::support::{FakeActor, FakeEntries};

thread_local! {
    static ATTACH_CALL: Cell<Option<(usize, usize, i32, f32)>> = const { Cell::new(None) };
    static HEALTH_CONTEXT: Cell<usize> = const { Cell::new(0) };
}

extern "C" fn get_attach_pos(this: *mut c_void, out: *mut [f32; 3], location: i32, alpha: f32) -> *mut [f32; 3] {
    ATTACH_CALL.with(|call| call.set(Some((this as usize, out as usize, location, alpha))));
    // SAFETY: the caller passes a buffer for a Vec3.
    unsafe { out.write([4.0, 70.62, -9.5]) };
    out
}

extern "C" fn get_health(context: *const c_void) -> i32 {
    HEALTH_CONTEXT.with(|seen| seen.set(context as usize));
    17
}

fn entries() -> GameEntries {
    FakeEntries::build(&[
        (
            EntryId::ACTOR_GET_ATTACH_POS,
            (get_attach_pos as extern "C" fn(*mut c_void, *mut [f32; 3], i32, f32) -> *mut [f32; 3]) as usize as u64,
        ),
        (EntryId::ACTOR_ENTITY_CONTEXT, 8),
        (EntryId::ACTOR_ATTRIBUTE_GET_HEALTH, (get_health as extern "C" fn(*const c_void) -> i32) as usize as u64),
    ])
}

fn fake() -> FakeActor {
    FakeActor::new(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut())
}

#[test]
fn head_position_passes_the_head_location_and_a_hidden_return_buffer() {
    let entries = entries();
    let mut fake = fake();
    // SAFETY: a live fake actor.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.head_position(), Some([4.0, 70.62, -9.5]));
    let (this, out, location, alpha) = ATTACH_CALL.with(Cell::get).unwrap();
    assert_eq!(this, fake.ptr() as usize);
    assert_ne!(out, 0);
    assert_ne!(out, this);
    assert_eq!(location, Actor::HEAD_LOCATION);
    assert_eq!(location, 3);
    assert_eq!(alpha, 0.0);
}

#[test]
fn health_passes_the_entity_context_inside_the_actor() {
    let entries = entries();
    let mut fake = fake();
    // SAFETY: a live fake actor.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &entries) }.unwrap();

    assert_eq!(actor.health(), Some(17));
    assert_eq!(HEALTH_CONTEXT.with(Cell::get), fake.ptr() as usize + 8);
}

#[test]
fn a_missing_entry_gives_none_and_calls_nothing() {
    let none = FakeEntries::build(&[(EntryId::ACTOR_ENTITY_CONTEXT, 8)]);
    let mut fake = fake();
    // SAFETY: a live fake actor.
    let actor = unsafe { Actor::from_raw(fake.ptr(), &none) }.unwrap();

    assert_eq!(actor.head_position(), None);
    assert_eq!(actor.health(), None);
    assert_eq!(ATTACH_CALL.with(Cell::get), None);
    assert_eq!(HEALTH_CONTEXT.with(Cell::get), 0);
}
