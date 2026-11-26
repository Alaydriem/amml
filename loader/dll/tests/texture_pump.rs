use amml::{TextureAdapter, TextureCalls, TexturePtrLayout, TexturePump};
use amml_abi::{AmmlApi, AmmlServices, AmmlStatus, AmmlStr, AmmlTextureId, AmmlUiTable};
use amml_runtime::{ModRegistry, TextureSourceKind};

use crate::support::{EventsMod, Harness, SessionGame};

const MODULE: &str = "@events/events";

fn start() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(amml::ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    harness
}

fn calls() -> TextureCalls {
    TextureCalls {
        resource_location_ctor: 1,
        image_buffer_ctor: 1,
        image_buffer_words: 6,
        buffer_deleter: 0,
        buffer_data: 8,
        buffer_size: 16,
        upload: 1,
        group_get: 1,
        unload: 1,
        group_base: 0x18,
        get_texture_group_slot: 1,
        context_get_texture_slot: 1,
    }
}

#[test]
fn a_request_that_lands_while_the_pump_settles_keeps_it_pending() {
    TexturePump::settle(|| (false, false));
    assert!(!TexturePump::pending());

    // The request comes after the pump took its work and before it read the book to settle.
    TexturePump::settle(|| {
        TexturePump::request();
        (false, false)
    });

    assert!(TexturePump::pending());
}

#[test]
fn waiting_textures_are_polled_later_not_at_every_draw() {
    TexturePump::settle(|| (false, true));

    assert!(!TexturePump::pending());
    TexturePump::settle(|| (true, false));
    assert!(TexturePump::pending());
}

#[test]
fn an_off_thread_unload_is_queued_and_the_pump_frees_it() {
    let harness = start();
    TextureAdapter::install_with(harness.state, calls());
    SessionGame::frame();
    let adapter = TextureAdapter::get().unwrap();
    let key = adapter.book().load(0, TextureSourceKind::Vanilla, "textures/ui/a", "").unwrap();
    let held = Box::into_raw(Box::new(TexturePtrLayout::empty())) as usize;
    assert!(adapter.book().bound(key, held));
    let id = AmmlTextureId { id: key.id, generation: key.generation, _reserved: 0 };
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table: &AmmlUiTable = unsafe { &*AmmlServices::ui(services).unwrap() };
    let unload = table.texture_unload.unwrap();
    let module = AmmlStr::from_str(MODULE);

    // SAFETY: the table is live. The unload is checked at once and only queued.
    let status = std::thread::scope(|scope| {
        scope.spawn(move || unsafe { unload(AmmlStr::from_str(MODULE), id) }).join().unwrap()
    });

    assert_eq!(status, AmmlStatus::OK);
    assert!(TexturePump::pending());
    assert!(!adapter.book().is_current(key));
    // SAFETY: the table is live. A second unload and an old generation are stale.
    assert_eq!(unsafe { unload(module, id) }, AmmlStatus::STALE);
    let old = AmmlTextureId { generation: id.generation + 1, ..id };
    // SAFETY: as above.
    assert_eq!(unsafe { unload(module, old) }, AmmlStatus::STALE);

    adapter.pump(0);

    assert!(adapter.book().take_work(1).releases.is_empty());
    assert!(!TexturePump::pending());
}

#[test]
fn the_pump_of_the_locator_bar_runs_no_free_and_the_next_screen_pump_does() {
    let harness = start();
    TextureAdapter::install_with(harness.state, calls());
    SessionGame::frame();
    let adapter = TextureAdapter::get().unwrap();
    let key = adapter.book().load(0, TextureSourceKind::Vanilla, "textures/ui/a", "").unwrap();
    let held = Box::into_raw(Box::new(TexturePtrLayout::empty())) as usize;
    assert!(adapter.book().bound(key, held));
    adapter.book().unload(0, key).unwrap();
    TexturePump::request();
    assert!(TexturePump::pending());

    TexturePump::run_uploads(0);

    // The free is still queued, and the pump stays pending for the pump that runs it.
    assert!(TexturePump::pending());
    assert!(adapter.book().has_due_work());

    TexturePump::run(0);

    assert!(!adapter.book().has_due_work());
    assert!(!TexturePump::pending());
}
