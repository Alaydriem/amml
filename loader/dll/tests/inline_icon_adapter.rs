use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{InlineIconAdapter, SessionAdapter, TextCalls, TextureAdapter, TextureCalls, TexturePump, UiFlags};
use amml_abi::{AmmlApi, AmmlImageView, AmmlInlineIcon, AmmlServices, AmmlStatus, AmmlStr, AmmlUiTable};
use amml_data::{EntryKind, Resolved};
use amml_runtime::{ModRegistry, TextureJob};

use crate::support::{EventsMod, Harness, SessionGame};

const MODULE: &str = "@events/events";

type WidthFn = extern "C" fn(usize, i32, u8) -> f32;

// How the stand-in width function was called: the font, the character and the flag.
static WIDTH_CALLS: Mutex<Vec<(usize, i32, u8)>> = Mutex::new(Vec::new());
static DESTROYED: AtomicUsize = AtomicUsize::new(0);
// The default font the stand-in client hands out.
static DEFAULT_FONT: AtomicUsize = AtomicUsize::new(0);
// The vtable of a `BitmapFont`, and of any other font.
static BITMAP_VTABLE: [usize; 4] = [0; 4];
static OTHER_VTABLE: [usize; 4] = [0; 4];

#[inline(never)]
extern "C" fn char_width_stand_in(font: usize, character: i32, force: u8) -> f32 {
    black_box(character);
    WIDTH_CALLS.lock().unwrap().push((font, character, force));
    5.0
}

unsafe extern "C" fn fake_font_handle(_: usize, out: *mut [u64; 10]) -> usize {
    let mut handle = [0u64; 10];
    handle[6] = DEFAULT_FONT.load(Ordering::SeqCst) as u64;
    // SAFETY: the adapter passes a buffer of the size of a `FontHandle`.
    unsafe { out.write(handle) };
    out as usize
}

unsafe extern "C" fn fake_font_destroy(_: *mut [u64; 10]) {
    DESTROYED.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn no_player(_: *mut c_void) -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn no_join(_: *mut c_void, _: u64, _: u64, _: u64, _: u64, _: u64, _: u64, _: u64) {}

// The stand-in client: its vtable has `getLocalPlayer` at slot 0 and `getFontHandle` at slot 5.
struct FakeClient {
    client: Box<[usize; 4]>,
    _vtable: Box<[usize; 8]>,
}

impl FakeClient {
    fn new() -> Self {
        let mut vtable = Box::new([0usize; 8]);
        vtable[0] = no_player as *const () as usize;
        vtable[1] = no_join as *const () as usize;
        vtable[5] = fake_font_handle as *const () as usize;
        let mut client = Box::new([0usize; 4]);
        client[0] = vtable.as_ptr().addr();
        Self { client, _vtable: vtable }
    }

    // Makes a font with `vtable` the default font of the client.
    fn use_font(&self, vtable: &'static [usize; 4]) {
        // Leaked: the game keeps its fonts for the process.
        let font = Box::leak(Box::new([vtable.as_ptr().addr(), 0usize]));
        DEFAULT_FONT.store(font.as_ptr().addr(), Ordering::SeqCst);
    }
}

fn function(address: usize) -> Resolved {
    Resolved { kind: EntryKind::Function, value: address as u64 - Harness::BASE, recipes_agreeing: 2 }
}

fn global(address: usize) -> Resolved {
    Resolved { kind: EntryKind::Global, value: address as u64 - Harness::BASE, recipes_agreeing: 2 }
}

fn field(value: usize) -> Resolved {
    Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 }
}

fn slot(value: u64) -> Resolved {
    Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 }
}

// The entries of the `ui.inline_icons` catalog row. The texture entries only need to be there: the texture adapter of
// these tests is installed over stand-in calls.
fn entries() -> Vec<(&'static str, Resolved)> {
    let mut extra = vec![
        ("BitmapFont._getCharWidth", function(char_width_stand_in as *const () as usize)),
        ("BitmapFont.vtable", global(std::ptr::from_ref(&BITMAP_VTABLE) as usize)),
        ("MinecraftUIRenderContext.drawText", slot(5)),
        ("MinecraftUIRenderContext.flushText", slot(6)),
        ("ClientInstance.getFontHandle", slot(5)),
        ("Font.getLineLength", slot(6)),
        ("FontHandle.dtor", function(fake_font_destroy as *const () as usize)),
        ("FontHandle.size", field(0x50)),
        ("FontHandle.mDefaultFont", field(0x30)),
        ("FontRepository.mLoadedFonts", field(0x40)),
    ];
    for name in [
        "ClientInstance.getTextureGroup",
        "TextureGroup.uploadTexture",
        "TextureGroup.getTexture",
        "TextureGroup.unloadTexture",
        "TextureGroup.textureGroupBase",
        "TextureGroup.baseOffset",
        "ResourceLocation.ctor",
        "ResourceLocation.size",
        "ImageBuffer.ctorFromDescription",
        "ImageBuffer.size",
        "ImageBuffer.mData",
        "ImageBuffer.mSize",
        "ImageBuffer.mDeleter",
        "MinecraftUIRenderContext.getTexture",
    ] {
        extra.push((name, field(0)));
    }
    let mut all = SessionGame::entries(&[]);
    all.extend(extra);
    all
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

fn text_calls() -> TextCalls {
    TextCalls {
        draw_slot: 5,
        flush_slot: 6,
        get_font_handle_slot: 5,
        line_length_slot: 6,
        font_dtor: fake_font_destroy as *const () as usize,
        font_handle_size: 0x50,
        default_font: 0x30,
        loaded_fonts: 0x40,
    }
}

// A harness with the session adapter and one loaded mod.
fn start_without_textures() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    SessionAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(amml::ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    harness
}

// A harness with the texture adapter, the session adapter, one loaded mod, the gate open on this thread and the stand-in
// client (with a bitmap default font) as the primary client.
fn start(client: &mut FakeClient) -> Harness {
    let harness = start_without_textures();
    TextureAdapter::install_with(harness.state, texture_calls());
    client.use_font(&BITMAP_VTABLE);
    SessionGame::frame();
    SessionGame::update_raw(client.client.as_ptr().cast_mut().cast(), 1, 0);
    harness
}

// The same, with the real install: the width hook and the area.
fn start_installed(client: &mut FakeClient) -> Harness {
    let harness = start(client);
    InlineIconAdapter::install(harness.state);
    harness
}

fn table(harness: &Harness) -> AmmlUiTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlUiTable::read(AmmlServices::ui(services).unwrap()) }.unwrap()
}

// A view of `width` x `height` RGBA pixels, all `value`.
fn view(width: u32, height: u32, value: u8, pixels: &mut Vec<u8>) -> AmmlImageView {
    *pixels = vec![value; (width * height * 4) as usize];
    AmmlImageView { width, height, format: 4, _reserved: 0, pixels: pixels.as_ptr(), len: pixels.len() }
}

fn register(table: &AmmlUiTable, image: &AmmlImageView, advance: f32) -> Result<AmmlInlineIcon, AmmlStatus> {
    let mut out = AmmlInlineIcon::default();
    // SAFETY: the table is live; the module, the image and the output live across the call.
    let status = unsafe { (table.icon_register.unwrap())(AmmlStr::from_str(MODULE), image, advance, &mut out) };
    if status == AmmlStatus::OK { Ok(out) } else { Err(status) }
}

fn update(table: &AmmlUiTable, icon: AmmlInlineIcon, image: &AmmlImageView) -> AmmlStatus {
    // SAFETY: as for `register`.
    unsafe { (table.icon_update.unwrap())(AmmlStr::from_str(MODULE), icon, image) }
}

fn release(table: &AmmlUiTable, icon: AmmlInlineIcon) -> AmmlStatus {
    // SAFETY: the table is live and the module lives across the call.
    unsafe { (table.icon_release.unwrap())(AmmlStr::from_str(MODULE), icon) }
}

fn active(table: &AmmlUiTable) -> u8 {
    // SAFETY: the table function takes no arguments.
    unsafe { (table.chat_icons_active.unwrap())() }
}

fn width(character: i32) -> f32 {
    let function: WidthFn = black_box(char_width_stand_in);
    function(0xF0, black_box(character), 1)
}

#[test]
fn register_without_bitmap_font_is_not_available() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    client.use_font(&OTHER_VTABLE);
    let table = table(&harness);
    let mut pixels = Vec::new();

    assert_eq!(register(&table, &view(8, 8, 1, &mut pixels), 9.0), Err(AmmlStatus::NOT_AVAILABLE));
    assert_eq!(active(&table), 0);
    assert!(!InlineIconAdapter::get().unwrap().book().has_icons());
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
}

// One pump of the page as the game would run it: `sync_page`, every upload answered (uploaded, bound, ready), then the next
// `sync_page`, which opens the gate. Gives the jobs of the pump.
fn pump_page() -> Vec<TextureJob> {
    static HELD: AtomicUsize = AtomicUsize::new(0x1000);
    let adapter = InlineIconAdapter::get().unwrap();
    let book = TextureAdapter::get().unwrap().book();
    adapter.sync_page();
    let work = book.take_work(0);
    for job in &work.jobs {
        if let TextureJob::Upload { key, .. } = job {
            book.uploaded(*key);
            assert!(book.bound(*key, HELD.fetch_add(0x10, Ordering::SeqCst)));
            book.mark_ready(*key);
        }
    }
    adapter.sync_page();
    work.jobs
}

#[test]
fn an_icon_is_registered_measured_and_released() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    assert!(harness.state.areas().is_available("ui.inline_icons", 1));
    assert_eq!(active(&table), 0);

    // No icon: one relaxed load, then the original with the registers it received.
    WIDTH_CALLS.lock().unwrap().clear();
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
    assert_eq!(width(0xE110), 5.0);
    assert_eq!(*WIDTH_CALLS.lock().unwrap(), [(0xF0, 0xE110, 1)]);

    let icon = register(&table, &view(8, 8, 1, &mut pixels), 9.0).unwrap();
    assert_eq!(icon.glyph, 0xE110);
    assert!(TexturePump::pending());
    pump_page();

    assert_eq!(active(&table), 1);
    assert_eq!(UiFlags::load() & UiFlags::ICONS, UiFlags::ICONS);
    WIDTH_CALLS.lock().unwrap().clear();
    assert_eq!(width(0xE110), 9.0);
    assert!(WIDTH_CALLS.lock().unwrap().is_empty(), "an icon is answered without the original");
    assert_eq!(width(0x41), 5.0);
    assert_eq!(width(0xE111), 5.0);
    assert_eq!(width(-1), 5.0);
    assert_eq!(WIDTH_CALLS.lock().unwrap().len(), 3);

    assert_eq!(release(&table, icon), AmmlStatus::OK);

    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
    assert_eq!(width(0xE110), 5.0);
    assert_eq!(release(&table, icon), AmmlStatus::STALE);
}

#[test]
fn no_icon_is_drawn_before_the_page_is_uploaded_and_ready() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let adapter = InlineIconAdapter::get().unwrap();
    let book = TextureAdapter::get().unwrap().book();
    let mut pixels = Vec::new();
    let icon = register(&table, &view(8, 8, 1, &mut pixels), 9.0).unwrap();

    // Registered, not uploaded: the icon is not measured and a listener is told that icons are off.
    WIDTH_CALLS.lock().unwrap().clear();
    assert_eq!((active(&table), UiFlags::load() & UiFlags::ICONS), (0, 0));
    assert_eq!(width(icon.glyph as i32), 5.0);
    assert_eq!(WIDTH_CALLS.lock().unwrap().len(), 1);

    // Uploaded and bound, the data of the game is not there yet: still off.
    adapter.sync_page();
    assert!(adapter.needs_sync(), "the pump runs once more to look at the upload");
    let jobs = book.take_work(0).jobs;
    let [TextureJob::Upload { key, .. }] = &jobs[..] else { panic!("{jobs:?}") };
    book.uploaded(*key);
    assert!(book.bound(*key, 0x1000));
    adapter.sync_page();
    assert!(!adapter.needs_sync());
    assert_eq!((active(&table), UiFlags::load() & UiFlags::ICONS), (0, 0));
    assert_eq!(width(icon.glyph as i32), 5.0);

    // The data is there: the gate opens at the next look, and not before.
    book.mark_ready(*key);
    assert_eq!(active(&table), 0);
    adapter.sync_page();
    assert_eq!((active(&table), UiFlags::load() & UiFlags::ICONS), (1, UiFlags::ICONS));
    assert_eq!(width(icon.glyph as i32), 9.0);
}

#[test]
fn a_page_freed_on_the_game_side_closes_the_gate_and_is_uploaded_again() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    let icon = register(&table, &view(8, 8, 1, &mut pixels), 9.0).unwrap();
    pump_page();
    assert_eq!(width(icon.glyph as i32), 9.0);

    // The texture book frees every texture (a render device reset); the icon book still has the icon.
    TextureAdapter::get().unwrap().device_reset();
    InlineIconAdapter::get().unwrap().sync_page();

    assert_eq!((active(&table), UiFlags::load() & UiFlags::ICONS), (0, 0));
    assert_eq!(width(icon.glyph as i32), 5.0);
    let jobs = TextureAdapter::get().unwrap().book().take_work(1).jobs;
    assert!(matches!(&jobs[..], [TextureJob::Upload { name, .. }] if name == "font/glyph_E1"));
}

#[test]
fn the_font_check_destroys_its_font_handle() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    let before = DESTROYED.load(Ordering::SeqCst);

    register(&table, &view(2, 2, 1, &mut pixels), 3.0).unwrap();

    assert_eq!(DESTROYED.load(Ordering::SeqCst), before + 1);
}

#[test]
fn register_off_the_game_thread_is_refused_and_a_free_off_the_game_thread_only_queues() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    let image = view(8, 8, 1, &mut pixels);
    let icon = register(&table, &image, 9.0).unwrap();
    TexturePump::settle(|| (false, false));
    assert!(!TexturePump::pending());
    let address = std::ptr::from_ref(&image).addr();

    let refused = std::thread::scope(|scope| {
        scope
            .spawn(move || {
                // SAFETY: the table is live; the image outlives the scope.
                let image = std::ptr::with_exposed_provenance::<AmmlImageView>(address);
                let mut out = AmmlInlineIcon::default();
                unsafe { (table.icon_register.unwrap())(AmmlStr::from_str(MODULE), image, 9.0, &mut out) }
            })
            .join()
            .unwrap()
    });
    assert_eq!(refused, AmmlStatus::REFUSED);

    // The icon of the game thread is freed from another thread: checked at once, the page is uploaded at the next draw.
    let freed = std::thread::scope(|scope| scope.spawn(move || release(&table, icon)).join().unwrap());
    assert_eq!(freed, AmmlStatus::OK);
    assert!(TexturePump::pending());
    assert!(!InlineIconAdapter::get().unwrap().book().has_icons());
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
}

#[test]
fn use_after_a_world_leave_is_stale() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    let image = view(8, 8, 1, &mut pixels);
    let icon = register(&table, &image, 9.0).unwrap();

    SessionGame::exit();
    InlineIconAdapter::get().unwrap().world_left();

    assert_eq!(update(&table, icon, &image), AmmlStatus::STALE);
    assert_eq!(release(&table, icon), AmmlStatus::STALE);
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
    assert_eq!(width(icon.glyph as i32), 5.0);
    // An icon of the new generation works.
    let fresh = register(&table, &image, 9.0).unwrap();
    assert_ne!(fresh.generation, icon.generation);
    assert_eq!(update(&table, fresh, &image), AmmlStatus::OK);
}

#[test]
fn bad_images_and_advances_are_refused_or_invalid() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    let mut other = Vec::new();

    assert_eq!(register(&table, &view(8, 8, 1, &mut pixels), 17.0), Err(AmmlStatus::REFUSED));
    assert_eq!(register(&table, &view(8, 8, 1, &mut pixels), 0.0), Err(AmmlStatus::REFUSED));
    assert_eq!(register(&table, &view(17, 17, 1, &mut pixels), 9.0), Err(AmmlStatus::REFUSED));
    let mut wrong_format = view(8, 8, 1, &mut other);
    wrong_format.format = 3;
    assert_eq!(register(&table, &wrong_format, 9.0), Err(AmmlStatus::INVALID_ARGUMENT));
    let mut short = view(8, 8, 1, &mut other);
    short.len -= 4;
    assert_eq!(register(&table, &short, 9.0), Err(AmmlStatus::INVALID_ARGUMENT));
    let mut empty = view(0, 0, 1, &mut other);
    empty.pixels = std::ptr::null();
    assert_eq!(register(&table, &empty, 9.0), Err(AmmlStatus::INVALID_ARGUMENT));
    let mut out = AmmlInlineIcon::default();
    // SAFETY: the table is live; a null image and a null output are the cases under test.
    let status = unsafe {
        let call = table.icon_register.unwrap();
        [
            call(AmmlStr::from_str(MODULE), std::ptr::null(), 9.0, &mut out),
            call(AmmlStr::from_str(MODULE), &wrong_format, 9.0, std::ptr::null_mut()),
        ]
    };
    assert_eq!(status, [AmmlStatus::INVALID_ARGUMENT; 2]);
    assert!(!InlineIconAdapter::get().unwrap().book().has_icons());
}

#[test]
fn a_disabled_mod_loses_its_icons() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let mut pixels = Vec::new();
    register(&table, &view(8, 8, 1, &mut pixels), 9.0).unwrap();
    pump_page();
    assert_eq!(UiFlags::load() & UiFlags::ICONS, UiFlags::ICONS);
    // The only loaded mod is the first one.
    let owner = 0;
    let _ = &harness;

    InlineIconAdapter::get().unwrap().disable_owner(owner);

    assert!(!InlineIconAdapter::get().unwrap().book().has_icons());
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
}

#[test]
fn the_page_is_uploaded_under_the_font_page_name_and_every_change_replaces_it_in_place() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let adapter = InlineIconAdapter::get().unwrap();
    let book = TextureAdapter::get().unwrap().book();
    let mut pixels = Vec::new();

    // No icon: nothing is uploaded and the vanilla row is not sent alone.
    adapter.sync_page();
    assert!(book.take_work(0).jobs.is_empty());

    let icon = register(&table, &view(2, 2, 77, &mut pixels), 3.0).unwrap();
    let jobs = pump_page();
    let [TextureJob::Upload { name, width, height, pixels: page, .. }] = &jobs[..] else { panic!("{jobs:?}") };
    assert_eq!((name.as_str(), *width, *height), ("font/glyph_E1", 256, 256));
    // Row 0 is the vanilla row the loader ships; the icon sits at the top left of cell 16 (row 1, column 0).
    assert!(page[..amml_runtime::GlyphPage::ROW0_BYTES].iter().any(|byte| *byte != 0));
    let at = 16 * 256 * 4;
    assert_eq!(&page[at..at + 8], &[77; 8]);
    assert!(book.take_work(1).jobs.is_empty());

    // A change uploads the page again under its name, in place: no unload of the name, not now and not the second time.
    for (round, value) in [(2u64, 99u8), (3, 55)] {
        assert_eq!(update(&table, icon, &view(2, 2, value, &mut pixels)), AmmlStatus::OK);
        adapter.sync_page();
        let work = book.take_work(round);
        assert!(work.releases.iter().all(|release| release.unload_name.is_none()), "{:?}", work.releases);
        let [TextureJob::Upload { key, name, pixels: page, .. }] = &work.jobs[..] else { panic!("{:?}", work.jobs) };
        assert_eq!(name, "font/glyph_E1");
        assert_eq!(&page[at..at + 4], &[value; 4]);
        book.uploaded(*key);
        assert!(book.bound(*key, 0x4000 + round as usize));
        book.mark_ready(*key);
        adapter.sync_page();
        // The gate stays open through a change: the icons that are already there do not flicker.
        assert_eq!(UiFlags::load() & UiFlags::ICONS, UiFlags::ICONS);
    }
}

#[test]
fn the_page_stays_when_the_last_icon_is_freed() {
    let mut client = FakeClient::new();
    let harness = start_installed(&mut client);
    let table = table(&harness);
    let adapter = InlineIconAdapter::get().unwrap();
    let book = TextureAdapter::get().unwrap().book();
    let mut pixels = Vec::new();
    let icon = register(&table, &view(2, 2, 77, &mut pixels), 3.0).unwrap();
    pump_page();

    assert_eq!(release(&table, icon), AmmlStatus::OK);
    adapter.sync_page();

    // The blank cell is sent in place; the texture is not unloaded.
    let work = book.take_work(1);
    assert!(work.releases.iter().all(|release| release.unload_name.is_none()));
    let [TextureJob::Upload { name, pixels: page, .. }] = &work.jobs[..] else { panic!("{:?}", work.jobs) };
    assert_eq!(name, "font/glyph_E1");
    assert_eq!(&page[16 * 256 * 4..16 * 256 * 4 + 4], &[0; 4]);
    assert_eq!(UiFlags::load() & UiFlags::ICONS, 0);
}

#[test]
fn the_area_is_not_offered_without_the_texture_adapter() {
    let harness = start_without_textures();

    let rows = InlineIconAdapter::install(harness.state);

    assert!(InlineIconAdapter::get().is_none());
    assert!(rows.iter().all(|row| row.shape.is_none()), "{rows:?}");
    assert!(!harness.state.areas().is_available("ui.inline_icons", 1));
    // The calls answer without the adapter.
    let table = table(&harness);
    let mut pixels = Vec::new();
    assert_eq!(register(&table, &view(2, 2, 1, &mut pixels), 3.0), Err(AmmlStatus::NOT_AVAILABLE));
    assert_eq!(active(&table), 0);
}

#[test]
fn an_adapter_over_calls_without_the_hook_serves_the_area() {
    let mut client = FakeClient::new();
    let harness = start(&mut client);

    assert!(InlineIconAdapter::install_with(harness.state, text_calls(), std::ptr::from_ref(&BITMAP_VTABLE) as usize));
    assert!(harness.state.areas().is_available("ui.inline_icons", 1));
    assert!(!InlineIconAdapter::install_with(harness.state, text_calls(), 0));
}
