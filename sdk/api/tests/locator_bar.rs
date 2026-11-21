use amml_abi::LocatorBarIconDraw;
use amml_api::testing::FakeGame;
use amml_api::{ActorUniqueId, Area, HookOrder, LocatorBarIconDrawExt, Rect, Rgba, TextureId, TextureSource};

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<LocatorBarIconDraw>();
    fake.offer_area(Area::UI_TEXTURES);
    fake.open_gate();
    fake
}

// A texture the mod uploaded and the game gave: ready to draw.
fn ready_texture(fake: &FakeGame) -> TextureId {
    let texture = fake.ctx().textures().upload_rgba(2, 2, &[255; 16]).unwrap();
    fake.textures().set_ready(texture, true);
    texture
}

#[test]
fn replace_texture_is_read_back() {
    let fake = serving();
    let texture = ready_texture(&fake);
    let _listener = fake
        .ctx()
        .events()
        .listen::<LocatorBarIconDraw>(HookOrder::NORMAL, move |event, _| {
            event.replace_texture(texture);
            event.set_rect(Rect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 });
            event.set_color(Rgba { r: 0x11, g: 0x22, b: 0x33, a: 0x44 });
        })
        .unwrap();

    let edit = fake.raise_locator_icon(Some(ActorUniqueId(7)));

    assert_eq!(edit.texture, Some(texture));
    assert_eq!(edit.rect, Rect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 });
    assert_eq!(edit.color, Rgba { r: 0x11, g: 0x22, b: 0x33, a: 0x44 });
}

#[test]
fn stale_texture_drops_all_edits() {
    let fake = serving();
    let stale = ready_texture(&fake);
    fake.textures().leave_world();
    // Not known to the book, and a texture the game did not give yet.
    let unknown = TextureId { id: 4242, generation: 1 };
    let unready = fake.ctx().textures().load(TextureSource::Vanilla("textures/ui/b")).unwrap();
    for texture in [stale, unknown, unready] {
        let listener = fake
            .ctx()
            .events()
            .listen::<LocatorBarIconDraw>(HookOrder::NORMAL, move |event, _| {
                event.replace_texture(texture);
                event.set_rect(Rect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 });
                event.set_color(Rgba { r: 1, g: 2, b: 3, a: 4 });
            })
            .unwrap();

        let edit = fake.raise_locator_icon(Some(ActorUniqueId(7)));

        // The game draws its own icon: the rect and the colour of the call, and no texture.
        assert_eq!(edit.texture, None, "{texture:?}");
        assert_eq!(edit.rect, Rect { x: 10.0, y: 20.0, width: 8.0, height: 9.0 }, "{texture:?}");
        assert_eq!(edit.color, Rgba { r: 255, g: 255, b: 255, a: 255 }, "{texture:?}");
        drop(listener);
    }
}

#[test]
fn a_rect_and_colour_edit_without_a_texture_applies() {
    let fake = serving();
    let _listener = fake
        .ctx()
        .events()
        .listen::<LocatorBarIconDraw>(HookOrder::NORMAL, |event, _| {
            event.set_color(Rgba { r: 9, g: 8, b: 7, a: 6 });
        })
        .unwrap();

    let edit = fake.raise_locator_icon(Some(ActorUniqueId(7)));

    assert_eq!(edit.texture, None);
    assert_eq!(edit.color, Rgba { r: 9, g: 8, b: 7, a: 6 });
    assert_eq!(edit.rect, Rect { x: 10.0, y: 20.0, width: 8.0, height: 9.0 });
}

#[test]
fn waypoint_without_actor_has_none() {
    let fake = serving();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<LocatorBarIconDraw>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.actor(), event.rect(), event.color()));
        })
        .unwrap();

    fake.raise_locator_icon(None);
    fake.raise_locator_icon(Some(ActorUniqueId(-5)));

    let white = Rgba { r: 255, g: 255, b: 255, a: 255 };
    let rect = Rect { x: 10.0, y: 20.0, width: 8.0, height: 9.0 };
    assert_eq!(*seen.lock().unwrap(), [(None, rect, white), (Some(ActorUniqueId(-5)), rect, white)]);
}
