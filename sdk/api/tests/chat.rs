use std::sync::{Arc, Mutex};

use amml_abi::ChatMessageAdded;
use amml_api::testing::{FakeChat, FakeGame};
use amml_api::{ApiError, Area, ChatKind, ChatMessageAddedExt, HookOrder, ImageView, InlineIcon, PixelFormat};

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<ChatMessageAdded>();
    fake.offer_area(Area::UI_INLINE_ICONS);
    fake.set_bitmap_font(true);
    fake.open_gate();
    fake
}

fn line() -> FakeChat {
    FakeChat {
        kind: ChatKind(1),
        username: "Alex".to_string(),
        xuid: "2535400000000001".to_string(),
        text: "<Alex> hi".to_string(),
        filtered_text: Some("<Alex> ***".to_string()),
    }
}

fn pixels(side: u32) -> Vec<u8> {
    vec![255; (side * side * 4) as usize]
}

fn view(side: u32, bytes: &[u8]) -> ImageView<'_> {
    ImageView { width: side, height: side, format: PixelFormat::Rgba8, pixels: bytes }
}

#[test]
fn reads_give_the_line_as_built() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            sink.lock().unwrap().push(format!(
                "{} {:?} {:?} {:?} {:?}",
                event.kind().0,
                event.username(),
                event.xuid(),
                event.text(),
                event.filtered_text()
            ));
        })
        .unwrap();

    fake.raise_chat(&line());

    assert_eq!(
        *seen.lock().unwrap(),
        ["1 Some(\"Alex\") Some(\"2535400000000001\") Some(\"<Alex> hi\") Some(\"<Alex> ***\")"]
    );
}

#[test]
fn set_text_is_applied_after_the_listeners() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let first = seen.clone();
    let _first = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            event.set_text("edited").unwrap();
            event.set_filtered_text("edited filtered").unwrap();
            // The line is read as the game built it: the edit is not seen, not even by the listener that made it.
            first.lock().unwrap().push(format!("{:?} {:?}", event.text(), event.filtered_text()));
        })
        .unwrap();
    let second = seen.clone();
    let _second = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::LAST, move |event, _| {
            second.lock().unwrap().push(format!("{:?}", event.text()));
            event.set_text("edited last").unwrap();
        })
        .unwrap();

    let (text, filtered) = fake.raise_chat(&line());

    assert_eq!(*seen.lock().unwrap(), ["Some(\"<Alex> hi\") Some(\"<Alex> ***\")", "Some(\"<Alex> hi\")"]);
    // The latest edit of a field wins.
    assert_eq!(text.as_deref(), Some("edited last"));
    assert_eq!(filtered.as_deref(), Some("edited filtered"));
}

#[test]
fn a_line_nobody_edits_comes_out_as_it_went_in() {
    let fake = serving();
    let _listener = fake.ctx().events().listen::<ChatMessageAdded>(HookOrder::NORMAL, |_, _| {}).unwrap();

    assert_eq!(fake.raise_chat(&line()), (Some("<Alex> hi".to_string()), Some("<Alex> ***".to_string())));
}

#[test]
fn a_line_with_no_filtered_text_reads_none_and_refuses_a_filtered_edit() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            sink.lock().unwrap().push(format!("{:?} {:?}", event.filtered_text(), event.set_filtered_text("x")));
        })
        .unwrap();
    let mut plain = line();
    plain.filtered_text = None;

    let (text, filtered) = fake.raise_chat(&plain);

    assert_eq!(*seen.lock().unwrap(), ["None Err(NotAvailable)"]);
    assert_eq!((text.as_deref(), filtered), (Some("<Alex> hi"), None));
}

#[test]
fn set_text_over_4_kib_is_refused() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            let mut results = sink.lock().unwrap();
            results.push(format!("{:?}", event.set_text(&"x".repeat(4097))));
            results.push(format!("{:?}", event.set_filtered_text(&"y".repeat(4096))));
        })
        .unwrap();

    let (text, filtered) = fake.raise_chat(&line());

    assert_eq!(*seen.lock().unwrap(), ["Err(Refused)", "Ok(())"]);
    assert_eq!(text.as_deref(), Some("<Alex> hi"));
    assert_eq!(filtered, Some("y".repeat(4096)));
}

#[test]
fn a_copy_of_the_event_kept_past_its_call_reads_nothing() {
    let fake = serving();
    let kept = Arc::new(Mutex::new(None));
    let sink = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            *sink.lock().unwrap() = Some(*event);
        })
        .unwrap();

    fake.raise_chat(&line());

    let event = kept.lock().unwrap().unwrap();
    assert_eq!(event.kind(), ChatKind(1));
    assert_eq!((event.username(), event.xuid(), event.text(), event.filtered_text()), (None, None, None, None));
    assert_eq!(event.set_text("late"), Err(ApiError::Refused));
    assert!(!event.inline_icons());
}

#[test]
fn inline_icons_follow_the_area_and_the_font() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ChatMessageAdded>(HookOrder::NORMAL, move |event, _| {
            sink.lock().unwrap().push(event.inline_icons());
        })
        .unwrap();

    fake.raise_chat(&line());
    fake.set_bitmap_font(false);
    fake.raise_chat(&line());
    fake.set_bitmap_font(true);
    fake.withdraw_area(Area::UI_INLINE_ICONS);
    fake.raise_chat(&line());

    assert_eq!(*seen.lock().unwrap(), [true, false, false]);
}

#[test]
fn icon_glyph_is_a_private_use_char() {
    let fake = serving();
    let bytes = pixels(8);
    let chat = fake.ctx().chat();

    let first = chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap();
    let second = chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap();

    assert_eq!(first.glyph(), '\u{E110}');
    assert_eq!(second.glyph(), '\u{E111}');
    assert!(('\u{E110}'..='\u{E1FF}').contains(&first.glyph()));
    assert_eq!(fake.icons(), ['\u{E110}', '\u{E111}']);
}

#[test]
fn register_without_bitmap_font_is_not_available() {
    let fake = serving();
    fake.set_bitmap_font(false);
    let bytes = pixels(8);

    let result = fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0);

    assert_eq!(result.err(), Some(ApiError::NotAvailable));
    assert!(fake.icons().is_empty());
}

#[test]
fn register_without_the_area_is_not_available() {
    let fake = FakeGame::new();
    fake.set_bitmap_font(true);
    fake.open_gate();
    let bytes = pixels(8);

    assert_eq!(fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0).err(), Some(ApiError::NotAvailable));
}

#[test]
fn the_limits_of_an_icon_are_refused() {
    let fake = serving();
    let chat = fake.ctx().chat();
    let bytes = pixels(8);
    let big = pixels(17);

    assert_eq!(chat.register_inline_icon(&view(8, &bytes), 16.5).err(), Some(ApiError::Refused));
    assert_eq!(chat.register_inline_icon(&view(8, &bytes), 0.0).err(), Some(ApiError::Refused));
    assert_eq!(chat.register_inline_icon(&view(17, &big), 9.0).err(), Some(ApiError::Refused));
    let wrong_size = ImageView { width: 8, height: 8, format: PixelFormat::Rgba8, pixels: &bytes[..16] };
    assert!(matches!(chat.register_inline_icon(&wrong_size, 9.0), Err(ApiError::Loader(_))));
    let other = ImageView { width: 8, height: 8, format: PixelFormat::Other(3), pixels: &bytes };
    assert!(matches!(chat.register_inline_icon(&other, 9.0), Err(ApiError::Loader(_))));
    let held: Vec<InlineIcon> = (0..240).map(|_| chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap()).collect();
    assert_eq!(chat.register_inline_icon(&view(8, &bytes), 9.0).err(), Some(ApiError::Refused));
    assert_eq!(held.len(), 240);
}

#[test]
fn update_changes_the_image_and_release_frees_the_icon() {
    let fake = serving();
    let chat = fake.ctx().chat();
    let bytes = pixels(8);
    let big = pixels(17);
    let icon = chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap();

    assert_eq!(icon.update(&view(4, &pixels(4))), Ok(()));
    assert_eq!(icon.update(&view(17, &big)), Err(ApiError::Refused));
    icon.release();

    assert!(fake.icons().is_empty());
}

#[test]
fn dropping_an_icon_frees_it() {
    let fake = serving();
    let bytes = pixels(8);
    let icon = fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0).unwrap();
    assert_eq!(fake.icons().len(), 1);

    drop(icon);

    assert!(fake.icons().is_empty());
}

#[test]
fn an_icon_dropped_off_the_game_thread_queues_the_free() {
    let fake = serving();
    let bytes = pixels(8);
    let icon = fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0).unwrap();
    let kept = fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0).unwrap();

    std::thread::scope(|scope| {
        scope.spawn(move || drop(icon)).join().unwrap();
    });

    // The free is queued: the icon is still counted until the loader's next UI draw.
    assert_eq!(fake.icons().len(), 2);
    assert_eq!(fake.pump_icons(), 1);
    assert_eq!(fake.icons(), [kept.glyph()]);
}

#[test]
fn use_after_a_world_leave_is_stale() {
    let fake = serving();
    let bytes = pixels(8);
    let chat = fake.ctx().chat();
    let icon = chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap();

    fake.leave_icon_world();

    assert_eq!(icon.update(&view(8, &bytes)), Err(ApiError::Stale));
    assert!(fake.icons().is_empty());
    // Dropping a stale icon is allowed, and an icon of the new generation works.
    drop(icon);
    let fresh = chat.register_inline_icon(&view(8, &bytes), 9.0).unwrap();
    assert_eq!(fresh.update(&view(8, &bytes)), Ok(()));
}

#[test]
fn an_icon_can_move_to_another_thread() {
    fn assert_send<T: Send>() {}
    assert_send::<InlineIcon>();
}

#[test]
fn a_loader_without_the_chat_functions_gives_not_available() {
    let fake = FakeGame::with_v5_services();
    let bytes = pixels(8);
    fake.offer_area(Area::UI_INLINE_ICONS);

    assert_eq!(fake.ctx().chat().register_inline_icon(&view(8, &bytes), 9.0).err(), Some(ApiError::NotAvailable));
}
