use amml_api::testing::FakeGame;
use amml_api::{ApiError, Area, TextureId, TextureSource};

fn serving() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::UI_TEXTURES);
    fake.open_gate();
    fake
}

#[test]
fn load_and_upload_give_distinct_ids_and_are_not_ready_at_first() {
    let fake = serving();
    let textures = fake.ctx().textures();

    let file = textures.load(TextureSource::ModFile("textures/sort.png")).unwrap();
    let vanilla = textures.load(TextureSource::Vanilla("textures/ui/button")).unwrap();
    let upload = textures.upload_rgba(2, 1, &[255; 8]).unwrap();

    assert_ne!(file.id, vanilla.id);
    assert_ne!(vanilla.id, upload.id);
    assert!(!textures.ready(file) && !textures.ready(upload));
    assert_eq!(fake.textures().uploads(), vec![(upload, 2, 1)]);
    assert_eq!(
        fake.textures().loads(),
        vec!["mod_file textures/sort.png".to_string(), "vanilla textures/ui/button".to_string()]
    );

    fake.textures().set_ready(upload, true);
    assert!(textures.ready(upload));
}

#[test]
fn a_mod_file_outside_the_mod_folder_is_refused_and_a_bad_upload_is_invalid() {
    let fake = serving();
    let textures = fake.ctx().textures();

    assert_eq!(textures.load(TextureSource::ModFile("../secret.png")), Err(ApiError::Refused));
    assert!(matches!(textures.upload_rgba(2, 2, &[0; 4]), Err(ApiError::Loader(_))));
    assert!(matches!(textures.upload_rgba(0, 2, &[]), Err(ApiError::Loader(_))));
}

#[test]
fn unload_is_recorded_and_a_second_unload_is_stale() {
    let fake = serving();
    let textures = fake.ctx().textures();
    let texture = textures.load(TextureSource::Vanilla("textures/ui/a")).unwrap();

    textures.unload(texture).unwrap();

    assert_eq!(fake.textures().unloads(), vec![texture]);
    assert_eq!(textures.unload(texture), Err(ApiError::Stale));
    assert!(!textures.ready(texture));
}

#[test]
fn use_after_world_leave_is_stale() {
    let fake = serving();
    let textures = fake.ctx().textures();
    let texture = textures.load(TextureSource::Vanilla("textures/ui/a")).unwrap();
    fake.textures().set_ready(texture, true);

    fake.textures().leave_world();

    assert!(!textures.ready(texture));
    assert_eq!(textures.unload(texture), Err(ApiError::Stale));
    // A texture of the new world has a new generation.
    let next = textures.load(TextureSource::Vanilla("textures/ui/a")).unwrap();
    assert_ne!(next.generation, texture.generation);
}

#[test]
fn drop_off_thread_queues_the_free() {
    let fake = serving();
    let textures = fake.ctx().textures();
    let texture = textures.upload_rgba(1, 1, &[1, 2, 3, 4]).unwrap();

    // The fake is bound to this thread; a call from another thread only queues, as the loader does.
    let status = std::thread::spawn(move || textures.unload(texture)).join().unwrap();

    assert_eq!(status, Ok(()));
    assert!(fake.textures().unloads().is_empty());
    assert_eq!(fake.textures().pump(), 1);
    assert_eq!(fake.textures().unloads(), vec![texture]);
    assert_eq!(fake.ctx().textures().unload(texture), Err(ApiError::Stale));
}

#[test]
fn an_off_thread_unload_checks_the_id_at_once_as_the_loader_does() {
    let fake = serving();
    let textures = fake.ctx().textures();
    let first = textures.upload_rgba(1, 1, &[1, 2, 3, 4]).unwrap();
    let old = textures.upload_rgba(1, 1, &[1, 2, 3, 4]).unwrap();
    fake.textures().leave_world();

    let results = std::thread::spawn(move || [textures.unload(old), textures.unload(first)]).join().unwrap();
    assert_eq!(results, [Err(ApiError::Stale), Err(ApiError::Stale)]);

    let fresh = fake.ctx().textures().upload_rgba(1, 1, &[1, 2, 3, 4]).unwrap();
    let textures = fake.ctx().textures();
    let twice = std::thread::spawn(move || [textures.unload(fresh), textures.unload(fresh)]).join().unwrap();
    assert_eq!(twice, [Ok(()), Err(ApiError::Stale)]);
    assert_eq!(fake.textures().pump(), 1);
}

#[test]
fn without_the_textures_area_every_call_is_not_available() {
    let fake = FakeGame::new();
    fake.open_gate();
    let textures = fake.ctx().textures();

    assert_eq!(textures.load(TextureSource::Vanilla("a")), Err(ApiError::NotAvailable));
    assert_eq!(textures.upload_rgba(1, 1, &[0; 4]), Err(ApiError::NotAvailable));
    assert!(!textures.ready(TextureId { id: 1, generation: 0 }));
    assert_eq!(textures.unload(TextureId { id: 1, generation: 0 }), Err(ApiError::NotAvailable));
}

#[test]
fn a_v6_loader_has_no_texture_table() {
    let fake = FakeGame::with_v6_services();

    assert_eq!(fake.ctx().textures().load(TextureSource::Vanilla("a")), Err(ApiError::NotAvailable));
}

#[test]
fn textures_reloaded_gives_the_reason_and_a_device_reset_makes_the_old_ids_stale() {
    use amml_api::amml_abi::TexturesReloaded;
    use amml_api::{HookOrder, TextureReload, TexturesReloadedExt};
    use std::sync::{Arc, Mutex};

    let fake = serving();
    fake.offer::<TexturesReloaded>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<TexturesReloaded>(HookOrder::NORMAL, move |event, _| record.lock().unwrap().push(event.reason()))
        .unwrap();
    let textures = fake.ctx().textures();
    let upload = textures.upload_rgba(1, 1, &[255; 4]).unwrap();

    fake.raise_textures_reloaded(TextureReload::ResourcePack);
    assert_eq!(textures.unload(upload), Ok(()), "a pack change keeps the ids");
    let upload = textures.upload_rgba(1, 1, &[255; 4]).unwrap();
    fake.raise_textures_reloaded(TextureReload::DeviceReset);

    assert_eq!(*seen.lock().unwrap(), [TextureReload::ResourcePack, TextureReload::DeviceReset]);
    assert_eq!(textures.unload(upload), Err(ApiError::Stale), "a device reset makes the ids stale");
}
