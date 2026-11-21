use amml_abi::AmmlRequestOutcome;
use amml_runtime::{
    ProtocolValues, RequestBook, RequestDone, RequestMove, RequestRefusal, RequestSlot, RequestStep, RequestTick,
};

// The slots the stand-in game reads: slots 0 to 19 hold 8 items of a stack of 64 (key 7) with the net id 100 + slot;
// slots 20 to 35 are empty.
fn slots() -> Vec<Option<RequestSlot>> {
    (0..36u8)
        .map(|slot| (slot < 20).then(|| RequestSlot { net_id: 100 + i32::from(slot), count: 8, max: 64, key: 7 }))
        .collect()
}

fn tick(book: &mut RequestBook, now: f64, screen_open: bool) -> RequestTick {
    book.tick(now, screen_open, &mut || Some(slots()))
}

fn swaps(count: u8) -> Vec<RequestMove> {
    (0..count).map(|index| RequestMove::Swap { a: index, b: index + 1 }).collect()
}

fn done(id: u64, outcome: AmmlRequestOutcome, accepted: u16, total: u16) -> RequestDone {
    RequestDone { id, outcome, accepted, total }
}

fn varint(bytes: &[u8]) -> (u32, usize) {
    let (mut value, mut used) = (0u32, 0);
    loop {
        let byte = bytes[used];
        value |= u32::from(byte & 0x7F) << (7 * used);
        used += 1;
        if byte & 0x80 == 0 {
            return (value, used);
        }
    }
}

// The wire ids of the `count` swaps in the `SendRequests` bytes of `steps`: the zigzag varints after the count byte.
fn wire_ids(steps: &[RequestStep], count: usize) -> Vec<i32> {
    let bytes = steps
        .iter()
        .find_map(|step| match step {
            RequestStep::SendRequests(bytes) => Some(bytes.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(usize::from(bytes[0]), count);
    let mut ids = Vec::new();
    let mut at = 1;
    for _ in 0..count {
        let (value, used) = varint(&bytes[at..]);
        ids.push(((value >> 1) as i32) ^ -((value & 1) as i32));
        // The id, the action count, the swap (2 bytes, then 2 slots of 7), the end (1 byte and 4).
        at += used + 1 + 2 + 14 + 5;
    }
    ids
}

fn answers(steps: &[RequestStep], count: usize, accepted: &[bool]) -> Vec<(i32, bool)> {
    wire_ids(steps, count).into_iter().zip(accepted.iter().copied()).collect()
}

// A hidden request, run to the point where its requests are sent: the Interact, the server's open as container 5.
fn sent(book: &mut RequestBook, moves: &[RequestMove]) -> (u64, Vec<RequestStep>) {
    let id = book.start(1, moves, true, true, false).unwrap();
    assert_eq!(tick(book, 0.0, false).steps, vec![RequestStep::SendInteract]);
    assert!(book.on_container_open(0.1, 5, ProtocolValues::INVENTORY_TYPE));
    (id, tick(book, 0.2, false).steps)
}

#[test]
fn hidden_open_sends_interact_first() {
    let mut book = RequestBook::new();
    book.start(1, &swaps(1), true, false, false).unwrap();

    let first = tick(&mut book, 0.0, false);
    assert_eq!(first.steps, vec![RequestStep::SendInteract]);
    assert!(first.done.is_empty());
    // The moves wait for the server's open: nothing else goes out before it.
    assert!(tick(&mut book, 0.1, false).steps.is_empty());
    assert!(book.on_container_open(0.2, 5, ProtocolValues::INVENTORY_TYPE));

    let second = tick(&mut book, 0.3, false);
    assert!(matches!(second.steps.first(), Some(RequestStep::SendRequests(_))), "{:?}", second.steps);
}

#[test]
fn server_open_and_close_of_the_hidden_inventory_are_hidden() {
    let mut book = RequestBook::new();
    // No request: nothing is hidden.
    assert!(!book.on_container_open(0.0, 5, ProtocolValues::INVENTORY_TYPE));
    assert!(!book.on_container_close(0.0, 5));
    let (_, steps) = sent(&mut book, &swaps(1));
    // A chest the player opened is not the hidden inventory.
    assert!(!book.on_container_open(0.25, 6, 7));
    book.on_responses(&answers(&steps, 1, &[true]));

    let end = tick(&mut book, 0.3, false);

    assert_eq!(end.steps, vec![RequestStep::SendClose(5)]);
    // The server answers our close once; another id and a second answer are for the game.
    assert!(!book.on_container_close(0.4, 6));
    assert!(book.on_container_close(0.4, 5));
    assert!(!book.on_container_close(0.4, 5));
}

#[test]
fn the_answer_to_our_close_is_for_the_game_after_the_window() {
    let mut book = RequestBook::new();
    let (_, steps) = sent(&mut book, &swaps(1));
    book.on_responses(&answers(&steps, 1, &[true]));
    tick(&mut book, 0.3, false);

    assert!(!book.on_container_close(2.4, 5));
}

#[test]
fn refused_moves_are_undone_in_reverse_order() {
    let mut book = RequestBook::new();
    let (id, steps) = sent(&mut book, &swaps(3));
    // Three swaps of neighbours: each is predicted on the client.
    let predicted: Vec<_> = steps.iter().filter(|step| matches!(step, RequestStep::Predict { .. })).collect();
    assert_eq!(predicted.len(), 3);
    book.on_responses(&answers(&steps, 3, &[true, false, false]));

    let end = tick(&mut book, 0.3, false);

    assert_eq!(
        end.steps,
        vec![
            // Newest first (the first stack moves along the three swaps, so each names the net id 100), then the move
            // the server did is shown again, then the close.
            RequestStep::Unpredict { a: 2, b: 3, net_id: 100 },
            RequestStep::Unpredict { a: 1, b: 2, net_id: 100 },
            RequestStep::Unpredict { a: 0, b: 1, net_id: 100 },
            RequestStep::Predict { a: 0, b: 1 },
            RequestStep::SendClose(5),
        ]
    );
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::REFUSED, 1, 3)]);
}

#[test]
fn answer_inside_send_is_held_to_the_next_tick() {
    let mut book = RequestBook::new();
    let (id, steps) = sent(&mut book, &swaps(1));
    // The answer comes before the tick returns, as a local server may answer inside the send.
    book.on_responses(&answers(&steps, 1, &[true]));

    // The request still runs: nothing ended, a second request is refused.
    assert_eq!(book.start(2, &swaps(1), false, false, false), Err(RequestRefusal::Busy));
    let next = tick(&mut book, 0.25, false);

    assert_eq!(next.done, vec![done(id, AmmlRequestOutcome::DONE, 1, 1)]);
    // And only now the book is free.
    assert!(book.start(2, &swaps(1), false, false, false).is_ok());
}

#[test]
fn second_request_while_one_runs_is_refused() {
    let mut book = RequestBook::new();
    book.start(1, &swaps(1), false, false, false).unwrap();

    assert_eq!(book.start(2, &swaps(1), false, false, false), Err(RequestRefusal::Busy));
    assert_eq!(book.start(1, &swaps(1), true, false, false), Err(RequestRefusal::Busy));
}

#[test]
fn hidden_open_is_refused_while_a_screen_is_open() {
    let mut book = RequestBook::new();

    assert_eq!(book.start(1, &swaps(1), true, false, true), Err(RequestRefusal::ScreenOpen));
    // Without the hidden open the caller has its own inventory open.
    assert!(book.start(1, &swaps(1), false, false, true).is_ok());
}

#[test]
fn screen_open_during_request_cancels() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), true, true, false).unwrap();
    tick(&mut book, 0.0, false);
    assert!(book.on_container_open(0.1, 5, ProtocolValues::INVENTORY_TYPE));

    // A screen opened between the server's open and the send: nothing is sent, the inventory is closed.
    let end = tick(&mut book, 0.2, true);

    assert_eq!(end.steps, vec![RequestStep::SendClose(5)]);
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::CANCELLED, 0, 1)]);
}

#[test]
fn a_screen_that_opens_before_the_server_opened_cancels_when_it_does() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), true, true, false).unwrap();
    tick(&mut book, 0.0, false);

    assert!(tick(&mut book, 0.1, true).done.is_empty());
    assert!(book.on_container_open(0.2, 5, ProtocolValues::INVENTORY_TYPE));
    let end = tick(&mut book, 0.3, true);

    assert_eq!(end.steps, vec![RequestStep::SendClose(5)]);
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::CANCELLED, 0, 1)]);
}

#[test]
fn a_screen_after_the_send_ends_the_request_but_the_answers_are_still_owed() {
    let mut book = RequestBook::new();
    let (id, steps) = sent(&mut book, &swaps(2));

    let cancelled = tick(&mut book, 0.3, true);

    assert_eq!(cancelled.done, vec![done(id, AmmlRequestOutcome::CANCELLED, 0, 2)]);
    // The server still does the moves: the request is not over, a new one is refused.
    assert_eq!(book.start(1, &swaps(1), false, false, false), Err(RequestRefusal::Busy));
    book.on_responses(&answers(&steps, 2, &[true, true]));
    let settled = tick(&mut book, 0.4, true);
    // It ends once, without a second event, and the hidden inventory is closed.
    assert!(settled.done.is_empty());
    assert_eq!(settled.steps, vec![RequestStep::SendClose(5)]);
    assert!(book.start(1, &swaps(1), false, false, false).is_ok());
}

#[test]
fn timeout_gives_no_answer() {
    let mut book = RequestBook::new();
    let (id, _) = sent(&mut book, &swaps(2));

    assert!(tick(&mut book, 1.9, false).done.is_empty());
    let end = tick(&mut book, 2.3, false);

    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::NO_ANSWER, 0, 2)]);
    assert!(end.steps.contains(&RequestStep::SendClose(5)));
}

#[test]
fn no_open_from_the_server_gives_no_open() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), true, false, false).unwrap();
    tick(&mut book, 0.0, false);

    let end = tick(&mut book, 2.5, false);

    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::NO_OPEN, 0, 1)]);
    assert!(end.steps.is_empty());
    // A late open is still the one we asked for: it is hidden from the game and closed.
    assert!(book.on_container_open(2.6, 5, ProtocolValues::INVENTORY_TYPE));
    assert_eq!(tick(&mut book, 2.7, false).steps, vec![RequestStep::SendClose(5)]);
}

#[test]
fn disable_ends_with_no_event() {
    let mut book = RequestBook::new();
    let (_, _) = sent(&mut book, &swaps(1));

    let steps = book.disable(Some(1), 0.3, true);

    // The hidden inventory is closed while the game runs.
    assert_eq!(steps, vec![RequestStep::SendClose(5)]);
    assert!(tick(&mut book, 0.4, false).done.is_empty());
    assert!(book.start(1, &swaps(1), false, false, false).is_ok());
}

#[test]
fn disable_with_the_game_gone_sends_nothing() {
    let mut book = RequestBook::new();
    let (_, _) = sent(&mut book, &swaps(1));

    assert!(book.disable(None, 0.3, false).is_empty());
    assert!(tick(&mut book, 0.4, false).done.is_empty());
}

#[test]
fn disable_of_another_owner_leaves_the_request() {
    let mut book = RequestBook::new();
    let (_, _) = sent(&mut book, &swaps(1));

    assert!(book.disable(Some(2), 0.3, true).is_empty());
    assert_eq!(book.start(2, &swaps(1), false, false, false), Err(RequestRefusal::Busy));
}

#[test]
fn a_move_whose_slots_changed_is_stale_after_the_settle_time() {
    let mut book = RequestBook::new();
    // Slot 30 is empty: a swap with it has nothing to swap.
    let id = book.start(1, &[RequestMove::Swap { a: 0, b: 30 }], false, false, false).unwrap();

    assert!(tick(&mut book, 0.0, false).done.is_empty());
    assert!(tick(&mut book, 0.3, false).done.is_empty());
    let end = tick(&mut book, 0.6, false);

    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::STALE, 0, 1)]);
    assert!(end.steps.is_empty());
}

#[test]
fn slots_that_cannot_be_read_are_stale() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), false, false, false).unwrap();

    let first = book.tick(0.0, false, &mut || None);
    let end = book.tick(0.6, false, &mut || None);

    assert!(first.done.is_empty());
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::STALE, 0, 1)]);
}

#[test]
fn a_request_without_the_hidden_open_sends_at_the_next_tick_and_closes_nothing() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), false, false, false).unwrap();

    let first = tick(&mut book, 0.0, false);
    assert!(matches!(first.steps.as_slice(), [RequestStep::SendRequests(_)]), "{:?}", first.steps);
    book.on_responses(&answers(&first.steps, 1, &[true]));
    let end = tick(&mut book, 0.1, false);

    assert!(end.steps.is_empty());
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::DONE, 1, 1)]);
}

#[test]
fn part_of_a_stack_is_not_shown_on_the_client() {
    let mut book = RequestBook::new();
    let moves = [RequestMove::Place { from: 0, to: 25, count: 3 }, RequestMove::Drop { from: 1, count: 2 }];
    book.start(1, &moves, false, true, false).unwrap();

    let first = tick(&mut book, 0.0, false);

    assert!(matches!(first.steps.as_slice(), [RequestStep::SendRequests(_)]), "{:?}", first.steps);
}

#[test]
fn nothing_is_shown_without_the_predict_option() {
    let mut book = RequestBook::new();
    book.start(1, &swaps(2), false, false, false).unwrap();

    let first = tick(&mut book, 0.0, false);

    assert!(matches!(first.steps.as_slice(), [RequestStep::SendRequests(_)]), "{:?}", first.steps);
}

#[test]
fn a_whole_stack_into_an_empty_slot_is_shown() {
    let mut book = RequestBook::new();
    book.start(1, &[RequestMove::Place { from: 0, to: 25, count: 8 }], false, true, false).unwrap();

    let first = tick(&mut book, 0.0, false);

    assert_eq!(first.steps.get(1), Some(&RequestStep::Predict { a: 0, b: 25 }));
}

#[test]
fn a_request_that_cannot_run_is_refused_when_it_starts() {
    let mut book = RequestBook::new();

    assert_eq!(book.start(1, &[], false, false, false), Err(RequestRefusal::Empty));
    assert_eq!(book.start(1, &swaps(65), false, false, false), Err(RequestRefusal::TooMany));
    assert_eq!(book.start(1, &[RequestMove::Swap { a: 0, b: 36 }], false, false, false), Err(RequestRefusal::BadMove));
    assert_eq!(book.start(1, &[RequestMove::Swap { a: 2, b: 2 }], false, false, false), Err(RequestRefusal::BadMove));
    assert_eq!(
        book.start(1, &[RequestMove::Place { from: 0, to: 1, count: 0 }], false, false, false),
        Err(RequestRefusal::BadMove)
    );
    assert_eq!(book.start(1, &[RequestMove::Pick], false, false, false), Err(RequestRefusal::Unsupported));
    // None of them took the book.
    assert!(book.start(1, &swaps(1), false, false, false).is_ok());
}

#[test]
fn a_world_leave_ends_the_request_cancelled_and_sends_nothing() {
    let mut book = RequestBook::new();
    let (id, steps) = sent(&mut book, &swaps(2));
    book.on_responses(&answers(&steps, 2, &[true, false]));

    let done = book.world_left();

    assert_eq!(done, vec![RequestDone { id, outcome: AmmlRequestOutcome::CANCELLED, accepted: 1, total: 2 }]);
    // The book is free and holds no window for the answer to a close that was never sent.
    assert!(tick(&mut book, 0.3, false).steps.is_empty());
    assert!(!book.on_container_close(0.3, 5));
    assert!(book.start(1, &swaps(1), false, false, false).is_ok());
}

#[test]
fn a_world_leave_with_no_request_or_a_reported_end_gives_no_event() {
    let mut book = RequestBook::new();
    assert!(book.world_left().is_empty());
    let (_, _) = sent(&mut book, &swaps(1));
    // A screen opened after the send: the end was reported already.
    assert_eq!(tick(&mut book, 0.3, true).done.len(), 1);

    assert!(book.world_left().is_empty());
}

#[test]
fn a_failed_send_of_the_requests_ends_not_sent_and_closes_the_inventory() {
    let mut book = RequestBook::new();
    let (id, _) = sent(&mut book, &swaps(2));

    let end = book.send_failed(0.25);

    assert_eq!(end.steps, vec![RequestStep::SendClose(5)]);
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::NOT_SENT, 0, 2)]);
    assert!(book.on_container_close(0.3, 5));
    assert!(book.start(1, &swaps(1), false, false, false).is_ok());
}

#[test]
fn a_failed_interact_ends_not_sent_with_no_close() {
    let mut book = RequestBook::new();
    let id = book.start(1, &swaps(1), true, true, false).unwrap();
    assert_eq!(tick(&mut book, 0.0, false).steps, vec![RequestStep::SendInteract]);

    let end = book.send_failed(0.0);

    assert!(end.steps.is_empty());
    assert_eq!(end.done, vec![done(id, AmmlRequestOutcome::NOT_SENT, 0, 1)]);
    // No open is coming: a chest open is the game's.
    assert!(!book.on_container_open(0.1, 5, ProtocolValues::INVENTORY_TYPE));
}

#[test]
fn request_ids_grow_and_wire_ids_are_odd_negative() {
    let mut book = RequestBook::new();
    let first = book.start(1, &swaps(2), false, false, false).unwrap();
    let steps = tick(&mut book, 0.0, false).steps;
    let ids = wire_ids(&steps, 2);
    book.disable(None, 0.1, false);
    let second = book.start(1, &swaps(1), false, false, false).unwrap();

    assert!(second > first);
    assert_eq!(ids, vec![-100001, -100003]);
}

#[test]
fn the_book_is_idle_when_nothing_runs_and_every_window_has_passed() {
    let mut book = RequestBook::new();
    assert!(book.is_idle());
    let (_, steps) = sent(&mut book, &swaps(1));
    assert!(!book.is_idle());
    book.on_responses(&answers(&steps, 1, &[true]));
    tick(&mut book, 0.3, false);
    // The request is over, but the server's answer to our close may still come.
    assert!(!book.is_idle());

    tick(&mut book, 2.4, false);

    assert!(book.is_idle());
}

#[test]
fn the_slots_are_wanted_only_when_the_moves_go_out_at_the_next_tick() {
    let mut book = RequestBook::new();
    assert!(!book.wants_slots());
    book.start(1, &swaps(1), true, false, false).unwrap();
    // The Interact goes first.
    assert!(!book.wants_slots());
    tick(&mut book, 0.0, false);
    assert!(!book.wants_slots());
    assert!(book.on_container_open(0.1, 5, ProtocolValues::INVENTORY_TYPE));
    assert!(book.wants_slots());
    tick(&mut book, 0.2, false);
    // The moves are sent; the answers need no read.
    assert!(!book.wants_slots());
    let mut plain = RequestBook::new();
    plain.start(1, &swaps(1), false, false, false).unwrap();
    assert!(plain.wants_slots());
}
