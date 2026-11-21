use std::sync::{Arc, Mutex};

use amml_abi::{AmmlApi, AmmlScreenKind, AmmlServices, AmmlStatus, ScreenChanged, ScreenRender};
use amml_api::testing::FakeGame;
use amml_api::testing::fake_screen::FakeScreen;
use amml_api::{
    Area, HookOrder, InputMode, Point, Rect, Screen, ScreenChangedExt, ScreenId, ScreenKind, ScreenRenderExt, Size,
    UiProfile, ViewId,
};

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.offer::<ScreenChanged>();
    fake.offer_area(Area::UI_SCREEN);
    fake.open_gate();
    fake
}

fn chest() -> FakeScreen {
    FakeScreen {
        kind: ScreenKind::SmallChest,
        size: Some(Size { width: 427, height: 240 }),
        content_rect: Some(Rect { x: 100.0, y: 50.0, width: 176.0, height: 166.0 }),
        pointer: Some(Point { x: 12.5, y: 30.0 }),
        input_mode: Some(InputMode::Mouse),
        ui_profile: Some(UiProfile::Classic),
        gui_scale: Some(2.0),
        can_draw: true,
        ..FakeScreen::default()
    }
}

#[test]
fn view_reads_come_from_the_fake_screen() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let view = event.view().unwrap();
            *record.lock().unwrap() = Some((
                event.screen(),
                event.view_id(),
                event.tick(),
                view.kind(),
                view.size(),
                view.content_rect(),
                view.pointer(),
                view.input_mode(),
                view.ui_profile(),
            ));
        })
        .unwrap();

    let dispatch = fake.raise_screen_render(ScreenId(7), ViewId(9), &chest());

    assert!(!dispatch.panicked);
    let got = seen.lock().unwrap().take().unwrap();
    assert_eq!(got.0, ScreenId(7));
    assert_eq!(got.1, ViewId(9));
    assert_eq!(got.2, 0);
    assert_eq!(got.3, ScreenKind::SmallChest);
    assert_eq!(got.4, Some(Size { width: 427, height: 240 }));
    assert_eq!(got.5, Some(Rect { x: 100.0, y: 50.0, width: 176.0, height: 166.0 }));
    assert_eq!(got.6, Some(Point { x: 12.5, y: 30.0 }));
    assert_eq!(got.7, Some(InputMode::Mouse));
    assert_eq!(got.8, Some(UiProfile::Classic));
}

#[test]
fn a_screen_that_cannot_be_read_gives_none_and_other() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let view = event.view().unwrap();
            *record.lock().unwrap() = Some((view.kind(), view.size(), view.content_rect(), view.pointer()));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &FakeScreen::default());

    assert_eq!(seen.lock().unwrap().take(), Some((ScreenKind::Other, None, None, None)));
}

#[test]
fn view_kept_after_the_listener_gives_none() {
    let fake = serving();
    let kept = Arc::new(Mutex::new(None));
    let record = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some((*event, event.view_id()));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(7), ViewId(9), &chest());

    let (copy, view) = kept.lock().unwrap().take().unwrap();
    // The copy of the event is not the event of a running call.
    assert!(copy.view().is_none());
    // The kept id is refused by the table: the dispatch is over.
    // SAFETY: the fake tables live as long as `fake`.
    let services = unsafe { AmmlApi::services(fake.api_table()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { &*AmmlServices::ui(services).unwrap() };
    let mut kind = AmmlScreenKind::OTHER;
    // SAFETY: the table is live and `kind` is a live local.
    assert_eq!(unsafe { (table.view_kind)(view.0, &mut kind) }, AmmlStatus::REFUSED);
}

#[test]
fn without_the_ui_area_the_view_gives_none() {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some(event.view().map(|view| view.size()));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(7), ViewId(9), &chest());

    // The scope opens, but the loader answers `NOT_AVAILABLE` for each read.
    assert_eq!(seen.lock().unwrap().take(), Some(Some(None)));
}

#[test]
fn a_v6_loader_has_no_ui_area() {
    let fake = FakeGame::with_v6_services();
    fake.offer_area(Area::UI_SCREEN);

    assert!(!fake.ctx().supports(Area::UI_SCREEN));
}

#[test]
fn screen_changed_reads_old_and_new() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenChanged>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some((event.old(), event.new(), event.screen()));
        })
        .unwrap();

    fake.raise_screen_changed(ScreenId(3), ScreenId::NONE, Screen::Container);

    assert_eq!(seen.lock().unwrap().take(), Some((ScreenId(3), ScreenId::NONE, Screen::Container)));
}

#[test]
fn screen_closing_reads_the_screen() {
    let fake = serving();
    fake.offer::<amml_abi::ScreenClosing>();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<amml_abi::ScreenClosing>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some(amml_api::ScreenClosingExt::screen(event));
        })
        .unwrap();

    fake.raise_screen_closing(ScreenId(4));

    assert_eq!(seen.lock().unwrap().take(), Some(ScreenId(4)));
}
