use std::sync::{Arc, Mutex};

use amml_abi::{ItemHovertext, UiPointer, UiScroll};
use amml_api::testing::FakeGame;
use amml_api::{
    Cancel, HookOrder, HovertextKind, InputMode, ItemHovertextExt, Point, ScreenId, UiPointerExt, UiScrollExt, ViewId,
};

#[test]
fn cancelled_pointer_does_not_reach_the_game() {
    let fake = FakeGame::new();
    fake.offer::<UiPointer>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<UiPointer>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((
                event.view(),
                event.pointer(),
                event.pressed(),
                event.position(),
                event.input_mode(),
                event.screen(),
            ));
            if event.pressed() && event.pointer() == 0 {
                event.cancel();
            }
        })
        .unwrap();

    // The down is cancelled, so the game does not get it; the up of that press is held back too.
    assert!(!fake.ui_pointer(ViewId(3), 0, true, 10.0, 20.0));
    assert!(!fake.ui_pointer(ViewId(3), 0, false, 10.0, 20.0));
    // The second pointer is not cancelled, and a new press of the first goes the same way as before.
    assert!(fake.ui_pointer(ViewId(3), 1, true, 1.0, 2.0));
    assert!(fake.ui_pointer(ViewId(3), 1, false, 1.0, 2.0));
    assert!(!fake.ui_pointer(ViewId(3), 0, true, 0.0, 0.0));

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 5);
    assert_eq!(seen[0], (ViewId(3), 0, true, Point { x: 10.0, y: 20.0 }, InputMode::Mouse, ScreenId::NONE));
    assert!(!seen[1].2);
}

#[test]
fn a_pointer_without_a_listener_reaches_the_game() {
    let fake = FakeGame::new();
    fake.offer::<UiPointer>();

    assert!(fake.ui_pointer(ViewId(1), 0, true, 0.0, 0.0));
    assert!(fake.ui_pointer(ViewId(1), 0, false, 0.0, 0.0));
}

#[test]
fn the_up_of_a_cancelled_down_is_raised_cancelled() {
    let fake = FakeGame::new();
    fake.offer::<UiPointer>();
    let cancelled = Arc::new(Mutex::new(Vec::new()));
    let record = cancelled.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<UiPointer>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.pressed(), amml_api::Event::is_cancelled(&*event)));
            if event.pressed() {
                event.cancel();
            }
        })
        .unwrap();

    fake.ui_pointer(ViewId(1), 0, true, 0.0, 0.0);
    fake.ui_pointer(ViewId(1), 0, false, 0.0, 0.0);

    assert_eq!(*cancelled.lock().unwrap(), [(true, false), (false, true)]);
}

#[test]
fn scroll_cancel_eats_the_notch() {
    let fake = FakeGame::new();
    fake.offer::<UiScroll>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<UiScroll>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.screen(), event.notches()));
            if event.notches() < 0 {
                event.cancel();
            }
        })
        .unwrap();

    assert!(fake.raise_scroll(Some(ScreenId(4)), -1));
    assert!(!fake.raise_scroll(Some(ScreenId(4)), 1));
    assert!(!fake.raise_scroll(None, 1));

    assert_eq!(*seen.lock().unwrap(), [(Some(ScreenId(4)), -1), (Some(ScreenId(4)), 1), (None, 1)]);
}

#[test]
fn hovertext_hide_sets_zero_lines() {
    let fake = FakeGame::new();
    fake.offer::<ItemHovertext>();
    let kinds = Arc::new(Mutex::new(Vec::new()));
    let record = kinds.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ItemHovertext>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.kind());
            event.hide_contents();
        })
        .unwrap();

    assert_eq!(fake.raise_hovertext(), 0);
    assert_eq!(*kinds.lock().unwrap(), [HovertextKind::ContainerItem]);
}

#[test]
fn hovertext_without_listener_keeps_one() {
    let fake = FakeGame::new();
    fake.offer::<ItemHovertext>();

    assert_eq!(fake.raise_hovertext(), 1);
}

#[test]
fn a_kind_the_crate_does_not_name_is_other() {
    assert_eq!(HovertextKind::from(7), HovertextKind::Other(7));
    assert_eq!(HovertextKind::from(ItemHovertext::CONTAINER_ITEM), HovertextKind::ContainerItem);
}
