use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml::{ModListener, SessionAdapter, TextureAdapter, TextureCalls, TextureReloadAdapter};
use amml_abi::{
    AmmlBand, AmmlEvent, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlTextureReload, TexturesReloaded,
};
use amml_data::{Availability, EntryKind, Resolved};
use amml_runtime::TextureRefusal;

use crate::support::{Harness, SessionGame};

// The arguments the stand-in `applyActiveResourcePacks` received, and the reasons the listener saw, in order.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

const APPLY: &str = "MinecraftGame.applyActiveResourcePacks";
const OWNER: usize = 1;

#[inline(never)]
extern "C" fn fake_apply(game: *mut c_void, manager: *mut c_void, changed: u64) {
    CALLS.lock().unwrap().push(format!(
        "apply {:#x} {:#x} {changed:#x}",
        black_box(game) as usize,
        black_box(manager) as usize
    ));
}

unsafe extern "C" fn on_reloaded(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `ui.textures_reloaded` with a `TexturesReloaded`.
    let event = unsafe { &*event.cast::<TexturesReloaded>() };
    CALLS.lock().unwrap().push(format!("reloaded {}", event.reason.0));
    AmmlStatus::OK
}

fn calls() -> Vec<String> {
    CALLS.lock().unwrap().clone()
}

// Calls the stand-in through its address, as the game calls the hooked function.
fn apply(game: usize, manager: usize, changed: u64) {
    let function = black_box(fake_apply as extern "C" fn(*mut c_void, *mut c_void, u64));
    function(black_box(game as *mut c_void), black_box(manager as *mut c_void), black_box(changed));
}

fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let mut all = SessionGame::entries(&[]);
    for name in [
        "ClientInstance.getTextureGroup",
        "TextureGroup.uploadTexture",
        "TextureGroup.getTexture",
        "TextureGroup.unloadTexture",
        "TextureGroup.textureGroupBase",
        "TextureGroup.baseOffset",
        "ResourceLocation.ctor",
    ] {
        all.push((name, field(0)));
    }
    let apply = Resolved {
        kind: EntryKind::Function,
        value: fake_apply as *const () as usize as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    all.push((APPLY, apply));
    all.into_iter().filter(|(id, _)| !without.contains(id)).collect()
}

fn texture_calls() -> TextureCalls {
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

// The session and texture adapters over the stand-ins, then the reload adapter; gives the harness and its rows.
fn install(without: &[&str]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(&entries(without));
    SessionAdapter::install(harness.state);
    TextureAdapter::install_with(harness.state, texture_calls());
    let rows = TextureReloadAdapter::install(harness.state);
    (harness, rows)
}

// `install` with every entry, a listener, and this thread as the game thread.
fn start() -> Harness {
    let (harness, rows) = install(&[]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].shape, Some("default"));
    let listener = ModListener { owner: 0, callback: on_reloaded, user: 0 };
    assert!(harness.state.events().listen(TexturesReloaded::ID, 1, AmmlBand::NORMAL, listener).is_ok());
    SessionGame::frame();
    CALLS.lock().unwrap().clear();
    harness
}

#[test]
fn a_pack_change_passes_every_register_on_binds_the_textures_again_and_raises_resource_pack() {
    let _harness = start();
    let book = TextureAdapter::get().unwrap().book();
    let key = book.upload(OWNER, 1, 1, &[0; 4]).unwrap();
    assert_eq!(book.take_work(0).jobs.len(), 1);
    assert!(book.take_work(0).jobs.is_empty());

    apply(0x6A_0000, 0x7B_0000, 0xDEAD_BEEF_0000_0001);

    assert_eq!(calls(), ["apply 0x6a0000 0x7b0000 0xdeadbeef00000001", "reloaded 0"]);
    assert_eq!(book.take_work(0).jobs.len(), 1, "the upload is bound again");
    assert!(book.is_current(key), "the id stays valid");
}

#[test]
fn a_change_off_the_game_thread_is_raised_after_the_next_frame_once() {
    let _harness = start();

    std::thread::scope(|scope| {
        scope.spawn(|| {
            apply(1, 2, 1);
            apply(1, 2, 1);
        });
    });
    assert_eq!(calls(), ["apply 0x1 0x2 0x1", "apply 0x1 0x2 0x1"]);

    SessionGame::frame();

    assert_eq!(calls(), ["apply 0x1 0x2 0x1", "apply 0x1 0x2 0x1", "reloaded 0"]);
}

#[test]
fn a_device_reset_frees_the_textures_and_raises_device_reset() {
    let _harness = start();
    let book = TextureAdapter::get().unwrap().book();
    let key = book.upload(OWNER, 1, 1, &[0; 4]).unwrap();

    TextureAdapter::get().unwrap().device_reset();

    assert_eq!(calls(), [format!("reloaded {}", AmmlTextureReload::DEVICE_RESET.0)]);
    assert_eq!(book.drawable(key), Err(TextureRefusal::Stale));
}

#[test]
fn a_device_reset_on_the_render_thread_is_raised_on_the_game_thread_after_the_next_frame() {
    let _harness = start();

    std::thread::scope(|scope| {
        scope.spawn(|| TextureAdapter::get().unwrap().device_reset());
    });
    assert!(calls().is_empty());

    SessionGame::frame();

    assert_eq!(calls(), ["reloaded 1"]);
}

#[test]
fn without_the_pack_entry_the_event_is_not_offered_and_the_function_is_not_hooked() {
    let (harness, rows) = install(&[APPLY]);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].shape, None);
    assert_eq!(rows[0].missing, vec![APPLY]);
    let listener = ModListener { owner: 0, callback: on_reloaded, user: 0 };
    assert!(harness.state.events().listen(TexturesReloaded::ID, 1, AmmlBand::NORMAL, listener).is_err());
}
