use amml_game::{ActorRenderQueue, EntryId};

use crate::support::{FakeEntries, FakeRenderQueue, GameMemory};

const A: usize = 0x1000;
const B: usize = 0x2000;
const C: usize = 0x3000;

fn erase(queue: &mut FakeRenderQueue, actor: usize) -> bool {
    let mut wrapper = ActorRenderQueue::from_layout(queue.layout());
    // SAFETY: the buffers are live, the keys and values have the same length, and the test is single threaded.
    unsafe { wrapper.erase_actor(actor as *mut std::ffi::c_void) }
}

#[test]
fn an_absent_key_changes_nothing() {
    let mut queue = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);

    assert!(!erase(&mut queue, 0x9999));

    assert_eq!(queue.live_keys(), [A, B, C]);
    assert_eq!(queue.live_values(), [1, 2, 3]);
    assert!(queue.buffers_unchanged());
}

#[test]
fn the_first_index_moves_the_tail_down_in_both_vectors() {
    let mut queue = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);

    assert!(erase(&mut queue, A));

    assert_eq!(queue.live_keys(), [B, C]);
    assert_eq!(queue.live_values(), [2, 3]);
    assert!(queue.buffers_unchanged());
}

#[test]
fn a_middle_index_keeps_keys_and_values_paired() {
    let mut queue = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);

    assert!(erase(&mut queue, B));

    assert_eq!(queue.live_keys(), [A, C]);
    assert_eq!(queue.live_values(), [1, 3]);
}

#[test]
fn the_last_index_only_shortens_the_vectors() {
    let mut queue = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);

    assert!(erase(&mut queue, C));

    assert_eq!(queue.live_keys(), [A, B]);
    assert_eq!(queue.live_values(), [1, 2]);
    assert!(queue.buffers_unchanged());
}

#[test]
fn the_only_element_leaves_empty_vectors_with_the_same_buffers() {
    let mut queue = FakeRenderQueue::new(&[A], &[7]);

    assert!(erase(&mut queue, A));

    assert!(queue.live_keys().is_empty());
    assert!(queue.live_values().is_empty());
    assert!(queue.buffers_unchanged());
}

#[test]
fn unequal_lengths_are_refused_and_nothing_changes() {
    for (keys, values) in [(&[A, B, C][..], &[1, 2][..]), (&[A, B][..], &[1, 2, 3][..])] {
        let mut queue = FakeRenderQueue::new(keys, values);

        assert!(!erase(&mut queue, A));

        assert_eq!(queue.live_keys(), keys);
        assert_eq!(queue.live_values(), values);
        assert!(queue.buffers_unchanged());
    }
}

#[test]
fn a_queue_with_null_pointers_is_refused() {
    let mut queue = FakeRenderQueue::empty();

    assert!(!erase(&mut queue, A));
}

#[test]
fn a_reversed_vector_is_refused() {
    let mut queue = FakeRenderQueue::new(&[A, B], &[1, 2]);
    let layout = queue.layout();
    std::mem::swap(&mut layout.keys_begin, &mut layout.keys_end);
    std::mem::swap(&mut layout.values_begin, &mut layout.values_end);

    assert!(!erase(&mut queue, A));
}

#[test]
fn a_misaligned_key_range_is_refused_and_nothing_changes() {
    let mut queue = FakeRenderQueue::new(&[A, B], &[1, 2]);
    let layout = queue.layout();
    // an end that is 4 bytes past the second key
    layout.keys_end = layout.keys_end.cast::<u8>().wrapping_add(4).cast();

    assert!(!erase(&mut queue, A));

    assert_eq!(queue.live_values(), [1, 2]);
}

const RENDERER_QUEUE_AT: usize = 0x1D8;

fn renderer_entries() -> amml_game::GameEntries {
    FakeEntries::build(&[(EntryId::LEVEL_RENDERER_CAMERA_ACTOR_RENDER_QUEUE, RENDERER_QUEUE_AT as u64)])
}

#[test]
fn from_renderer_finds_the_queue_at_the_entry_offset_and_an_erase_writes_there() {
    let entries = renderer_entries();
    let mut fake = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);
    let mut renderer = GameMemory::new(0x300);
    // copy the six qwords of the layout to renderer + 0x1D8
    let layout = fake.layout();
    let words = [
        layout.keys_begin as u64,
        layout.keys_end as u64,
        layout.keys_cap as u64,
        layout.values_begin as u64,
        layout.values_end as u64,
        layout.values_cap as u64,
    ];
    for (index, word) in words.iter().enumerate() {
        renderer.put::<u64>(RENDERER_QUEUE_AT + index * 8, *word);
    }

    // SAFETY: a live fake renderer with the queue at the entry's offset.
    let mut queue = unsafe { ActorRenderQueue::from_renderer(renderer.ptr(), &entries) }.unwrap();
    // SAFETY: the buffers are live, the keys and values have the same length, and the test is single threaded.
    assert!(unsafe { queue.erase_actor(B as *mut std::ffi::c_void) });

    assert_eq!(renderer.get::<u64>(RENDERER_QUEUE_AT + 8), words[1] - 8);
    assert_eq!(renderer.get::<u64>(RENDERER_QUEUE_AT + 0x20), words[4] - 1);
    assert_eq!(renderer.get::<u64>(RENDERER_QUEUE_AT), words[0]);
    assert_eq!(renderer.get::<u64>(RENDERER_QUEUE_AT + 0x10), words[2]);
}

#[test]
fn from_renderer_is_none_for_a_null_renderer_or_a_missing_entry() {
    let entries = renderer_entries();
    // SAFETY: null is allowed.
    assert!(unsafe { ActorRenderQueue::from_renderer(std::ptr::null_mut(), &entries) }.is_none());

    let none = FakeEntries::build(&[]);
    let mut renderer = GameMemory::new(0x300);
    // SAFETY: no entry is present, so the renderer is never read.
    assert!(unsafe { ActorRenderQueue::from_renderer(renderer.ptr(), &none) }.is_none());
}

#[test]
fn a_misaligned_keys_pointer_is_refused_and_nothing_changes() {
    let mut queue = FakeRenderQueue::new(&[A, B, C], &[1, 2, 3]);
    let layout = queue.layout();
    // begin and end both 4 bytes off, so the range is still a multiple of 8
    layout.keys_begin = layout.keys_begin.cast::<u8>().wrapping_add(4).cast();
    layout.keys_end = layout.keys_end.cast::<u8>().wrapping_add(4).cast();

    assert!(!erase(&mut queue, A));

    assert_eq!(queue.live_values(), [1, 2, 3]);
}
