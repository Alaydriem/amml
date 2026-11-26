use amml_runtime::{CaptureRing, SlotRelease};

fn fill(ring: &CaptureRing, present: u64) -> usize {
    assert!(ring.wants(present));
    ring.filled(present).unwrap()
}

#[test]
fn a_full_ring_skips_presents_without_blocking() {
    let ring = CaptureRing::new(2, 1);
    fill(&ring, 0);
    fill(&ring, 1);
    assert!(!ring.wants(2));
    assert!(!ring.wants(3));
    assert_eq!(ring.skipped(), 2);
    assert_eq!(ring.free_slots(), 0);
}

#[test]
fn a_taken_frame_returns_its_slot_on_give_back() {
    let ring = CaptureRing::new(2, 1);
    fill(&ring, 0);
    assert_eq!(ring.take(), Some((0, 0)));
    assert_eq!(ring.free_slots(), 1);
    assert_eq!(ring.give_back(0), SlotRelease::Returned);
    assert_eq!(ring.free_slots(), 2);
    assert!(!ring.is_closed());
}

#[test]
fn closing_with_a_taken_frame_frees_it_at_give_back() {
    let ring = CaptureRing::new(2, 1);
    fill(&ring, 0);
    ring.take();
    assert_eq!(ring.close(), vec![1]);
    assert!(ring.is_closed());
    assert_eq!(ring.give_back(0), SlotRelease::FreeNow);
    assert_eq!(ring.give_back(0), SlotRelease::NotHeld);
}

#[test]
fn only_every_nth_present_is_wanted() {
    let ring = CaptureRing::new(8, 3);
    let wanted: Vec<u64> = (0..7).filter(|present| ring.wants(*present)).collect();
    assert_eq!(wanted, vec![0, 3, 6]);
    assert_eq!(ring.skipped(), 0);
}

#[test]
fn a_writing_slot_is_not_taken() {
    let ring = CaptureRing::new(2, 1);
    assert!(ring.wants(0));
    assert_eq!(ring.writing(0), Some(0));
    assert_eq!(ring.take(), None);
    assert_eq!(ring.filled(0), Some(0));
    assert_eq!(ring.take(), Some((0, 0)));
}

#[test]
fn frames_are_taken_oldest_first() {
    let ring = CaptureRing::new(3, 1);
    assert!(ring.wants(0));
    assert!(ring.wants(1));
    ring.filled(1);
    ring.filled(0);
    assert_eq!(ring.take(), Some((1, 1)));
    assert_eq!(ring.take(), Some((0, 0)));
    assert_eq!(ring.take(), None);
}

#[test]
fn a_closed_ring_wants_nothing_and_frees_a_write_in_progress_at_filled() {
    let ring = CaptureRing::new(2, 1);
    assert!(ring.wants(0));
    assert_eq!(ring.close(), vec![1]);
    assert!(!ring.wants(1));
    assert_eq!(ring.skipped(), 0);
    assert_eq!(ring.filled(0), None);
    assert_eq!(ring.free_slots(), 2);
}

#[test]
fn closing_frees_filled_slots_at_once() {
    let ring = CaptureRing::new(2, 1);
    fill(&ring, 0);
    assert_eq!(ring.close(), vec![0, 1]);
    assert_eq!(ring.take(), None);
    assert!(ring.close().is_empty());
}

#[test]
fn an_abandoned_reservation_frees_its_slot_in_an_open_ring() {
    let ring = CaptureRing::new(1, 1);
    assert!(ring.wants(0));
    assert!(!ring.wants(1));

    assert_eq!(ring.abandon(0), SlotRelease::Returned);

    assert_eq!(ring.free_slots(), 1);
    assert_eq!(ring.filled(0), None);
    assert!(ring.wants(2));
    assert_eq!(ring.skipped(), 1);
}

#[test]
fn an_abandoned_reservation_in_a_closed_ring_is_freed_now() {
    let ring = CaptureRing::new(2, 1);
    assert!(ring.wants(0));
    assert_eq!(ring.close(), vec![1]);

    assert_eq!(ring.abandon(0), SlotRelease::FreeNow);

    assert_eq!(ring.abandon(0), SlotRelease::NotHeld);
    assert_eq!(ring.filled(0), None);
    assert_eq!(ring.free_slots(), 2);
}

#[test]
fn abandon_and_give_back_touch_only_their_own_state() {
    let ring = CaptureRing::new(2, 1);
    fill(&ring, 0);
    ring.take();
    assert!(ring.wants(1));

    assert_eq!(ring.abandon(0), SlotRelease::NotHeld);
    assert_eq!(ring.give_back(1), SlotRelease::NotHeld);
    assert_eq!(ring.free_slots(), 0);
}
