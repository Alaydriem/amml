use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use amml::{ChatAdapter, ChatDetours, GuiMessageLayout, ModListener, UiFlags};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlChatField, AmmlEventContext, AmmlEventHeader, AmmlServices, AmmlStatus, AmmlStr,
    AmmlUiTable, ChatMessageAdded,
};
use amml_data::{EntryKind, Resolved};
use amml_msvc::MsvcString;
use amml_runtime::ModRegistry;

use crate::support::{EventsMod, Harness, NetworkGame};

const MODULE: &str = "@events/events";
const SIZE: usize = 0x118;
const KIND_AT: usize = 0;
const USERNAME_AT: usize = 0x70;
const FULL_AT: usize = 0x90;
const FILTERED_AT: usize = 0xB0;
const ENGAGED_AT: usize = 0xD0;
const XUID_AT: usize = 0xD8;
const WORDS: usize = SIZE / 8 + 2;

type PlainFn = extern "C" fn(usize, i32, usize, usize, usize, usize, usize, u64, u64, u64) -> usize;
type FlagWordFn = extern "C" fn(usize, i32, usize, usize, usize, usize, usize, u64, u64, u64, u64) -> usize;

// The stack slots the tests pass, with non-zero upper bytes in every slot: the game may leave such bytes, and the detours must
// pass each slot on bit for bit. A slot that was cut to its narrow type would lose them.
const FILTERED: usize = 0x1111_2222_3333_4444;
const TTS: usize = 0x5555_6666_7777_8888;
const AUTHOR: usize = 0x9999_AAAA_BBBB_CCCC;
const LIFETIME: u64 = 0xDEAD_BEEF_40A0_0000;
const FORCE: u64 = 0xAAAA_BBBB_CCCC_DD01;
const REQUIRED: u64 = 0x1234_5678_9ABC_DE00;
const FLAG_WORD: u64 = 0xFEDC_BA98_7654_3207;
const USERNAME_PTR: usize = 0x0A0B_0C0D_0E0F_1011;
const MESSAGE_PTR: usize = 0x2021_2223_2425_2627;

// The arguments each stand-in constructor received: `this` and the type are left out, the rest in order.
static ARGS: Mutex<Vec<Vec<u64>>> = Mutex::new(Vec::new());

// What the stand-in constructors build.
struct Spec {
    kind: u32,
    username: &'static str,
    full: String,
    filtered: Option<String>,
    xuid: &'static str,
}

static SPEC: Mutex<Option<Spec>> = Mutex::new(None);
// Which stand-in constructors ran: `plain` or `flag_word`.
static RAN: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

fn leak() -> usize {
    Box::leak(Box::new([0u64; WORDS])).as_mut_ptr().expose_provenance()
}

fn write<T>(base: usize, offset: usize, value: T) {
    // SAFETY: the stand-in memory is leaked, `WORDS` words, and every write is inside it.
    unsafe { std::ptr::with_exposed_provenance_mut::<T>(base + offset).write_unaligned(value) };
}

fn read<T: Copy>(base: usize, offset: usize) -> T {
    // SAFETY: as for `write`.
    unsafe { std::ptr::with_exposed_provenance::<T>(base + offset).read_unaligned() }
}

fn string(text: &str) -> MsvcString {
    // SAFETY: the heap is the stand-in `operator new` and `operator delete` of the harness, the same pair the loader uses.
    unsafe { MsvcString::new_in(&NetworkGame::heap(), text) }.unwrap()
}

// What a constructor of the game leaves in the line it builds.
fn fill(this: usize) {
    let spec = SPEC.lock().unwrap();
    let spec = spec.as_ref().unwrap();
    write(this, KIND_AT, spec.kind);
    write(this, USERNAME_AT, string(spec.username));
    write(this, FULL_AT, string(&spec.full));
    write(this, XUID_AT, string(spec.xuid));
    match &spec.filtered {
        Some(text) => {
            write(this, FILTERED_AT, string(text));
            write(this, ENGAGED_AT, 1u8);
        }
        None => {
            write(this, FILTERED_AT, string(""));
            write(this, ENGAGED_AT, 0u8);
        }
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
extern "C" fn plain_ctor(
    this: usize,
    kind: i32,
    username: usize,
    message: usize,
    filtered: usize,
    tts: usize,
    author: usize,
    lifetime: u64,
    force: u64,
    required: u64,
) -> usize {
    ARGS.lock().unwrap().push(vec![
        kind as u64,
        username as u64,
        message as u64,
        filtered as u64,
        tts as u64,
        author as u64,
        lifetime,
        force,
        required,
    ]);
    RAN.lock().unwrap().push("plain");
    fill(this);
    this
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
extern "C" fn flag_word_ctor(
    this: usize,
    kind: i32,
    username: usize,
    message: usize,
    filtered: usize,
    tts: usize,
    author: usize,
    lifetime: u64,
    force: u64,
    required: u64,
    flag_word: u64,
) -> usize {
    ARGS.lock().unwrap().push(vec![
        kind as u64,
        username as u64,
        message as u64,
        filtered as u64,
        tts as u64,
        author as u64,
        lifetime,
        force,
        required,
        flag_word,
    ]);
    RAN.lock().unwrap().push("flag_word");
    fill(this);
    this
}

fn function(address: usize) -> Resolved {
    Resolved { kind: EntryKind::Function, value: address as u64 - Harness::BASE, recipes_agreeing: 2 }
}

fn field(value: usize) -> Resolved {
    Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 }
}

// The entries of a build: `shape` is the value of `GuiMessage.ctorShape`; the 26.6x build has no flag word constructor,
// the 70.x build has both (the plain one still resolves there).
fn entries(shape: usize, size: Option<usize>) -> Vec<(&'static str, Resolved)> {
    let mut entries = vec![
        ("GuiMessage.ctor", function(plain_ctor as *const () as usize)),
        ("GuiMessage.ctorShape", field(shape)),
        ("GuiMessage.mType", field(KIND_AT)),
        ("GuiMessage.mUsername", field(USERNAME_AT)),
        ("GuiMessage.mFullString", field(FULL_AT)),
        ("GuiMessage.mFilteredFullString", field(FILTERED_AT)),
        ("GuiMessage.mFilteredFullStringEngaged", field(ENGAGED_AT)),
        ("GuiMessage.mAuthorXuid", field(XUID_AT)),
    ];
    if shape == 1 {
        entries.push(("GuiMessage.ctorWithFlagWord", function(flag_word_ctor as *const () as usize)));
    }
    if let Some(size) = size {
        entries.push(("GuiMessage.size", field(size)));
    }
    entries
}

fn start(shape: usize) -> Harness {
    start_with(&entries(shape, Some(if shape == 1 { 0x118 } else { 0x110 })))
}

fn start_with(entries: &[(&str, Resolved)]) -> Harness {
    let harness = Harness::start_with_entries(entries);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(amml::ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    ChatAdapter::install(harness.state);
    harness
}

fn spec(full: &str) {
    *SPEC.lock().unwrap() = Some(Spec {
        kind: 1,
        username: "Alex",
        full: full.to_string(),
        filtered: Some("filtered line, more than fifteen bytes".to_string()),
        xuid: "2535400000000001",
    });
    RAN.lock().unwrap().clear();
    ARGS.lock().unwrap().clear();
}

fn call_plain(this: usize) {
    let function: PlainFn = black_box(plain_ctor);
    let result = function(this, 1, USERNAME_PTR, MESSAGE_PTR, FILTERED, TTS, AUTHOR, LIFETIME, FORCE, REQUIRED);
    assert_eq!(result, this);
    let expected = [1, USERNAME_PTR as u64, MESSAGE_PTR as u64, FILTERED as u64, TTS as u64, AUTHOR as u64];
    let received = ARGS.lock().unwrap().last().cloned().unwrap();
    assert_eq!(received, [&expected[..], &[LIFETIME, FORCE, REQUIRED]].concat(), "{received:x?}");
}

fn call_flag_word(this: usize) {
    let function: FlagWordFn = black_box(flag_word_ctor);
    let result =
        function(this, 1, USERNAME_PTR, MESSAGE_PTR, FILTERED, TTS, AUTHOR, LIFETIME, FORCE, REQUIRED, FLAG_WORD);
    assert_eq!(result, this);
    let expected = [1, USERNAME_PTR as u64, MESSAGE_PTR as u64, FILTERED as u64, TTS as u64, AUTHOR as u64];
    let received = ARGS.lock().unwrap().last().cloned().unwrap();
    assert_eq!(received, [&expected[..], &[LIFETIME, FORCE, REQUIRED, FLAG_WORD]].concat(), "{received:x?}");
}

fn layout() -> GuiMessageLayout {
    GuiMessageLayout::new(SIZE, KIND_AT, USERNAME_AT, FULL_AT, FILTERED_AT, ENGAGED_AT, XUID_AT).unwrap()
}

// The text of `field` of the line at `message`, read through the layout.
fn text_in(message: usize, field: AmmlChatField) -> Result<String, AmmlStatus> {
    // SAFETY: the stand-in line is leaked memory laid out as the layout says.
    let (address, length) = unsafe { layout().text_of(message, field) }?;
    // SAFETY: `text_of` proved `length` bytes at `address` that are UTF-8.
    let bytes = unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(address), length) };
    Ok(String::from_utf8(bytes.to_vec()).unwrap())
}

fn table() -> AmmlUiTable {
    let services = amml::ServicesTable::build();
    // SAFETY: the loader's tables live for the process.
    unsafe { AmmlUiTable::read(AmmlServices::ui(services).unwrap()) }.unwrap()
}

fn api_table(harness: &Harness) -> AmmlUiTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    unsafe { AmmlUiTable::read(AmmlServices::ui(services).unwrap()) }.unwrap()
}

fn chat_text(table: &AmmlUiTable, field: AmmlChatField) -> Result<String, AmmlStatus> {
    let mut out = AmmlStr::EMPTY;
    // SAFETY: the table is live and `out` is writable.
    let status = unsafe { (table.chat_text.unwrap())(field, &mut out) };
    if status != AmmlStatus::OK {
        return Err(status);
    }
    // SAFETY: the loader's view is valid until the event ends, and the test reads it inside the event.
    Ok(unsafe { out.as_str() }.unwrap().to_string())
}

fn set_text(table: &AmmlUiTable, field: AmmlChatField, text: &str) -> AmmlStatus {
    // SAFETY: the table is live and the strings live across the call.
    unsafe { (table.chat_set_text.unwrap())(AmmlStr::from_str(MODULE), field, AmmlStr::from_str(text)) }
}

// What the listener does: 0 reads the four strings, 1 edits both texts and reads the full string again, 2 sends text over
// 4 KiB, 3 sends read only and unknown fields.
static ACTION: AtomicU8 = AtomicU8::new(0);
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_chat(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `ui.chat_message` with a `ChatMessageAdded`.
    let event = unsafe { &*event.cast::<ChatMessageAdded>() };
    let table = table();
    let mut seen = SEEN.lock().unwrap();
    seen.push(format!("kind {}", event.kind));
    match ACTION.load(Ordering::SeqCst) {
        1 => {
            seen.push(format!("set {:?}", set_text(&table, AmmlChatField::TEXT, "edited line")));
            seen.push(format!("set {:?}", set_text(&table, AmmlChatField::FILTERED_TEXT, "edited filtered")));
            seen.push(format!("again {:?}", chat_text(&table, AmmlChatField::TEXT)));
        }
        2 => {
            let big = "x".repeat(4097);
            let limit = "y".repeat(4096);
            seen.push(format!("over {:?}", set_text(&table, AmmlChatField::TEXT, &big)));
            seen.push(format!("limit {:?}", set_text(&table, AmmlChatField::FILTERED_TEXT, &limit)));
        }
        3 => {
            seen.push(format!("username {:?}", set_text(&table, AmmlChatField::USERNAME, "x")));
            seen.push(format!("xuid {:?}", set_text(&table, AmmlChatField::XUID, "x")));
            seen.push(format!("unknown {:?}", set_text(&table, AmmlChatField(9), "x")));
            seen.push(format!("read unknown {:?}", chat_text(&table, AmmlChatField(9))));
        }
        _ => {
            for field in
                [AmmlChatField::USERNAME, AmmlChatField::XUID, AmmlChatField::TEXT, AmmlChatField::FILTERED_TEXT]
            {
                seen.push(format!("{} {:?}", field.0, chat_text(&table, field)));
            }
        }
    }
    AmmlStatus::OK
}

fn listen(harness: &Harness) {
    SEEN.lock().unwrap().clear();
    let listener = ModListener { owner: 0, callback: on_chat, user: 0 };
    harness.state.events().listen("ui.chat_message", 1, AmmlBand::NORMAL, listener).unwrap();
}

#[test]
fn shape_selects_the_hook_target() {
    assert_eq!(ChatDetours::target_for(0), Some("GuiMessage.ctor"));
    assert_eq!(ChatDetours::target_for(1), Some("GuiMessage.ctorWithFlagWord"));
    assert_eq!(ChatDetours::target_for(2), None);
    assert_eq!(ChatDetours::hook_for(0).unwrap().entry, "GuiMessage.ctor");
    assert_eq!(ChatDetours::hook_for(1).unwrap().entry, "GuiMessage.ctorWithFlagWord");
    assert!(ChatDetours::hook_for(7).is_none());
    assert_eq!(ChatDetours::catalog_shape_for(0), Some("ctor"));
    assert_eq!(ChatDetours::catalog_shape_for(1), Some("ctor_flag_word"));
}

#[test]
fn a_26_6x_build_hooks_the_plain_constructor_and_raises_the_event() {
    let harness = start(0);
    listen(&harness);
    ACTION.store(0, Ordering::SeqCst);
    assert_eq!(UiFlags::load() & UiFlags::CHAT, UiFlags::CHAT);
    spec("<Alex> hello");
    let message = leak();

    call_plain(message);

    assert_eq!(*RAN.lock().unwrap(), ["plain"]);
    let seen = SEEN.lock().unwrap();
    assert_eq!(seen[0], "kind 1");
    assert_eq!(seen[1], "1 Ok(\"Alex\")");
    assert_eq!(seen[2], "2 Ok(\"2535400000000001\")");
    assert_eq!(seen[3], "3 Ok(\"<Alex> hello\")");
    assert_eq!(seen[4], "4 Ok(\"filtered line, more than fifteen bytes\")");
}

#[test]
fn a_70_x_build_hooks_the_flag_word_constructor_and_never_the_plain_one() {
    let harness = start(1);
    listen(&harness);
    ACTION.store(0, Ordering::SeqCst);
    spec("<Alex> hello");

    // `GuiMessage.ctor` still resolves on this build, with the other argument list: it must stay unhooked.
    call_plain(leak());
    assert!(SEEN.lock().unwrap().is_empty());

    call_flag_word(leak());
    assert_eq!(*RAN.lock().unwrap(), ["plain", "flag_word"]);
    assert_eq!(SEEN.lock().unwrap()[0], "kind 1");
    assert_eq!(SEEN.lock().unwrap().len(), 5);
}

#[test]
fn a_line_without_a_listener_is_left_alone_and_nothing_is_raised() {
    let _harness = start(0);
    spec("<Alex> hello");
    let message = leak();
    assert_eq!(UiFlags::load() & UiFlags::CHAT, 0);

    call_plain(message);

    assert_eq!(text_in(message, AmmlChatField::TEXT).unwrap(), "<Alex> hello");
    assert!(SEEN.lock().unwrap().is_empty());
}

#[test]
fn chat_read_outside_the_event_is_refused() {
    let harness = start(0);
    let table = api_table(&harness);
    let mut out = AmmlStr::EMPTY;

    // SAFETY: the table is live and `out` is writable.
    assert_eq!(unsafe { (table.chat_text.unwrap())(AmmlChatField::TEXT, &mut out) }, AmmlStatus::REFUSED);
    assert_eq!(set_text(&table, AmmlChatField::TEXT, "late"), AmmlStatus::REFUSED);
    // SAFETY: a null output is the case under test.
    let invalid = unsafe { (table.chat_text.unwrap())(AmmlChatField::TEXT, std::ptr::null_mut()) };
    assert_eq!(invalid, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn chat_calls_without_the_adapter_are_not_available() {
    let harness = Harness::start_with_entries(&entries(0, None));
    ChatAdapter::install(harness.state);
    let table = api_table(&harness);
    let mut out = AmmlStr::EMPTY;

    assert!(ChatAdapter::get().is_none());
    // SAFETY: the table is live and `out` is writable.
    assert_eq!(unsafe { (table.chat_text.unwrap())(AmmlChatField::TEXT, &mut out) }, AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn a_listener_reads_the_line_as_built_and_its_edit_is_written_after_the_listeners() {
    let harness = start(0);
    listen(&harness);
    ACTION.store(1, Ordering::SeqCst);
    spec("<Alex> hello");
    let message = leak();
    let (_, freed_before) = crate::support::HeapProbe::counts();

    call_plain(message);

    let seen = SEEN.lock().unwrap();
    assert_eq!(seen[1], format!("set {:?}", AmmlStatus::OK));
    assert_eq!(seen[2], format!("set {:?}", AmmlStatus::OK));
    assert_eq!(seen[3], "again Ok(\"<Alex> hello\")");
    assert_eq!(text_in(message, AmmlChatField::TEXT).unwrap(), "edited line");
    assert_eq!(text_in(message, AmmlChatField::FILTERED_TEXT).unwrap(), "edited filtered");
    assert_eq!(text_in(message, AmmlChatField::USERNAME).unwrap(), "Alex");
    assert_eq!(text_in(message, AmmlChatField::XUID).unwrap(), "2535400000000001");
    // The old block of the long filtered string went back to the game heap.
    let (_, freed) = crate::support::HeapProbe::counts();
    assert!(freed > freed_before, "{freed_before} {freed}");
}

#[test]
fn set_text_over_4_kib_is_refused() {
    let harness = start(0);
    listen(&harness);
    ACTION.store(2, Ordering::SeqCst);
    spec("<Alex> hello");
    let message = leak();

    call_plain(message);

    let seen = SEEN.lock().unwrap();
    assert_eq!(seen[1], format!("over {:?}", AmmlStatus::REFUSED));
    assert_eq!(seen[2], format!("limit {:?}", AmmlStatus::OK));
    // The refused text did not reach the game; the text of exactly 4096 bytes did.
    assert_eq!(text_in(message, AmmlChatField::TEXT).unwrap(), "<Alex> hello");
    assert_eq!(text_in(message, AmmlChatField::FILTERED_TEXT).unwrap(), "y".repeat(4096));
}

#[test]
fn read_only_and_unknown_fields_are_invalid_argument() {
    let harness = start(0);
    listen(&harness);
    ACTION.store(3, Ordering::SeqCst);
    spec("<Alex> hello");

    call_plain(leak());

    let seen = SEEN.lock().unwrap();
    let invalid = AmmlStatus::INVALID_ARGUMENT;
    assert_eq!(seen[1], format!("username {invalid:?}"));
    assert_eq!(seen[2], format!("xuid {invalid:?}"));
    assert_eq!(seen[3], format!("unknown {invalid:?}"));
    assert_eq!(seen[4], format!("read unknown Err({invalid:?})"));
}

#[test]
fn a_line_with_no_filtered_text_gives_not_available_and_takes_no_filtered_edit() {
    let harness = start(0);
    listen(&harness);
    ACTION.store(1, Ordering::SeqCst);
    *SPEC.lock().unwrap() = Some(Spec { kind: 2, username: "", full: "plain".to_string(), filtered: None, xuid: "" });
    RAN.lock().unwrap().clear();
    let message = leak();

    call_plain(message);

    let seen = SEEN.lock().unwrap();
    assert_eq!(seen[0], "kind 2");
    assert_eq!(seen[2], format!("set {:?}", AmmlStatus::NOT_AVAILABLE));
    assert_eq!(text_in(message, AmmlChatField::TEXT).unwrap(), "edited line");
    assert_eq!(text_in(message, AmmlChatField::FILTERED_TEXT), Err(AmmlStatus::NOT_AVAILABLE));
}

#[test]
fn non_utf8_text_reads_none() {
    let message = leak();
    let heap = NetworkGame::heap();
    // A short string holds its bytes in the object; a long one holds a heap block.
    let short = [0xFFu8, 0xFE, 0x41];
    let long: Vec<u8> = std::iter::repeat_n(0xC3u8, 40).collect();
    for (bytes, field, at) in [(&short[..], AmmlChatField::TEXT, FULL_AT), (&long[..], AmmlChatField::XUID, XUID_AT)] {
        // SAFETY: the heap is the stand-in `operator new`; the text is valid ASCII for `new_in` and is overwritten with the
        // bad bytes in place.
        let text = unsafe { MsvcString::new_in(&heap, &"a".repeat(bytes.len())) }.unwrap();
        write(message, at, text);
        let data = if bytes.len() > 15 { read::<usize>(message, at) } else { message + at };
        // SAFETY: `data` is the storage or the block of the string just written, `bytes.len()` bytes long.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                std::ptr::with_exposed_provenance_mut::<u8>(data),
                bytes.len(),
            )
        };

        assert_eq!(text_in(message, field), Err(AmmlStatus::INVALID_ARGUMENT), "{} bytes", bytes.len());
    }
}

#[test]
fn a_long_text_that_is_valid_reads_from_its_heap_block_and_a_corrupt_size_reads_none() {
    let message = leak();
    write(message, FULL_AT, string("a line that is longer than the sixteen byte buffer"));
    assert_eq!(text_in(message, AmmlChatField::TEXT).unwrap(), "a line that is longer than the sixteen byte buffer");

    // The size is larger than the capacity: not believed.
    write(message, FULL_AT + 16, 999_usize);
    assert_eq!(text_in(message, AmmlChatField::TEXT), Err(AmmlStatus::INVALID_ARGUMENT));
    // A capacity beyond the bound the loader believes.
    write(message, FULL_AT + 16, 4_usize);
    write(message, FULL_AT + 24, GuiMessageLayout::MAX_TEXT + 1);
    assert_eq!(text_in(message, AmmlChatField::TEXT), Err(AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn area_refused_without_gui_message_size() {
    let harness = start_with(&entries(0, None));

    assert!(ChatAdapter::get().is_none());
    assert!(!harness.state.events().is_available("ui.chat_message", 1));
    assert!(GuiMessageLayout::from_state(harness.state).is_none());
    assert_eq!(UiFlags::load() & UiFlags::CHAT, 0);
}

#[test]
fn area_refused_without_a_member_and_for_a_size_no_mirror_has() {
    let mut without_member = entries(0, Some(0x110));
    without_member.retain(|(name, _)| *name != "GuiMessage.mAuthorXuid");
    let harness = start_with(&without_member);
    assert!(ChatAdapter::get().is_none());
    assert!(!harness.state.events().is_available("ui.chat_message", 1));

    for size in [0x108, 0x120, 0] {
        assert!(
            GuiMessageLayout::new(size, KIND_AT, USERNAME_AT, FULL_AT, FILTERED_AT, ENGAGED_AT, XUID_AT).is_none(),
            "{size:#x}"
        );
    }
    for size in GuiMessageLayout::SIZES {
        assert!(
            GuiMessageLayout::new(size, KIND_AT, USERNAME_AT, FULL_AT, FILTERED_AT, ENGAGED_AT, XUID_AT).is_some(),
            "{size:#x}"
        );
    }
    // A member that overlaps another, one past the end and a string that is not 8-aligned.
    assert!(
        GuiMessageLayout::new(SIZE, KIND_AT, USERNAME_AT, USERNAME_AT + 8, FILTERED_AT, ENGAGED_AT, XUID_AT).is_none()
    );
    assert!(GuiMessageLayout::new(SIZE, KIND_AT, USERNAME_AT, FULL_AT, FILTERED_AT, ENGAGED_AT, SIZE - 8).is_none());
    assert!(GuiMessageLayout::new(SIZE, KIND_AT, USERNAME_AT + 4, FULL_AT, FILTERED_AT, ENGAGED_AT, XUID_AT).is_none());
}

#[test]
fn a_build_whose_catalog_shape_is_not_the_constructor_shape_hooks_nothing() {
    // The shape value says 70.x but the flag word constructor did not resolve: only `GuiMessage.ctor` is there. That
    // constructor has the wrong argument list on this build, so the event is not offered.
    let mut entries = entries(1, Some(0x118));
    entries.retain(|(name, _)| *name != "GuiMessage.ctorWithFlagWord");
    let harness = start_with(&entries);

    assert!(ChatAdapter::get().is_none());
    assert!(!harness.state.events().is_available("ui.chat_message", 1));
    spec("<Alex> hello");
    call_plain(leak());
    assert_eq!(*RAN.lock().unwrap(), ["plain"]);
}
