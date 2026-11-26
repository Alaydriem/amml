use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{
    DrawCalls, InventoryAdapter, ItemDrawAdapter, ModLoader, SessionAdapter, TexturePump, UiDrawAdapter, UiScope,
};
use amml_abi::{
    AmmlApi, AmmlContainerEntry, AmmlInventoryTable, AmmlItemDraw, AmmlItemDrawFlags, AmmlPoint, AmmlServices,
    AmmlStatus, AmmlStr, AmmlUiTable,
};
use amml_data::{EntryKind, Resolved};
use amml_msvc::MsvcString;
use amml_runtime::{ItemDrawBook, ModRegistry, UiScopeKind};

use crate::support::{ClientGame, EventsMod, FakeCompound, FakeValue, GAME_CLIENT_A, Harness, TestHeap};

const MODULE: &str = "@events/events";
const OTHER: &str = "@events/events-provider";
// The offset of `Item::mFullName` in the fake items.
const FULL_NAME: usize = 0x20;

static STACK_VTABLE: [extern "C" fn(*mut c_void, u32) -> *mut c_void; 1] = [slot_zero];
static SLOT_ZERO_CALLS: AtomicUsize = AtomicUsize::new(0);
static STONE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static BUNDLE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static UNKNOWN_TAG: AtomicUsize = AtomicUsize::new(0);
static DTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
// The entry tags that make a bundle, each with the user data its stack gets (0 for none).
static BUNDLES: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

// Builds a stack of count 5 in the buffer, as `ItemStack::fromTag` does, except for the tag of an item the game does
// not know, which gets the empty stack. A tag registered in `BUNDLES` makes a bundle with its user data.
extern "C" fn from_tag(buffer: *mut c_void, tag: *const c_void) -> *mut c_void {
    let known = tag.addr() != UNKNOWN_TAG.load(Ordering::SeqCst);
    let bundle = BUNDLES.lock().unwrap().iter().find(|(entry, _)| *entry == tag.addr()).map(|(_, data)| *data);
    let counter =
        if bundle.is_some() { BUNDLE_COUNTER.load(Ordering::SeqCst) } else { STONE_COUNTER.load(Ordering::SeqCst) };
    // SAFETY: the buffer is the 0x98 bytes the loader allocated.
    unsafe {
        buffer.cast::<usize>().write(STACK_VTABLE.as_ptr().addr());
        buffer.cast::<usize>().add(1).write(if known { counter } else { 0 });
        buffer.cast::<usize>().add(2).write(bundle.unwrap_or(0));
        buffer.cast::<u8>().add(0x22).write(5);
    }
    buffer
}

// The resolved `ItemStackBase.dtor` entry.
extern "C" fn dtor(_this: *mut c_void) {
    DTOR_CALLS.fetch_add(1, Ordering::SeqCst);
}

// Vtable slot 0 of the fake stacks: no entry proves it, so a destroy must never reach it.
extern "C" fn slot_zero(this: *mut c_void, _flags: u32) -> *mut c_void {
    SLOT_ZERO_CALLS.fetch_add(1, Ordering::SeqCst);
    this
}

extern "C" fn damage(_: *const c_void) -> i16 {
    0
}

// An item whose `mFullName` is `name`, and the counter that points to it.
fn fake_item(name: &str) -> (Box<[u64; 16]>, Box<[usize; 1]>) {
    let mut item = Box::new([0u64; 16]);
    // SAFETY: the test heap is the Rust allocator; the string is never freed.
    let text = unsafe { MsvcString::new_in(&TestHeap::heap(), name) }.unwrap();
    // SAFETY: the item block holds a HashedString of the 30h layout at FULL_NAME: the hash, then the string.
    unsafe { item.as_mut_ptr().cast::<u8>().add(FULL_NAME + 8).cast::<MsvcString>().write(text) };
    let counter = Box::new([item.as_ptr().addr()]);
    (item, counter)
}

// A stack buffer whose user data is `user_data` (0 for none), and its item.
struct FakeStack {
    buffer: Box<[u64; 19]>,
    _item: Box<[u64; 16]>,
    _counter: Box<[usize; 1]>,
}

impl FakeStack {
    fn new(name: &str, user_data: usize) -> Self {
        let (item, counter) = fake_item(name);
        let mut buffer = Box::new([0u64; 19]);
        buffer[0] = STACK_VTABLE.as_ptr().addr() as u64;
        buffer[1] = counter.as_ptr().addr() as u64;
        buffer[2] = user_data as u64;
        Self { buffer, _item: item, _counter: counter }
    }

    fn address(&self) -> usize {
        self.buffer.as_ptr().addr()
    }
}

struct Items {
    _stone: (Box<[u64; 16]>, Box<[usize; 1]>),
    _bundle: (Box<[u64; 16]>, Box<[usize; 1]>),
}

impl Items {
    fn new() -> Self {
        let stone = fake_item("minecraft:stone");
        let bundle = fake_item("minecraft:red_bundle");
        STONE_COUNTER.store(stone.1.as_ptr().addr(), Ordering::SeqCst);
        BUNDLE_COUNTER.store(bundle.1.as_ptr().addr(), Ordering::SeqCst);
        UNKNOWN_TAG.store(0, Ordering::SeqCst);
        Self { _stone: stone, _bundle: bundle }
    }
}

fn entries() -> Vec<(&'static str, Resolved)> {
    let function = |address: usize| Resolved {
        kind: EntryKind::Function,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
    let global = |address: usize| Resolved {
        kind: EntryKind::Global,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let mut all = ClientGame::entries(&[]);
    all.extend([
        ("Player.playerInventory", field(0x5e0)),
        ("PlayerInventory.mInventory", field(0xb8)),
        ("PlayerInventory.mSelected", field(0x10)),
        ("FillingContainer.mItems", field(0x198)),
        ("ItemStack.size", field(0x98)),
        ("ItemStackBase.mItem", field(0x8)),
        ("ItemStackBase.mCount", field(0x22)),
        ("ItemStackBase.getDamageValue", function(damage as *const () as usize)),
        ("Item.getMaxDamage", slot(0x24)),
        ("Item.mFullName", field(FULL_NAME as u64)),
        ("HashedString.layout", field(0)),
        ("Actor.getItemInHandSlot", function(damage as *const () as usize)),
        ("ItemStackBase.mBlock", field(0x18)),
        ("Block.mDirectData.mLightEmission", field(0x10)),
        ("ItemStack.fromTag", function(from_tag as *const () as usize)),
        ("ItemStackBase.dtor", function(dtor as *const () as usize)),
        ("CompoundTag.nodeVariantIndex", field(0x68)),
        ("CompoundTag.vtable", global(FakeCompound::compound_vtable())),
    ]);
    all
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    SessionAdapter::install(harness.state);
    InventoryAdapter::install(harness.state);
    ItemDrawAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let provider = EventsMod::build("provider", &["provider"]);
    harness.install_mod("events", "events-provider", &provider, "events_provider.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    harness
}

fn table(harness: &Harness) -> &'static AmmlInventoryTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    unsafe { &*AmmlServices::inventory(services).unwrap() }
}

fn item_entry(slot: i8) -> FakeCompound {
    FakeCompound::build(vec![("Count", FakeValue::Int(1)), ("Slot", FakeValue::Byte(slot))])
}

fn bundle_entry(slot: i8, user_data: &FakeCompound) -> FakeCompound {
    let tag = item_entry(slot);
    BUNDLES.lock().unwrap().push((tag.ptr().addr(), user_data.ptr().addr()));
    tag
}

fn box_of(entries: Vec<FakeCompound>) -> FakeCompound {
    FakeCompound::build(vec![("Items", FakeValue::List(entries))])
}

fn bundle_data(entries: Vec<FakeCompound>) -> FakeCompound {
    FakeCompound::build(vec![
        ("bundle_weight", FakeValue::Int(40)),
        ("storage_item_component_content", FakeValue::List(entries)),
    ])
}

// Lends `stack` in the running `ScreenRender` scope, the way a container screen read does.
fn lend(stack: &FakeStack) -> *const c_void {
    let lent = InventoryAdapter::get().unwrap().lend_visual(stack.address()).unwrap();
    std::ptr::with_exposed_provenance(lent)
}

fn open(table: &AmmlInventoryTable, stack: *const c_void) -> (AmmlStatus, u64, u32) {
    let (mut contents, mut len) = (0u64, 0u32);
    // SAFETY: the table is live; the outputs are live locals.
    let status = unsafe {
        (table.contents_open.unwrap())(AmmlStr::from_str(MODULE), std::ptr::null(), stack, &mut contents, &mut len)
    };
    (status, contents, len)
}

fn read(table: &AmmlInventoryTable, contents: u64, index: u32) -> (AmmlStatus, AmmlContainerEntry) {
    let mut entry = AmmlContainerEntry::empty();
    // SAFETY: the table is live; `entry` is a live local.
    let status = unsafe { (table.contents_entry.unwrap())(AmmlStr::from_str(MODULE), contents, index, &mut entry) };
    (status, entry)
}

fn name(table: &AmmlInventoryTable, contents: u64, index: u32) -> String {
    let (mut buffer, mut len) = ([0u8; 64], 0usize);
    // SAFETY: the table is live; the buffer and the length are live locals.
    let status = unsafe {
        (table.contents_name.unwrap())(AmmlStr::from_str(MODULE), contents, index, buffer.as_mut_ptr(), 64, &mut len)
    };
    assert_eq!(status, AmmlStatus::OK);
    String::from_utf8(buffer[..len].to_vec()).unwrap()
}

fn release(table: &AmmlInventoryTable, draw: AmmlItemDraw) -> AmmlStatus {
    // SAFETY: the table is live.
    unsafe { (table.item_draw_release.unwrap())(AmmlStr::from_str(MODULE), draw) }
}

fn close(table: &AmmlInventoryTable, contents: u64) {
    // SAFETY: the table is live.
    unsafe { (table.contents_close.unwrap())(AmmlStr::from_str(MODULE), contents) };
}

fn frees_done() -> usize {
    DTOR_CALLS.load(Ordering::SeqCst)
}

// Runs the work of one UI draw point, as the `ScreenView.render` detour does.
fn draw_point() {
    TexturePump::run(0);
}

#[test]
fn shulker_tag_with_bundles_gives_nested_contents() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let unknown = item_entry(5);
    UNKNOWN_TAG.store(unknown.ptr().addr(), Ordering::SeqCst);
    let data = bundle_data(vec![item_entry(1), item_entry(0)]);
    let tag = box_of(vec![item_entry(0), bundle_entry(3, &data), unknown]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });

    let (status, contents, len) = open(table, lend(&shulker));

    assert_eq!((status, len), (AmmlStatus::OK, 2));
    assert_ne!(contents, 0);
    let (status, plain) = read(table, contents, 0);
    assert_eq!(status, AmmlStatus::OK);
    assert_eq!((plain.slot, plain.item.count, plain.has_bundle_weight, plain.bundle), (0, 5, 0, 0));
    assert_ne!(plain.draw.id, 0);
    assert_eq!(name(table, contents, 0), "minecraft:stone");
    let (_, bundle) = read(table, contents, 1);
    assert_eq!((bundle.slot, bundle.has_bundle_weight, bundle.bundle_weight), (3, 1, 40));
    assert_ne!(bundle.bundle, 0);
    assert_eq!(name(table, contents, 1), "minecraft:red_bundle");
    let (status, inner) = read(table, bundle.bundle, 1);
    assert_eq!((status, inner.slot, inner.item.count), (AmmlStatus::OK, 0, 5));
    let (_, first) = read(table, bundle.bundle, 0);
    assert_eq!(first.slot, 1);
    // An entry read twice is refused; an index past the end is invalid.
    assert_eq!(read(table, contents, 0).0, AmmlStatus::REFUSED);
    assert_eq!(read(table, contents, 2).0, AmmlStatus::INVALID_ARGUMENT);

    close(table, bundle.bundle);
    close(table, contents);
    for draw in [plain.draw, bundle.draw, inner.draw, first.draw] {
        assert_eq!(release(table, draw), AmmlStatus::OK);
    }
    draw_point();
    // The four copies and the stack of the entry the game does not know, which is destroyed at once.
    assert_eq!(frees_done(), 5);
    // Every destroy went through the resolved entry, none through vtable slot 0.
    assert_eq!(SLOT_ZERO_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn item_without_a_tag_gives_no_contents() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let other = FakeCompound::build(vec![("Damage", FakeValue::Int(3))]);
    let plain = FakeStack::new("minecraft:stone", 0);
    let tagged = FakeStack::new("minecraft:stone", other.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });

    assert_eq!(open(table, lend(&plain)), (AmmlStatus::OK, 0, 0));
    assert_eq!(open(table, lend(&tagged)), (AmmlStatus::OK, 0, 0));
}

#[test]
fn closing_the_contents_frees_the_draws_nobody_read() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0), item_entry(1), item_entry(2)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let (_, contents, len) = open(table, lend(&shulker));
    assert_eq!(len, 3);
    let (_, kept) = read(table, contents, 1);

    close(table, contents);
    // A second close does nothing.
    close(table, contents);

    assert_eq!(read(table, contents, 0).0, AmmlStatus::STALE);
    draw_point();
    assert_eq!(frees_done(), 2);
    assert_eq!(release(table, kept.draw), AmmlStatus::OK);
    draw_point();
    assert_eq!(frees_done(), 3);
}

#[test]
fn a_draw_released_off_the_game_thread_is_freed_at_the_next_draw_point() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let (_, contents, _) = open(table, lend(&shulker));
    let (_, entry) = read(table, contents, 0);
    close(table, contents);

    let release_fn = table.item_draw_release.unwrap();
    // SAFETY: the table is live. The release is checked at once and only queued.
    let status = std::thread::scope(|scope| {
        scope.spawn(move || unsafe { release_fn(AmmlStr::from_str(MODULE), entry.draw) }).join().unwrap()
    });

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(frees_done(), 0);
    assert!(TexturePump::pending());
    assert_eq!(release(table, entry.draw), AmmlStatus::STALE);

    draw_point();

    assert_eq!(frees_done(), 1);
    assert!(!TexturePump::pending());
}

#[test]
fn world_leave_frees_every_copy_and_the_old_ids_are_stale() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0), item_entry(1)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let (_, contents, _) = open(table, lend(&shulker));
    let (_, entry) = read(table, contents, 0);

    ItemDrawAdapter::get().unwrap().world_left();
    draw_point();

    assert_eq!(frees_done(), 2);
    assert_eq!(read(table, contents, 1).0, AmmlStatus::STALE);
    assert_eq!(release(table, entry.draw), AmmlStatus::STALE);
}

#[test]
fn a_mod_holds_at_most_the_hard_bound_of_draws_and_a_refused_open_leaves_none_behind() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of((0..27).map(item_entry).collect());
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let fits = ItemDrawBook::MAX_PER_OWNER / 27;
    let mut opened = Vec::new();
    for _ in 0..fits {
        let (status, contents, len) = open(table, lend(&shulker));
        assert_eq!((status, len), (AmmlStatus::OK, 27));
        opened.push(contents);
    }

    // The next open makes the draws left under the bound, then the stack that passes it (destroyed at once), and gives
    // them all back.
    let left = ItemDrawBook::MAX_PER_OWNER - fits * 27;
    assert_eq!(open(table, lend(&shulker)).0, AmmlStatus::REFUSED);
    draw_point();
    assert_eq!(frees_done(), left + 1);
    close(table, opened[0]);
    draw_point();
    assert_eq!(frees_done(), left + 1 + 27);
    assert_eq!(open(table, lend(&shulker)).0, AmmlStatus::OK);
}

#[test]
fn contents_are_refused_off_the_game_thread_and_for_a_stack_that_was_not_lent() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let lent = lend(&shulker).addr();

    let off_thread = std::thread::scope(|scope| {
        scope.spawn(move || open(table, std::ptr::with_exposed_provenance(lent))).join().unwrap()
    });

    assert_eq!(off_thread.0, AmmlStatus::REFUSED);
    assert_eq!(open(table, std::ptr::with_exposed_provenance(shulker.address() + 8)).0, AmmlStatus::REFUSED);
}

// The calls the stand-in item renderer, context constructor and context destructor saw.
static RENDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CONTEXTS_BUILT: AtomicUsize = AtomicUsize::new(0);
static CONTEXTS_DESTROYED: AtomicUsize = AtomicUsize::new(0);
// The item renderer the stand-in context holds at +58h.
const RENDERER: usize = 0xAAAA;

extern "C" fn fake_context_ctor(this: *mut c_void, screen: usize, client: usize, game: usize) -> *mut c_void {
    CONTEXTS_BUILT.fetch_add(1, Ordering::SeqCst);
    assert!(screen == 0x7000 && client != 0 && game != 0, "{screen:#x} {client:#x} {game:#x}");
    // SAFETY: the loader gives a buffer of `BaseActorRenderContext.size` bytes.
    unsafe { this.cast::<usize>().add(0x58 / 8).write(RENDERER) };
    this
}

extern "C" fn fake_context_dtor(_: *mut c_void) {
    CONTEXTS_DESTROYED.fetch_add(1, Ordering::SeqCst);
}

#[allow(clippy::too_many_arguments)]
extern "C" fn fake_render(
    renderer: usize,
    context: usize,
    stack: usize,
    frame: i32,
    x: f32,
    y: f32,
    foil: u8,
    alpha: f32,
    light: f32,
    scale: f32,
    z: i32,
) {
    let live = context != 0 && stack != 0;
    let line = format!("{renderer:#x} {live} {frame} {x} {y} {foil} {alpha} {light} {scale} {z}");
    RENDERS.lock().unwrap().push(line);
}

extern "C" fn fake_gui_scale(_: usize) -> f32 {
    2.0
}

fn draw_entries() -> Vec<(&'static str, Resolved)> {
    let function = |address: usize| Resolved {
        kind: EntryKind::Function,
        value: address as u64 - Harness::BASE,
        recipes_agreeing: 2,
    };
    let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
    let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
    let mut all = entries();
    all.extend([
        ("ItemRenderer.renderGuiItemNew", function(fake_render as *const () as usize)),
        ("BaseActorRenderContext.ctor", function(fake_context_ctor as *const () as usize)),
        ("BaseActorRenderContext.dtor", function(fake_context_dtor as *const () as usize)),
        ("BaseActorRenderContext.size", field(0xc0)),
        ("Item.getAnimationFrameFor", slot(0x77)),
        ("MinecraftUIRenderContext.mScreenContext", field(8)),
    ]);
    all
}

// The stand-in `MinecraftUIRenderContext`: only the `ScreenContext*` at +8 is read.
struct FakeContext {
    object: Box<[usize; 4]>,
}

impl FakeContext {
    fn new() -> Self {
        Self { object: Box::new([0, 0x7000, 0, 0]) }
    }

    fn address(&self) -> usize {
        self.object.as_ptr().addr()
    }
}

fn start_drawing() -> Harness {
    let harness = Harness::start_with_entries(&draw_entries());
    SessionAdapter::install(harness.state);
    InventoryAdapter::install(harness.state);
    let calls = DrawCalls {
        fill_slot: 1,
        outline_slot: 2,
        image_slot: 3,
        flush_slot: 4,
        screen_context: 8,
        gui_data: 8,
        gui_scale: fake_gui_scale as *const () as usize,
        queued_images: None,
    };
    UiDrawAdapter::install_with(harness.state, calls);
    ItemDrawAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

fn ui_table(harness: &Harness) -> &'static AmmlUiTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    unsafe { &*AmmlServices::ui(services).unwrap() }
}

fn draw_item(table: &AmmlUiTable, view: u64, draw: AmmlItemDraw, flags: AmmlItemDrawFlags) -> AmmlStatus {
    // SAFETY: the table is live.
    unsafe {
        (table.draw_item.unwrap())(AmmlStr::from_str(MODULE), view, draw, AmmlPoint { x: 4.0, y: 5.0 }, flags, 0.5, 3)
    }
}

#[test]
fn item_draws_share_one_render_context_for_a_dispatch_and_pass_the_game_arguments() {
    let _items = Items::new();
    let harness = start_drawing();
    let (inventory, ui) = (table(&harness), ui_table(&harness));
    let tag = box_of(vec![item_entry(0), item_entry(1)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let context = FakeContext::new();
    let scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let (_, contents, _) = open(inventory, lend(&shulker));
    let (_, first) = read(inventory, contents, 0);
    let (_, second) = read(inventory, contents, 1);

    let statuses = [
        draw_item(ui, 7, first.draw, AmmlItemDrawFlags::FOIL_ONLY),
        draw_item(ui, 7, second.draw, AmmlItemDrawFlags::NONE),
    ];
    ItemDrawAdapter::get().unwrap().finish();

    assert_eq!(statuses, [AmmlStatus::OK; 2]);
    assert_eq!(
        RENDERS.lock().unwrap().clone(),
        vec!["0xaaaa true 0 4 5 1 0.5 1 1 3".to_string(), "0xaaaa true 0 4 5 0 0.5 1 1 3".to_string()]
    );
    assert_eq!((CONTEXTS_BUILT.load(Ordering::SeqCst), CONTEXTS_DESTROYED.load(Ordering::SeqCst)), (1, 1));

    // The next dispatch builds its own context.
    drop(scope);
    let _next = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 8 }, context.address());
    assert_eq!(draw_item(ui, 8, first.draw, AmmlItemDrawFlags::NONE), AmmlStatus::OK);
    assert_eq!(CONTEXTS_BUILT.load(Ordering::SeqCst), 2);
}

#[test]
fn an_item_draw_outside_its_view_with_odd_flags_or_after_a_world_leave_is_refused() {
    let _items = Items::new();
    let harness = start_drawing();
    let (inventory, ui) = (table(&harness), ui_table(&harness));
    let tag = box_of(vec![item_entry(0)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let context = FakeContext::new();
    let _scope = UiScope::enter_with_context(UiScopeKind::ScreenRender { view: 7 }, context.address());
    let (_, contents, _) = open(inventory, lend(&shulker));
    let (_, entry) = read(inventory, contents, 0);

    assert_eq!(draw_item(ui, 9, entry.draw, AmmlItemDrawFlags::NONE), AmmlStatus::REFUSED);
    assert_eq!(draw_item(ui, 7, entry.draw, AmmlItemDrawFlags(0x80)), AmmlStatus::INVALID_ARGUMENT);
    let unknown = AmmlItemDraw { id: entry.draw.id + 99, ..entry.draw };
    assert_eq!(draw_item(ui, 7, unknown, AmmlItemDrawFlags::NONE), AmmlStatus::STALE);
    assert!(RENDERS.lock().unwrap().is_empty());

    ItemDrawAdapter::get().unwrap().world_left();

    assert_eq!(draw_item(ui, 7, entry.draw, AmmlItemDrawFlags::NONE), AmmlStatus::STALE);
    assert!(RENDERS.lock().unwrap().is_empty());
}

#[test]
fn a_box_of_27_full_bundles_opens_with_the_contents_of_every_bundle() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let datas: Vec<FakeCompound> = (0..27).map(|_| bundle_data((0..64).map(item_entry).collect())).collect();
    let tag = box_of(datas.iter().enumerate().map(|(slot, data)| bundle_entry(slot as i8, data)).collect());
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });

    let (status, contents, len) = open(table, lend(&shulker));

    assert_eq!((status, len), (AmmlStatus::OK, 27));
    let bundles: Vec<u64> = (0..27).map(|index| read(table, contents, index).1.bundle).collect();
    assert!(bundles.iter().all(|id| *id != 0), "{bundles:?}");
    let (status, last) = read(table, bundles[26], 63);
    assert_eq!((status, last.slot), (AmmlStatus::OK, 63));
}

#[test]
fn one_open_keeps_its_budget_and_the_bundles_past_it_lose_their_contents() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    // 27 bundles of 64 items fill the budget of one open; the first bundle also holds a bundle of 10 items, so the
    // last bundle runs out after 54 of its items.
    let inner = bundle_data((0..10).map(item_entry).collect());
    let mut first: Vec<FakeCompound> = vec![bundle_entry(0, &inner)];
    first.extend((1..64).map(item_entry));
    let mut datas = vec![bundle_data(first)];
    datas.extend((1..27).map(|_| bundle_data((0..64).map(item_entry).collect())));
    let tag = box_of(datas.iter().enumerate().map(|(slot, data)| bundle_entry(slot as i8, data)).collect());
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });

    let (status, contents, len) = open(table, lend(&shulker));

    assert_eq!((status, len), (AmmlStatus::OK, 27));
    let bundles: Vec<u64> = (0..27).map(|index| read(table, contents, index).1.bundle).collect();
    assert!(bundles[..26].iter().all(|id| *id != 0), "{bundles:?}");
    assert_eq!(bundles[26], 0);
    // The last bundle made 54 items before the budget and one stack past it: all 55 were given back.
    draw_point();
    assert_eq!(frees_done(), 55);
}

#[test]
fn another_mod_cannot_read_name_or_close_a_contents_id() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0), item_entry(1)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let (_, contents, _) = open(table, lend(&shulker));
    let other = AmmlStr::from_str(OTHER);
    let mut entry = AmmlContainerEntry::empty();
    let (mut buffer, mut len) = ([0u8; 16], 0usize);

    // SAFETY: the table is live; the outputs are live locals.
    let results = unsafe {
        (
            (table.contents_entry.unwrap())(other, contents, 0, &mut entry),
            (table.contents_name.unwrap())(other, contents, 0, buffer.as_mut_ptr(), 16, &mut len),
        )
    };
    // SAFETY: the table is live. The close of another mod does nothing.
    unsafe { (table.contents_close.unwrap())(other, contents) };

    assert_eq!(results, (AmmlStatus::REFUSED, AmmlStatus::REFUSED));
    // The owner still reads both entries, so the close of the other mod freed nothing.
    assert_eq!(read(table, contents, 0).0, AmmlStatus::OK);
    assert_eq!(read(table, contents, 1).0, AmmlStatus::OK);
}

fn change_stamp(table: &AmmlInventoryTable, stack: *const c_void) -> (AmmlStatus, u64) {
    let mut stamp = 0u64;
    // SAFETY: the table is live; `stamp` is a live local.
    let status = unsafe { (table.item_change_stamp.unwrap())(std::ptr::null(), stack, &mut stamp) };
    (status, stamp)
}

#[test]
fn the_change_stamp_is_steady_for_one_tag_and_differs_for_another_tag_with_the_same_contents() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let first = box_of(vec![item_entry(0), item_entry(1)]);
    let second = box_of(vec![item_entry(0), item_entry(1)]);
    let one = FakeStack::new("minecraft:shulker_box", first.ptr().addr());
    let two = FakeStack::new("minecraft:shulker_box", second.ptr().addr());
    let plain = FakeStack::new("minecraft:stone", 0);
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });

    let (status, a) = change_stamp(table, lend(&one));
    let (_, again) = change_stamp(table, lend(&one));
    let (_, b) = change_stamp(table, lend(&two));
    let (plain_status, none) = change_stamp(table, lend(&plain));

    assert_eq!(status, AmmlStatus::OK);
    assert_eq!(a, again);
    assert_ne!(a, b);
    assert_ne!(a, 0);
    assert_eq!((plain_status, none), (AmmlStatus::OK, 0));
}

#[test]
fn the_change_stamp_changes_when_a_nested_tag_is_edited_in_place() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let mut tag = box_of(vec![item_entry(0)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let (_, before) = change_stamp(table, lend(&shulker));

    // The item in the box moves to slot 9: the box tag and the item tag keep their addresses.
    tag.child_mut(0).set_byte("Slot", 9);
    let (_, after) = change_stamp(table, lend(&shulker));
    tag.child_mut(0).set_byte("Slot", 0);
    let (_, back) = change_stamp(table, lend(&shulker));

    assert_ne!(before, after);
    assert_eq!(before, back);
}

#[test]
fn the_change_stamp_is_refused_off_the_game_thread_and_for_a_stack_that_was_not_lent() {
    let _items = Items::new();
    let harness = start();
    ClientGame::frame();
    let table = table(&harness);
    let tag = box_of(vec![item_entry(0)]);
    let shulker = FakeStack::new("minecraft:shulker_box", tag.ptr().addr());
    let _scope = UiScope::enter(UiScopeKind::ScreenRender { view: 1 });
    let lent = lend(&shulker).addr();

    let off_thread = std::thread::scope(|scope| {
        scope.spawn(move || change_stamp(table, std::ptr::with_exposed_provenance(lent))).join().unwrap()
    });

    assert_eq!(off_thread.0, AmmlStatus::REFUSED);
    assert_eq!(change_stamp(table, std::ptr::with_exposed_provenance(shulker.address() + 8)).0, AmmlStatus::REFUSED);
}
