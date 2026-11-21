use std::sync::{Arc, Mutex};

use amml_abi::{AmmlStatus, ClientTick, InventoryRequestDone};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{
    ApiError, Area, HookOrder, InventoryMove, InventoryRequestDoneExt, RequestId, RequestOptions, RequestOutcome,
    Screen,
};

type Asked = Arc<Mutex<Vec<Result<RequestId, ApiError>>>>;
type Ended = Arc<Mutex<Vec<(RequestId, RequestOutcome, u16, u16)>>>;

fn client(top_screen: Screen) -> FakeClient {
    FakeClient { local_player: true, world_active: true, top_screen, ..FakeClient::default() }
}

fn serving(fake: &FakeGame, top_screen: Screen) {
    fake.offer::<ClientTick>();
    fake.offer::<InventoryRequestDone>();
    fake.offer_area(Area::CLIENT_READ);
    fake.offer_area(Area::INVENTORY_READ);
    fake.offer_area(Area::INVENTORY_REQUEST);
    fake.open_gate();
    fake.set_client(client(top_screen));
}

// Runs `ask` on the inventory of the local player inside a `ClientTick` listener and gives its result.
fn ask(
    fake: &FakeGame,
    ask: impl Fn(&amml_api::Inventory<'_>) -> Result<RequestId, ApiError> + Send + 'static,
) -> Result<RequestId, ApiError> {
    let asked: Asked = Arc::new(Mutex::new(Vec::new()));
    let record = asked.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let inventory = at.client().and_then(|client| client.local_player()).and_then(|player| player.inventory());
            record.lock().unwrap().push(ask(&inventory.unwrap()));
        })
        .unwrap();
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    asked.lock().unwrap().pop().unwrap()
}

fn listen_for_ends(fake: &FakeGame) -> (Ended, amml_api::Listener) {
    let ended: Ended = Arc::new(Mutex::new(Vec::new()));
    let record = ended.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<InventoryRequestDone>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.id(), event.outcome(), event.accepted(), event.total()));
        })
        .unwrap();
    (ended, listener)
}

#[test]
fn request_swap_records_one_swap() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);

    let id = ask(&fake, |inventory| inventory.request_swap(0, 9)).unwrap();

    assert_eq!(
        fake.requests(),
        vec![(id, vec![InventoryMove::Swap { a: 0, b: 9 }], RequestOptions { hidden_open: true, predict: true })]
    );
}

#[test]
fn request_moves_records_the_moves_and_the_options() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Container);
    let moves = [InventoryMove::Place { from: 1, to: 20, count: 3 }, InventoryMove::Drop { from: 2, count: 1 }];
    let options = RequestOptions { hidden_open: false, predict: false };

    let id = ask(&fake, move |inventory| inventory.request_moves(&moves, options)).unwrap();

    assert_eq!(fake.requests(), vec![(id, moves.to_vec(), options)]);
}

#[test]
fn done_comes_at_the_next_frame() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);
    let (ended, _listener) = listen_for_ends(&fake);
    let id = ask(&fake, |inventory| inventory.request_swap(0, 9)).unwrap();

    // Nothing ends inside the call, nor before the server answers.
    assert!(ended.lock().unwrap().is_empty());
    fake.frame();
    assert!(ended.lock().unwrap().is_empty());
    fake.answer(id, 1);
    assert!(ended.lock().unwrap().is_empty());
    fake.frame();

    assert_eq!(*ended.lock().unwrap(), vec![(id, RequestOutcome::Done, 1, 1)]);
    // The event comes once.
    fake.frame();
    assert_eq!(ended.lock().unwrap().len(), 1);
}

#[test]
fn a_refused_answer_gives_the_refused_outcome() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);
    let (ended, _listener) = listen_for_ends(&fake);
    let moves = [InventoryMove::Swap { a: 0, b: 1 }, InventoryMove::Swap { a: 2, b: 3 }];
    let id = ask(&fake, move |inventory| inventory.request_moves(&moves, RequestOptions::default())).unwrap();

    fake.answer(id, 1);
    fake.frame();

    assert_eq!(*ended.lock().unwrap(), vec![(id, RequestOutcome::Refused, 1, 2)]);
}

#[test]
fn refused_while_another_request_runs() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);
    let first = ask(&fake, |inventory| inventory.request_swap(0, 9)).unwrap();

    assert_eq!(ask(&fake, |inventory| inventory.request_swap(1, 2)), Err(ApiError::Refused));
    fake.answer(first, 1);
    fake.frame();
    assert!(ask(&fake, |inventory| inventory.request_swap(1, 2)).is_ok());
}

#[test]
fn the_hidden_open_is_refused_while_a_screen_is_open() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Container);

    assert_eq!(ask(&fake, |inventory| inventory.request_swap(0, 9)), Err(ApiError::Refused));
    assert!(fake.requests().is_empty());
}

#[test]
fn a_request_that_cannot_run_is_refused_by_the_call() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);
    let options = RequestOptions::default();

    assert_eq!(
        ask(&fake, move |inventory| inventory.request_moves(&[], options)),
        Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT))
    );
    assert_eq!(
        ask(&fake, move |inventory| inventory.request_moves(&[InventoryMove::Swap { a: 0, b: 36 }], options)),
        Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT))
    );
    assert_eq!(
        ask(&fake, move |inventory| inventory.request_moves(&[InventoryMove::Drop { from: 0, count: 0 }], options)),
        Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT))
    );
    assert!(fake.requests().is_empty());
}

#[test]
fn a_loader_without_the_area_gives_not_available() {
    let fake = FakeGame::new();
    serving(&fake, Screen::Hud);
    fake.withdraw_area(Area::INVENTORY_REQUEST);

    assert_eq!(ask(&fake, |inventory| inventory.request_swap(0, 9)), Err(ApiError::NotAvailable));
}

#[test]
fn a_loader_before_inventory_v5_gives_not_available() {
    let fake = FakeGame::with_v4_inventory();
    serving(&fake, Screen::Hud);

    assert_eq!(ask(&fake, |inventory| inventory.request_swap(0, 9)), Err(ApiError::NotAvailable));
}

#[test]
fn an_outcome_the_crate_does_not_name_is_other() {
    assert_eq!(RequestOutcome::from(9), RequestOutcome::Other(9));
    assert_eq!(RequestOutcome::from(1), RequestOutcome::Done);
    assert_eq!(RequestOutcome::from(5), RequestOutcome::Cancelled);
    assert_eq!(RequestOutcome::from(7), RequestOutcome::NotSent);
}
