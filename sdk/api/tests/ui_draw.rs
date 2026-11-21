use std::sync::{Arc, Mutex};

use amml_abi::ScreenRender;
use amml_api::testing::FakeGame;
use amml_api::testing::fake_screen::FakeScreen;
use amml_api::{
    ApiError, Area, Color, HookOrder, Rect, ScreenId, ScreenKind, ScreenRenderExt, TextAlign, TextureSource, Tint, Uv,
    ViewId,
};

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.offer_area(Area::UI_SCREEN);
    fake.offer_area(Area::UI_DRAW);
    fake.offer_area(Area::UI_TEXTURES);
    fake.offer_area(Area::UI_TEXT);
    fake.open_gate();
    fake
}

fn screen() -> FakeScreen {
    FakeScreen { kind: ScreenKind::SmallChest, gui_scale: Some(2.0), can_draw: true, ..FakeScreen::default() }
}

const RECT: Rect = Rect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 };
const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0 };
const UV: Uv = Uv { u0: 0.0, v0: 0.0, u1: 1.0, v1: 1.0 };

#[test]
fn fill_rect_records_one_draw() {
    let fake = serving();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, |event, _| {
            let draw = event.draw().unwrap();
            draw.fill_rect(RECT, RED, 0.5).unwrap();
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(fake.draws(), vec!["fill 10 20 30 40".to_string()]);
}

#[test]
fn outline_rect_records_one_draw() {
    let fake = serving();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, |event, _| {
            event.draw().unwrap().outline_rect(RECT, RED, 1.0, 2).unwrap();
            assert_eq!(event.draw().unwrap().outline_rect(RECT, RED, 1.0, 0), Err(ApiError::Loader(invalid())));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(fake.draws(), vec!["outline 10 20 30 40".to_string()]);
}

fn invalid() -> amml_abi::AmmlStatus {
    amml_abi::AmmlStatus::INVALID_ARGUMENT
}

#[test]
fn image_of_a_texture_not_ready_is_skipped() {
    let fake = serving();
    let textures = fake.ctx().textures();
    let texture = textures.load(TextureSource::Vanilla("textures/ui/a")).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.draw().unwrap().image(texture, RECT, UV, Tint::WHITE));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());
    assert!(fake.draws().is_empty());

    fake.textures().set_ready(texture, true);
    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(*seen.lock().unwrap(), vec![Ok(()), Ok(())]);
    assert_eq!(fake.draws(), vec![format!("image {} 10 20 30 40", texture.id)]);
}

#[test]
fn image_when_can_draw_is_false_is_refused() {
    let fake = serving();
    let texture = fake.ctx().textures().load(TextureSource::Vanilla("textures/ui/a")).unwrap();
    fake.textures().set_ready(texture, true);
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let draw = event.draw().unwrap();
            *record.lock().unwrap() = Some((draw.can_draw(), draw.image(texture, RECT, UV, Tint::WHITE)));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &FakeScreen { can_draw: false, ..screen() });

    assert_eq!(seen.lock().unwrap().take(), Some((false, Err(ApiError::Refused))));
    assert!(fake.draws().is_empty());
}

#[test]
fn the_gui_scale_comes_from_the_screen() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.draw().unwrap().gui_scale());
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());
    fake.raise_screen_render(ScreenId(1), ViewId(2), &FakeScreen { gui_scale: None, ..screen() });

    assert_eq!(*seen.lock().unwrap(), vec![Some(2.0), None]);
}

#[test]
fn a_draw_kept_after_the_listener_is_refused_and_a_copy_of_the_event_has_none() {
    let fake = serving();
    let kept = Arc::new(Mutex::new(None));
    let record = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some((*event, event.draw().is_some()));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    let (copy, had) = kept.lock().unwrap().take().unwrap();
    assert!(had);
    assert!(copy.draw().is_none());
    assert!(fake.draws().is_empty());
}

#[test]
fn without_the_draw_area_every_call_is_not_available() {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let draw = event.draw().unwrap();
            *record.lock().unwrap() = Some((draw.fill_rect(RECT, RED, 1.0), draw.can_draw(), draw.gui_scale()));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(seen.lock().unwrap().take(), Some((Err(ApiError::NotAvailable), false, None)));
    assert!(!fake.ctx().supports(Area::UI_DRAW));
}

#[test]
fn a_v6_loader_has_no_draw() {
    let fake = FakeGame::with_v6_services();
    fake.offer::<ScreenRender>();
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            *record.lock().unwrap() = Some(event.draw().is_none());
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(seen.lock().unwrap().take(), Some(true));
    assert!(!fake.ctx().supports(Area::UI_DRAW));
    assert!(!fake.ctx().supports(Area::UI_TEXTURES));
}

#[test]
fn text_width_is_six_per_char_in_the_fake() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let draw = event.draw().unwrap();
            let mut widths = record.lock().unwrap();
            widths.extend([draw.text_width(""), draw.text_width("abc"), draw.text_width("h\u{e9}llo")]);
            widths.push(draw.text_width(&"x".repeat(257)));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(*seen.lock().unwrap(), vec![Some(0.0), Some(18.0), Some(30.0), None]);
}

#[test]
fn text_is_recorded_with_its_rect() {
    let fake = serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let draw = event.draw().unwrap();
            let mut results = record.lock().unwrap();
            results.push(draw.text("32", RECT, RED, 1.0, TextAlign::Right));
            results.push(draw.text(&"x".repeat(257), RECT, RED, 1.0, TextAlign::Left));
            results.push(draw.text("a", RECT, RED, f32::NAN, TextAlign::Center));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(*seen.lock().unwrap(), vec![Ok(()), Err(ApiError::Refused), Err(ApiError::Loader(invalid()))]);
    assert_eq!(fake.draws(), vec!["text \"32\" 10 20".to_string()]);
}

#[test]
fn text_without_the_text_area_is_not_available() {
    let fake = FakeGame::new();
    fake.offer::<ScreenRender>();
    fake.offer_area(Area::UI_DRAW);
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
            let draw = event.draw().unwrap();
            *record.lock().unwrap() = Some((draw.text("a", RECT, RED, 1.0, TextAlign::Left), draw.text_width("a")));
        })
        .unwrap();

    fake.raise_screen_render(ScreenId(1), ViewId(2), &screen());

    assert_eq!(seen.lock().unwrap().take(), Some((Err(ApiError::NotAvailable), None)));
    assert!(fake.draws().is_empty());
}
