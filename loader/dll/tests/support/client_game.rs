use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{
    ActiveCameraGame, ActivityGame, FakeVtable, Harness, HeapProbe, InfoGame, RouterGame, SessionGame, WorldGame,
};

/// A stand-in `ClientInstance`: only its vtable pointer.
pub type FakeGameClient = &'static FakeVtable<12>;

// The IClientInstance vtable: 0 getLocalPlayer, 1 getMinecraftInput, 2 isWorldActive, 3 isShowingMenu,
// 4 getTopScreenName, 5 getOptions, 6 getMinecraftGame, 7 getGameConnectionInfo, 8 getLevel, 9 isInRealm,
// 10 getCameraRegistry, 11 getCurrentUIRouter.
static CLIENT_VTABLE: FakeVtable<12> = FakeVtable([
    ClientGame::local_player as *const (),
    ActivityGame::minecraft_input as *const (),
    ClientGame::world_active as *const (),
    ClientGame::showing_menu as *const (),
    ClientGame::top_screen_name as *const (),
    ClientGame::options as *const (),
    ClientGame::minecraft_game as *const (),
    InfoGame::connection_info as *const (),
    InfoGame::get_level as *const (),
    InfoGame::is_in_realm as *const (),
    ActiveCameraGame::camera_registry as *const (),
    RouterGame::current_router as *const (),
]);
// The MinecraftGame vtable: 2 update, 3 grabMouse, 4 releaseMouse, 5 getWorldSystem.
static GAME_VTABLE: FakeVtable<6> = FakeVtable([
    ClientGame::unused as *const (),
    ClientGame::unused as *const (),
    ClientGame::game_update as *const (),
    ClientGame::grab_mouse as *const (),
    ClientGame::release_mouse as *const (),
    WorldGame::world_system as *const (),
]);
// The AppPlatformListener vtable of MinecraftGame: 0 onAppFocusLost, 1 onAppFocusGained.
static LISTENER_VTABLE: FakeVtable<2> =
    FakeVtable([ClientGame::focus_lost as *const (), ClientGame::focus_gained as *const ()]);
// The IOptionRegistry vtable: 0 getHideHud, 1 setHideGUI, 2 getIfValid, 3 getEffectiveMaxFramerateOption,
// 4 getViewDistanceChunks.
static OPTIONS_VTABLE: FakeVtable<5> = FakeVtable([
    ClientGame::get_hide_hud as *const (),
    ClientGame::set_hide_gui as *const (),
    ClientGame::get_if_valid as *const (),
    ClientGame::effective_max_framerate as *const (),
    ClientGame::view_distance as *const (),
]);
static OPTIONS: &FakeVtable<5> = &OPTIONS_VTABLE;
// The view distance `getViewDistanceChunks` gives.
static VIEW_DISTANCE: AtomicI32 = AtomicI32::new(12);
// The `IntOption::set` calls: (the option, the value, the save flag).
static INT_SETS: Mutex<Vec<(usize, i32, bool)>> = Mutex::new(Vec::new());
// The options `getIfValid` gives: (id, the address of a stand-in `Option`).
static OPTION_LIST: Mutex<Vec<(i32, usize)>> = Mutex::new(Vec::new());
// The answer of the registry's slot 13 that `getEffectiveMaxFramerateOption` reads: 2 selects option F7h.
static FRAMERATE_SLOT_13: AtomicI32 = AtomicI32::new(0);
// The stand-in `Option` of `pause_menu_on_focus_lost`, 0 while there is none.
static PAUSE_OPTION: AtomicUsize = AtomicUsize::new(0);

/// The first client.
pub static GAME_CLIENT_A: FakeGameClient = &CLIENT_VTABLE;
/// A second client.
pub static GAME_CLIENT_B: FakeGameClient = &CLIENT_VTABLE;

// A stand-in MinecraftGame of 0x200 bytes: its vtable at +0 and its AppPlatformListener vtable at the offset
// `place_listener` sets.
static GAME: [AtomicUsize; 64] = [const { AtomicUsize::new(0) }; 64];

static PLAYER: AtomicUsize = AtomicUsize::new(0);
static WORLD_ACTIVE: AtomicBool = AtomicBool::new(true);
static SHOWING_MENU: AtomicBool = AtomicBool::new(false);
static HIDE_HUD: AtomicBool = AtomicBool::new(false);
static SCREEN: Mutex<String> = Mutex::new(String::new());
static HUD_SETS: Mutex<Vec<bool>> = Mutex::new(Vec::new());
static FOCUS_LOST_THIS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
// Calls of each stand-in, in the order of `ClientGame::calls`.
static CALLS: [AtomicUsize; 10] = [const { AtomicUsize::new(0) }; 10];

/// Stand-ins for the game functions the client adapter calls and hooks: a client with its vtable, a
/// `MinecraftGame` with an `AppPlatformListener` subobject, and an option registry.
pub struct ClientGame;

impl ClientGame {
    /// The offset of the listener subobject in the derived entry.
    pub const LISTENER_OFFSET: usize = 0x88;
    /// The id of the stand-in effective max frame rate option.
    pub const MAX_FRAMERATE_ID: i32 = 0xF6;
    /// The id of the other max frame rate option, effective while slot 13 of the registry answers 2.
    pub const MAX_FRAMERATE_OTHER_ID: i32 = 0xF7;
    /// The id of the stand-in `gfx_vsync` option.
    pub const VSYNC_ID: i32 = 3;
    /// The id of the stand-in `pause_menu_on_focus_lost` option (its 26.63 id).
    pub const PAUSE_OPTION_ID: i32 = 0x32F;
    // The game's `Option` and `Option::Impl` offsets the loader reads.
    const OPTION_SIZE: usize = 0x30;
    const IMPL_SIZE: usize = 0x240;
    const IMPL_AT: usize = 0x8;
    const BOOL_VALUE_AT: usize = 0x10;
    const INT_VALUE_AT: usize = 0x18;
    const SAVE_TAG_AT: usize = 0x188;
    const ID_AT: usize = 0x1CC;
    const TYPE_AT: usize = 0x1D4;
    // Indexes into `CALLS`.
    const WORLD_ACTIVE: usize = 0;
    const SHOWING_MENU: usize = 1;
    const TOP_SCREEN: usize = 2;
    const OPTIONS: usize = 3;
    const MINECRAFT_GAME: usize = 4;
    const GET_HIDE_HUD: usize = 5;
    const GRAB: usize = 6;
    const RELEASE: usize = 7;
    const FOCUS_LOST: usize = 8;
    const FOCUS_GAINED: usize = 9;

    /// The session entries and every client entry, pointing to the stand-ins; `without` names entries to
    /// leave out. The listener subobject is placed at `LISTENER_OFFSET`.
    /// Three options are in the registry: two others and `pause_menu_on_focus_lost`, on.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::place_listener(Self::LISTENER_OFFSET);
        Self::place_options(true);
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let global = |address: usize| Resolved {
            kind: EntryKind::Global,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let mut entries =
            SessionGame::entries(&["MinecraftGame.update", "MinecraftGame.vtable", "ClientInstance.getLocalPlayer"]);
        entries.extend([
            ("MinecraftGame.update", slot(2)),
            ("MinecraftGame.vtable", global(std::ptr::from_ref(&GAME_VTABLE) as usize)),
            ("ClientInstance.getLocalPlayer", slot(0)),
            ("ClientInstance.isWorldActive", slot(2)),
            ("ClientInstance.isShowingMenu", slot(3)),
            ("ClientInstance.getTopScreenName", slot(4)),
            ("ClientInstance.getOptions", slot(5)),
            ("ClientInstance.getMinecraftGame", slot(6)),
            ("IOptionRegistry.getHideHud", slot(0)),
            ("IOptionRegistry.setHideGUI", slot(1)),
            ("OptionRegistry.getIfValid", slot(2)),
            ("OptionRegistry.getEffectiveMaxFramerateOption", slot(3)),
            ("IOptionRegistry.getViewDistanceChunks", slot(4)),
            ("IntOption.set", function(Self::int_option_set as *const () as usize)),
            ("IMinecraftGame.grabMouse", slot(3)),
            ("IMinecraftGame.releaseMouse", slot(4)),
            ("MinecraftGame.grabMouse", function(Self::grab_mouse as *const () as usize)),
            ("MinecraftGame.onAppFocusLost", function(Self::focus_lost as *const () as usize)),
            ("MinecraftGame.onAppFocusGained", slot(1)),
            ("MinecraftGame.AppPlatformListener.vtable", global(std::ptr::from_ref(&LISTENER_VTABLE) as usize)),
            ("MinecraftGame.appPlatformListenerOffset", field(Self::LISTENER_OFFSET as u64)),
        ]);
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    pub fn client(client: &'static FakeGameClient) -> *mut c_void {
        std::ptr::from_ref(client).cast_mut().cast()
    }

    /// The address of the stand-in MinecraftGame.
    pub fn game() -> usize {
        std::ptr::from_ref(&GAME) as usize
    }

    /// Puts the MinecraftGame vtable at +0 and the AppPlatformListener vtable at `offset` (and nowhere else).
    pub fn place_listener(offset: usize) {
        for word in &GAME {
            word.store(0, Ordering::SeqCst);
        }
        GAME[0].store(std::ptr::from_ref(&GAME_VTABLE) as usize, Ordering::SeqCst);
        GAME[offset / 8].store(std::ptr::from_ref(&LISTENER_VTABLE) as usize, Ordering::SeqCst);
    }

    /// Fills the option registry: a bool option `gfx_vsync` (id 3, a small string tag), an int option
    /// `ctrl_sensitivity_value` (id 0x40) and, when `with_pause` is true, the bool option
    /// `pause_menu_on_focus_lost` (id `PAUSE_OPTION_ID`), on.
    pub fn place_options(with_pause: bool) {
        let mut list = vec![
            (3, Self::make_option(3, "gfx_vsync", 6, 1)),
            (0x40, Self::make_option(0x40, "ctrl_sensitivity_value", 5, 0)),
            (Self::MAX_FRAMERATE_ID, Self::make_option(Self::MAX_FRAMERATE_ID, "gfx_max_framerate", 5, 60)),
            (
                Self::MAX_FRAMERATE_OTHER_ID,
                Self::make_option(Self::MAX_FRAMERATE_OTHER_ID, "gfx_max_framerate_b", 5, 30),
            ),
        ];
        FRAMERATE_SLOT_13.store(0, Ordering::SeqCst);
        let pause = if with_pause {
            let option = Self::make_option(Self::PAUSE_OPTION_ID, "pause_menu_on_focus_lost", 0, 1);
            list.push((Self::PAUSE_OPTION_ID, option));
            option
        } else {
            0
        };
        PAUSE_OPTION.store(pause, Ordering::SeqCst);
        *OPTION_LIST.lock().unwrap() = list;
    }

    /// Adds an option with id `id` and the tag `tag` (a second pause option makes the tag ambiguous).
    pub fn add_option(id: i32, tag: &str) {
        let option = Self::make_option(id, tag, 0, 1);
        OPTION_LIST.lock().unwrap().push((id, option));
    }

    /// Makes the option with id `id` give `own_id` in its `mID`, as a build with another Impl layout would.
    pub fn set_option_own_id(id: i32, own_id: i32) {
        let option = OPTION_LIST.lock().unwrap().iter().find(|(known, _)| *known == id).unwrap().1;
        // SAFETY: `option` is a leaked stand-in `Option` whose Impl is `IMPL_SIZE` bytes.
        unsafe {
            let implementation = ((option + Self::IMPL_AT) as *const usize).read();
            ((implementation + Self::ID_AT) as *mut i32).write_unaligned(own_id);
        }
    }

    /// The int value of the option with id `id`, as the game holds it.
    pub fn int_option(id: i32) -> i32 {
        let option = Self::option_address(id);
        // SAFETY: `option` is a leaked stand-in `Option` of `OPTION_SIZE` bytes.
        unsafe { ((option + Self::INT_VALUE_AT) as *const i32).read_unaligned() }
    }

    /// Changes the int value of the option with id `id`, as the user does in the settings screen.
    pub fn set_int_option(id: i32, value: i32) {
        let option = Self::option_address(id);
        // SAFETY: `option` is a leaked stand-in `Option` of `OPTION_SIZE` bytes.
        unsafe { ((option + Self::INT_VALUE_AT) as *mut i32).write_unaligned(value) };
    }

    /// The `IntOption::set` calls so far, in order: (the option id, the value, the save flag).
    pub fn int_sets() -> Vec<(i32, i32, bool)> {
        let sets = INT_SETS.lock().unwrap().clone();
        sets.into_iter()
            .map(|(option, value, save)| {
                let id = OPTION_LIST.lock().unwrap().iter().find(|(_, held)| *held == option).map(|(id, _)| *id);
                (id.unwrap_or(-1), value, save)
            })
            .collect()
    }

    /// The answer of the registry's slot 13 from now on: 2 makes option F7h the effective max frame rate option.
    pub fn set_framerate_slot_13(answer: i32) {
        FRAMERATE_SLOT_13.store(answer, Ordering::SeqCst);
    }

    /// The view distance `getViewDistanceChunks` gives from now on.
    pub fn set_view_distance(chunks: i32) {
        VIEW_DISTANCE.store(chunks, Ordering::SeqCst);
    }

    fn option_address(id: i32) -> usize {
        OPTION_LIST.lock().unwrap().iter().find(|(known, _)| *known == id).unwrap().1
    }

    /// The value of `pause_menu_on_focus_lost`.
    pub fn set_pause_on_focus_lost(on: bool) {
        let option = PAUSE_OPTION.load(Ordering::SeqCst);
        assert_ne!(option, 0, "no pause option");
        // SAFETY: `option` is a leaked stand-in `Option` of `OPTION_SIZE` bytes.
        unsafe { ((option + Self::BOOL_VALUE_AT) as *mut u8).write(u8::from(on)) };
    }

    // A leaked stand-in `Option` and its `Option::Impl`: the Impl pointer at +8, the value at +10h, and in the
    // Impl the save tag (an MSVC string), the id, the option type, and no override.
    fn make_option(id: i32, tag: &str, option_type: i32, value: u8) -> usize {
        let option: &'static mut [u64] = Box::leak(vec![0u64; Self::OPTION_SIZE / 8].into_boxed_slice());
        let implementation: &'static mut [u64] = Box::leak(vec![0u64; Self::IMPL_SIZE / 8].into_boxed_slice());
        let option = option.as_mut_ptr() as usize;
        let implementation = implementation.as_mut_ptr() as usize;
        let mut words = [0usize; 4];
        if tag.len() <= 15 {
            let mut inline = [0u8; 16];
            inline[..tag.len()].copy_from_slice(tag.as_bytes());
            words[0] = usize::from_le_bytes(inline[..8].try_into().unwrap());
            words[1] = usize::from_le_bytes(inline[8..].try_into().unwrap());
            words[3] = 15;
        } else {
            let text: &'static mut str = Box::leak(tag.to_string().into_boxed_str());
            words[0] = text.as_ptr() as usize;
            words[3] = tag.len();
        }
        words[2] = tag.len();
        // SAFETY: both blocks are leaked, writable and large enough for every offset written.
        unsafe {
            ((option + Self::IMPL_AT) as *mut usize).write(implementation);
            ((option + Self::BOOL_VALUE_AT) as *mut u8).write(value);
            ((option + Self::INT_VALUE_AT) as *mut i32).write_unaligned(i32::from(value));
            ((implementation + Self::SAVE_TAG_AT) as *mut [usize; 4]).write(words);
            ((implementation + Self::ID_AT) as *mut i32).write_unaligned(id);
            ((implementation + Self::TYPE_AT) as *mut i32).write_unaligned(option_type);
        }
        option
    }

    /// What the local player read gives from now on; 0 is null.
    pub fn set_player(value: usize) {
        PLAYER.store(value, Ordering::SeqCst);
    }

    /// The name `getTopScreenName` gives from now on.
    pub fn set_screen(name: &str) {
        *SCREEN.lock().unwrap() = name.to_string();
    }

    /// The player's own hide-HUD value.
    pub fn set_hide_hud(hidden: bool) {
        HIDE_HUD.store(hidden, Ordering::SeqCst);
    }

    /// Calls (through its address) `MinecraftGame::update` on the stand-in game.
    pub fn frame() {
        // SAFETY: slot 2 holds `game_update`, an `extern "C" fn(*mut c_void)`.
        let update = unsafe { Self::slot_fn(&GAME_VTABLE.0, 2) };
        update(black_box(Self::game() as *mut c_void));
    }

    /// `ClientInstance::update` on `client`.
    pub fn update(client: &'static FakeGameClient) {
        SessionGame::update_raw(Self::client(client), 1, 0);
    }

    /// The game grabs the mouse: a call through slot 3 of the MinecraftGame vtable.
    pub fn game_grab() {
        // SAFETY: slot 3 holds `grab_mouse`, an `extern "C" fn(*mut c_void)`.
        let grab = unsafe { Self::slot_fn(&GAME_VTABLE.0, 3) };
        grab(black_box(Self::game() as *mut c_void));
    }

    /// The game releases the mouse: a call through slot 4 of the MinecraftGame vtable.
    pub fn game_release() {
        // SAFETY: slot 4 holds `release_mouse`, an `extern "C" fn(*mut c_void)`.
        let release = unsafe { Self::slot_fn(&GAME_VTABLE.0, 4) };
        release(black_box(Self::game() as *mut c_void));
    }

    /// The app loses focus: `onAppFocusLost` through the listener vtable, on the listener subobject.
    pub fn game_focus_lost() {
        // SAFETY: slot 0 holds `focus_lost`, an `extern "C" fn(*mut c_void)`.
        let focus_lost = unsafe { Self::slot_fn(&LISTENER_VTABLE.0, 0) };
        focus_lost(black_box((Self::game() + Self::LISTENER_OFFSET) as *mut c_void));
    }

    /// The app gains focus: `onAppFocusGained` through the listener vtable, on the listener subobject.
    pub fn game_focus_gained() {
        // SAFETY: slot 1 holds `focus_gained`, an `extern "C" fn(*mut c_void)`.
        let focus_gained = unsafe { Self::slot_fn(&LISTENER_VTABLE.0, 1) };
        focus_gained(black_box((Self::game() + Self::LISTENER_OFFSET) as *mut c_void));
    }

    /// How many times the game's `onAppFocusGained` ran.
    pub fn focus_gains() -> usize {
        CALLS[Self::FOCUS_GAINED].load(Ordering::SeqCst)
    }

    /// Calls of the stand-ins: isWorldActive, isShowingMenu, getTopScreenName, getOptions,
    /// getMinecraftGame, getHideHud, grabMouse, releaseMouse, onAppFocusLost.
    pub fn calls() -> [usize; 9] {
        std::array::from_fn(|index| CALLS[index].load(Ordering::SeqCst))
    }

    pub fn grabs() -> usize {
        CALLS[Self::GRAB].load(Ordering::SeqCst)
    }

    /// The values `setHideGUI` received, in order.
    pub fn hud_sets() -> Vec<bool> {
        HUD_SETS.lock().unwrap().clone()
    }

    /// The `this` of each `onAppFocusLost` call, in order.
    pub fn focus_lost_this() -> Vec<usize> {
        FOCUS_LOST_THIS.lock().unwrap().clone()
    }

    // The function in slot `index` of `table`, as a one-argument `extern "C"` function.
    //
    // Safety: the slot holds an `extern "C" fn(*mut c_void)`.
    unsafe fn slot_fn(table: &[*const ()], index: usize) -> extern "C" fn(*mut c_void) {
        // SAFETY: the caller guarantees the slot's signature.
        unsafe { std::mem::transmute::<*const (), extern "C" fn(*mut c_void)>(black_box(table[index])) }
    }

    fn count(index: usize) {
        CALLS[index].fetch_add(black_box(1), Ordering::SeqCst);
    }

    #[inline(never)]
    extern "C" fn unused(this: *mut c_void) {
        black_box(this);
    }

    #[inline(never)]
    extern "C" fn local_player(this: *mut c_void) -> *mut c_void {
        black_box(this);
        PLAYER.load(Ordering::SeqCst) as *mut c_void
    }

    #[inline(never)]
    extern "C" fn world_active(this: *mut c_void) -> u8 {
        black_box(this);
        Self::count(Self::WORLD_ACTIVE);
        u8::from(WORLD_ACTIVE.load(Ordering::SeqCst))
    }

    #[inline(never)]
    extern "C" fn showing_menu(this: *mut c_void) -> u8 {
        black_box(this);
        Self::count(Self::SHOWING_MENU);
        u8::from(SHOWING_MENU.load(Ordering::SeqCst))
    }

    // `std::string getTopScreenName() const`: builds the name in `out` as a heap-mode MSVC string (capacity
    // 31) on the probe heap, so the caller must free it once.
    #[inline(never)]
    extern "C" fn top_screen_name(this: *mut c_void, out: *mut [usize; 4]) -> *mut [usize; 4] {
        black_box(this);
        Self::count(Self::TOP_SCREEN);
        let name = SCREEN.lock().unwrap().clone();
        let bytes = &name.as_bytes()[..name.len().min(31)];
        // SAFETY: the probe heap returns 32 writable bytes.
        let block = unsafe { HeapProbe::alloc(32) };
        // SAFETY: `block` holds 32 bytes; at most 31 name bytes and the terminator are written.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), block, bytes.len());
            block.add(bytes.len()).write(0);
        }
        // SAFETY: the caller passes room for one 32-byte string.
        unsafe { out.write([block as usize, 0, bytes.len(), 31]) };
        out
    }

    #[inline(never)]
    extern "C" fn options(this: *mut c_void) -> *mut c_void {
        black_box(this);
        Self::count(Self::OPTIONS);
        std::ptr::from_ref(&OPTIONS).cast_mut().cast()
    }

    #[inline(never)]
    extern "C" fn minecraft_game(this: *mut c_void) -> *mut c_void {
        black_box(this);
        Self::count(Self::MINECRAFT_GAME);
        Self::game() as *mut c_void
    }

    #[inline(never)]
    extern "C" fn get_hide_hud(this: *mut c_void) -> u8 {
        black_box(this);
        Self::count(Self::GET_HIDE_HUD);
        u8::from(HIDE_HUD.load(Ordering::SeqCst))
    }

    #[inline(never)]
    extern "C" fn set_hide_gui(this: *mut c_void, hidden: u8) {
        black_box(this);
        HIDE_HUD.store(hidden != 0, Ordering::SeqCst);
        HUD_SETS.lock().unwrap().push(hidden != 0);
    }

    // `std::optional<Option*> getIfValid(OptionID)`: fills the 16-byte optional at `out`.
    #[inline(never)]
    extern "C" fn get_if_valid(this: *mut c_void, out: *mut [usize; 2], id: i32) -> *mut [usize; 2] {
        black_box(this);
        let option = OPTION_LIST.lock().unwrap().iter().find(|(known, _)| *known == id).map(|(_, option)| *option);
        // SAFETY: the caller passes room for the 16-byte optional.
        unsafe { out.write([option.unwrap_or(0), usize::from(option.is_some())]) };
        out
    }

    // `Option* getEffectiveMaxFramerateOption()`: writes the `Option*` of option F6h, or F7h while slot 13 answers 2, to
    // `out`.
    #[inline(never)]
    extern "C" fn effective_max_framerate(this: *mut c_void, out: *mut usize) -> *mut usize {
        black_box(this);
        let wanted = if FRAMERATE_SLOT_13.load(Ordering::SeqCst) == 2 {
            Self::MAX_FRAMERATE_OTHER_ID
        } else {
            Self::MAX_FRAMERATE_ID
        };
        let option = OPTION_LIST.lock().unwrap().iter().find(|(id, _)| *id == wanted).map(|(_, o)| *o);
        // SAFETY: the caller passes room for one pointer.
        unsafe { out.write(option.unwrap_or(0)) };
        out
    }

    #[inline(never)]
    extern "C" fn view_distance(this: *mut c_void) -> i32 {
        black_box(this);
        VIEW_DISTANCE.load(Ordering::SeqCst)
    }

    // `void IntOption::set(int value, bool save)`: records the call and stores the value in the option.
    #[inline(never)]
    extern "C" fn int_option_set(this: *mut c_void, value: i32, save: u8) {
        INT_SETS.lock().unwrap().push((this as usize, value, save != 0));
        // SAFETY: `this` is a stand-in `Option` of `OPTION_SIZE` bytes.
        unsafe { ((this as usize + Self::INT_VALUE_AT) as *mut i32).write_unaligned(value) };
    }

    #[inline(never)]
    extern "C" fn game_update(this: *mut c_void) {
        black_box(this);
        black_box(GAME[1].load(Ordering::SeqCst));
    }

    #[inline(never)]
    extern "C" fn grab_mouse(this: *mut c_void) {
        black_box(this);
        Self::count(Self::GRAB);
    }

    #[inline(never)]
    extern "C" fn release_mouse(this: *mut c_void) {
        black_box(this);
        Self::count(Self::RELEASE);
    }

    #[inline(never)]
    extern "C" fn focus_gained(this: *mut c_void) {
        black_box(this);
        Self::count(Self::FOCUS_GAINED);
    }

    #[inline(never)]
    extern "C" fn focus_lost(this: *mut c_void) {
        black_box(this);
        Self::count(Self::FOCUS_LOST);
        FOCUS_LOST_THIS.lock().unwrap().push(this as usize);
    }
}
