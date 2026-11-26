use std::sync::{Arc, Mutex};

use amml_abi::{
    AmmlModifiers, AmmlMouseButton, AmmlPoint, AmmlPointerKind, AmmlStatus, WindowChar, WindowPointer, WindowSize,
};
use amml_api::testing::{FakeGame, FakeWindow};
use amml_api::{
    ApiError, Area, Cancel, CursorClip, HookOrder, MouseButton, Point, PointerKind, Size, WindowCharExt,
    WindowPointerExt, WindowSizeExt,
};

fn pointer(kind: AmmlPointerKind, button: AmmlMouseButton, down: bool) -> WindowPointer {
    WindowPointer {
        header: FakeGame::header::<WindowPointer>(),
        kind,
        button,
        down: u8::from(down),
        _reserved: 0,
        modifiers: AmmlModifiers(0),
        _reserved2: 0,
        client: AmmlPoint { x: 10.0, y: 20.0 },
        buffer: AmmlPoint { x: 20.0, y: 40.0 },
        notches: 0.0,
    }
}

#[test]
fn a_loader_without_a_window_table_gives_none() {
    let fake = FakeGame::with_v2_services();
    fake.offer_area(Area::WINDOW_READ);
    fake.offer_area(Area::WINDOW_REQUEST_SIZE);
    fake.set_window(FakeWindow::default());
    let window = fake.ctx().window();

    assert_eq!(window.client_size(), None);
    assert_eq!(window.dpi(), None);
    assert_eq!(window.request_size(Some(Size { width: 640, height: 360 })), Err(ApiError::NotAvailable));
    assert_eq!(window.replay_focus_loss(), Err(ApiError::NotAvailable));
    assert!(!fake.ctx().supports(Area::WINDOW_READ));
    assert!(fake.window_requests().is_empty());
}

#[test]
fn window_reads_come_from_the_table() {
    let fake = FakeGame::new();
    fake.offer_area(Area::WINDOW_READ);
    fake.set_window(FakeWindow {
        client: Size { width: 1280, height: 720 },
        buffer: Size { width: 2560, height: 1440 },
        focus: false,
        dpi: 144,
        cursor: Point { x: 5.0, y: 6.0 },
    });
    let window = fake.ctx().window();

    assert_eq!(window.client_size(), Some(Size { width: 1280, height: 720 }));
    assert_eq!(window.buffer_size(), Some(Size { width: 2560, height: 1440 }));
    assert_eq!(window.to_buffer(Point { x: 10.0, y: 20.0 }), Some(Point { x: 20.0, y: 40.0 }));
    assert_eq!(window.cursor_pos(), Some(Point { x: 5.0, y: 6.0 }));
    assert_eq!(window.has_focus(), Some(false));
    assert_eq!(window.is_foreground(), Some(false));
    assert_eq!(window.dpi(), Some(144));
    assert!(fake.ctx().supports(Area::WINDOW_READ));
}

#[test]
fn window_calls_need_their_area_and_an_attached_window() {
    let fake = FakeGame::new();
    let window = fake.ctx().window();
    let size = Some(Size { width: 640, height: 360 });

    fake.offer_area(Area::WINDOW_READ);
    fake.offer_area(Area::WINDOW_REQUEST_SIZE);
    assert_eq!(window.client_size(), None);
    assert_eq!(window.request_size(size), Err(ApiError::NotReady));

    fake.set_window(FakeWindow { buffer: Size::default(), ..FakeWindow::default() });
    assert_eq!(window.client_size(), Some(Size { width: 1280, height: 720 }));
    assert_eq!(window.buffer_size(), None, "the back buffer is not known yet");
    assert_eq!(window.to_buffer(Point { x: 1.0, y: 1.0 }), None);

    fake.withdraw_area(Area::WINDOW_REQUEST_SIZE);
    assert_eq!(window.request_size(size), Err(ApiError::NotAvailable));
    assert!(fake.window_requests().is_empty());
}

#[test]
fn requests_reach_the_loader_and_a_zero_side_is_refused() {
    let fake = FakeGame::new();
    fake.offer_area(Area::WINDOW_READ);
    fake.offer_area(Area::WINDOW_REQUEST_SIZE);
    fake.set_window(FakeWindow::default());
    let window = fake.ctx().window();

    assert_eq!(window.request_size(Some(Size { width: 640, height: 360 })), Ok(()));
    assert_eq!(window.request_size(None), Ok(()));
    assert_eq!(
        window.request_size(Some(Size { width: 0, height: 360 })),
        Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT))
    );
    assert_eq!(window.replay_focus_loss(), Ok(()));

    assert_eq!(fake.window_requests(), ["request_size 640x360", "request_size none", "replay_focus_loss"]);
}

#[test]
fn a_lone_high_surrogate_is_no_char() {
    let lone = WindowChar { header: FakeGame::header::<WindowChar>(), ch: 0xD800 };
    let typed = WindowChar { header: FakeGame::header::<WindowChar>(), ch: u32::from('a') };
    let astral = WindowChar { header: FakeGame::header::<WindowChar>(), ch: 0x1F600 };

    assert_eq!(lone.ch(), None);
    assert_eq!(typed.ch(), Some('a'));
    assert_eq!(astral.ch(), Some('\u{1F600}'));
}

#[test]
fn a_button_reads_only_on_a_button_event_and_a_listener_can_cancel_it() {
    let fake = FakeGame::new();
    fake.offer::<WindowPointer>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WindowPointer>(HookOrder::FIRST, move |event, _| {
            record.lock().unwrap().push((event.kind(), event.button(), event.down(), event.client(), event.buffer()));
            if event.kind() == PointerKind::Button {
                event.cancel();
            }
        })
        .unwrap();

    let click = fake.raise(&mut pointer(AmmlPointerKind::BUTTON, AmmlMouseButton::RIGHT, true));
    // The loader writes `LEFT` and the down flag on a move; neither means anything there.
    let moved = fake.raise(&mut pointer(AmmlPointerKind::MOVE, AmmlMouseButton::LEFT, true));

    assert!(click.cancelled);
    assert!(!moved.cancelled);
    let at = (Point { x: 10.0, y: 20.0 }, Point { x: 20.0, y: 40.0 });
    assert_eq!(
        *seen.lock().unwrap(),
        [
            (PointerKind::Button, Some(MouseButton::Right), true, at.0, at.1),
            (PointerKind::Move, None, false, at.0, at.1),
        ]
    );
}

#[test]
fn a_written_window_size_reaches_later_listeners() {
    let fake = FakeGame::new();
    fake.offer::<WindowSize>();
    let ctx = fake.ctx();
    let _write = ctx
        .events()
        .listen::<WindowSize>(HookOrder::WRITE, |event, _| event.set_size(Size { width: 800, height: 600 }))
        .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _read = ctx
        .events()
        .listen::<WindowSize>(HookOrder::READ_AFTER_WRITERS, move |event, _| {
            record.lock().unwrap().push(event.size());
        })
        .unwrap();
    let mut event = WindowSize {
        header: FakeGame::header::<WindowSize>(),
        minimised: 0,
        _reserved: [0; 3],
        width: 1280,
        height: 720,
    };

    fake.raise(&mut event);

    assert_eq!(*seen.lock().unwrap(), [Size { width: 800, height: 600 }]);
    assert_eq!(event.size(), Size { width: 800, height: 600 });
}

#[test]
fn clip_cursor_is_recorded() {
    let fake = FakeGame::new();
    let window = fake.ctx().window();
    assert_eq!(window.clip_cursor(Some(CursorClip::Centre)), Err(ApiError::NotAvailable), "the area is not offered");

    fake.offer_area(Area::WINDOW_CURSOR_CLIP);
    assert_eq!(window.clip_cursor(Some(CursorClip::Centre)), Err(ApiError::NotReady), "no window is attached");

    fake.set_window(FakeWindow::default());
    assert_eq!(window.clip_cursor(Some(CursorClip::Centre)), Ok(()));
    assert_eq!(window.clip_cursor(None), Ok(()));
    assert_eq!(fake.window_requests(), ["clip_cursor centre", "clip_cursor none"]);
}

#[test]
fn a_loader_without_a_window_table_cannot_clip() {
    let fake = FakeGame::with_v2_services();
    fake.offer_area(Area::WINDOW_CURSOR_CLIP);
    fake.set_window(FakeWindow::default());

    assert_eq!(fake.ctx().window().clip_cursor(Some(CursorClip::Centre)), Err(ApiError::NotAvailable));
    assert!(fake.window_requests().is_empty());
}
