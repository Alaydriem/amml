use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml::{BlockHandles, ModListener, RequestAdapter, RequestPort, UiFlags};
use amml_abi::{
    AmmlBand, AmmlBlockHandle, AmmlEventContext, AmmlEventHeader, AmmlInventoryMove, AmmlRequestOptions,
    AmmlRequestOutcome, AmmlStatus, InventoryRequestDone,
};
use amml_data::{EntryKind, Resolved};
use amml_runtime::{CreativeEntry, RequestMove, RequestSlot, RequestStep};

use crate::support::{FakeVtable, Harness};

// The three handlers the loader hooks, as the game's vtable holds them.
static HANDLED: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static ENDED: Mutex<Vec<(u64, u8, u16, u16)>> = Mutex::new(Vec::new());

#[inline(never)]
extern "C" fn open_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    black_box((this, source, packet));
    HANDLED.lock().unwrap().push(black_box("open"));
}

#[inline(never)]
extern "C" fn close_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    black_box((this, source, packet));
    HANDLED.lock().unwrap().push(black_box("close"));
}

#[inline(never)]
extern "C" fn response_stand_in(this: *mut c_void, source: *mut c_void, packet: *mut c_void) {
    black_box((this, source, packet));
    HANDLED.lock().unwrap().push(black_box("response"));
}

#[inline(never)]
extern "C" fn function_stand_in() {
    black_box(0);
}

static HANDLERS: FakeVtable<3> =
    FakeVtable([open_stand_in as *const (), close_stand_in as *const (), response_stand_in as *const ()]);
static GLOBAL: [u64; 2] = [0; 2];

fn entries() -> Vec<(&'static str, Resolved)> {
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
    let stand_in = function_stand_in as *const () as usize;
    vec![
        ("MinecraftPackets.createPacket", function(stand_in)),
        ("ClientInstance.getPacketSender", slot(3)),
        ("PacketSender.send", slot(2)),
        ("Packet.read", slot(8)),
        ("ReadOnlyBinaryStream.vtable", global(std::ptr::from_ref(&GLOBAL) as usize)),
        ("ReadOnlyBinaryStream.dtor", function(stand_in)),
        ("Actor.getRuntimeID", function(stand_in)),
        ("FillingContainer.swapSlots", slot(5)),
        ("ItemStack.mNetIdVariant", field(0x80)),
        ("ClientInstance.getLocalPlayer", slot(0)),
        ("Player.playerInventory", field(0x10)),
        ("PlayerInventory.mInventory", field(0x20)),
        ("FillingContainer.mItems", field(0x30)),
        ("LegacyClientNetworkHandler.vtable", global(std::ptr::from_ref(&HANDLERS) as usize)),
        ("ClientNetworkHandler.handle.ContainerOpenPacket", slot(0)),
        ("ClientNetworkHandler.handle.ContainerClosePacket", slot(1)),
        ("ClientNetworkHandler.handle.ItemStackResponsePacket", slot(2)),
    ]
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    RequestAdapter::install(harness.state);
    harness.state.events().offer("inventory.request_done", 1);
    HANDLED.lock().unwrap().clear();
    ENDED.lock().unwrap().clear();
    harness
}

unsafe extern "C" fn on_done(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `inventory.request_done` with an `InventoryRequestDone`.
    let event = unsafe { &*event.cast::<InventoryRequestDone>() };
    ENDED.lock().unwrap().push((event.id, event.outcome, event.accepted, event.total));
    AmmlStatus::OK
}

fn listen(harness: &Harness) {
    let listener = ModListener { owner: 0, callback: on_done, user: 0 };
    harness.state.events().listen("inventory.request_done", 1, AmmlBand::NORMAL, listener).unwrap();
}

// A port over 36 slots of 8 items, net ids 100 + slot; `during_send` runs inside every `SendRequests`, as an answer of
// a local server that comes back before the game's send returns.
struct Port {
    screen: bool,
    ran: Vec<RequestStep>,
    during_send: Option<Box<dyn FnMut()>>,
    // The game takes no `SendRequests` packet.
    refuse_requests: bool,
    // The local player is in creative mode, and the creative entry the stand-in registry has for any block.
    creative: bool,
    entry: Option<CreativeEntry>,
    // A slot the stand-in game holds no stack in.
    empty_slot: Option<u8>,
}

impl Port {
    fn new() -> Self {
        Self {
            screen: false,
            ran: Vec::new(),
            during_send: None,
            refuse_requests: false,
            creative: true,
            entry: Some(CreativeEntry { creative_net_id: 300, count: 64 }),
            empty_slot: None,
        }
    }
}

impl RequestPort for Port {
    fn screen_open(&mut self, _strict: bool) -> bool {
        self.screen
    }

    fn read_slots(&mut self) -> Option<Vec<Option<RequestSlot>>> {
        let empty = self.empty_slot;
        Some(
            (0..36)
                .map(|slot| {
                    (empty != Some(slot as u8)).then_some(RequestSlot { net_id: 100 + slot, count: 8, max: 64, key: 7 })
                })
                .collect(),
        )
    }

    fn is_creative(&mut self) -> bool {
        self.creative
    }

    fn creative_entry(&mut self, _block: usize) -> Option<CreativeEntry> {
        self.entry
    }

    fn run(&mut self, step: &RequestStep) -> bool {
        self.ran.push(step.clone());
        if self.refuse_requests && matches!(step, RequestStep::SendRequests(_)) {
            return false;
        }
        if matches!(step, RequestStep::SendRequests(_))
            && let Some(during) = self.during_send.as_mut()
        {
            during();
        }
        true
    }
}

fn swap() -> [RequestMove; 1] {
    [RequestMove::Swap { a: 0, b: 1 }]
}

fn options(hidden_open: bool) -> AmmlRequestOptions {
    AmmlRequestOptions { hidden_open: u8::from(hidden_open), predict: 0, _reserved: [0; 2] }
}

// A `ContainerOpenPacket` / `ContainerClosePacket` of the stand-in game: the id at +30h, the type at +31h.
fn container_packet(size: usize, id: i8, kind: i8) -> usize {
    let memory = Box::leak(vec![0u8; size].into_boxed_slice());
    memory[0x30] = id as u8;
    memory[0x31] = kind as u8;
    memory.as_ptr().expose_provenance()
}

// An `ItemStackResponsePacket` with one answer for each (request id, result): the vector at +30h, 0x30 bytes each.
fn response_packet(answers: &[(i32, u8)]) -> usize {
    let elements: &'static mut [u8] = Box::leak(vec![0u8; answers.len() * 0x30].into_boxed_slice());
    for (index, (id, result)) in answers.iter().enumerate() {
        elements[index * 0x30] = *result;
        elements[index * 0x30 + 0x10..index * 0x30 + 0x14].copy_from_slice(&id.to_le_bytes());
    }
    let begin = elements.as_ptr().expose_provenance();
    let packet: &'static mut [u8] = Box::leak(vec![0u8; 0x50].into_boxed_slice());
    packet[0x30..0x38].copy_from_slice(&begin.to_le_bytes());
    packet[0x38..0x40].copy_from_slice(&(begin + elements.len()).to_le_bytes());
    packet.as_ptr().expose_provenance()
}

// The hooked game handlers, as the game calls them through its vtable.
fn handle(slot: usize, packet: usize) {
    // SAFETY: the vtable holds the three stand-in handlers, which have the signature of the game's handler.
    let function = unsafe {
        std::mem::transmute::<*const (), extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>(HANDLERS.0[slot])
    };
    let function = black_box(function);
    function(std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::with_exposed_provenance_mut(packet));
}

const OPEN: usize = 0;
const CLOSE: usize = 1;
const RESPONSE: usize = 2;
const INVENTORY_TYPE: i8 = -1;

fn adapter() -> &'static RequestAdapter {
    RequestAdapter::get().unwrap()
}

#[test]
fn the_hidden_inventory_open_and_its_close_never_reach_the_game() {
    let harness = start();
    let adapter = adapter();
    let mut port = Port::new();
    adapter.request_with(&mut port, 1, &swap(), options(true)).unwrap();
    adapter.tick_with(&mut port, 0);
    assert_eq!(port.ran, vec![RequestStep::SendInteract]);

    // The server's open of the inventory is ours; a chest open is the game's.
    handle(OPEN, container_packet(0x50, 5, INVENTORY_TYPE));
    assert!(HANDLED.lock().unwrap().is_empty());
    handle(OPEN, container_packet(0x50, 6, 7));
    assert_eq!(*HANDLED.lock().unwrap(), ["open"]);

    adapter.tick_with(&mut port, 0);
    let wire = match &port.ran[1] {
        RequestStep::SendRequests(bytes) => bytes.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(wire[0], 1);
    // The server answers (accepted); the game gets the answer too.
    handle(RESPONSE, response_packet(&[(-100_001, 0)]));
    assert_eq!(*HANDLED.lock().unwrap(), ["open", "response"]);
    listen(&harness);
    adapter.tick_with(&mut port, 0);
    assert_eq!(port.ran.last(), Some(&RequestStep::SendClose(5)));
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::DONE.0, 1, 1)]);

    // The answer to our close is ours once; another id is the game's.
    handle(CLOSE, container_packet(0x38, 9, 0));
    assert_eq!(HANDLED.lock().unwrap().last(), Some(&"close"));
    handle(CLOSE, container_packet(0x38, 5, 0));
    assert_eq!(HANDLED.lock().unwrap().len(), 3);
    handle(CLOSE, container_packet(0x38, 5, 0));
    assert_eq!(HANDLED.lock().unwrap().len(), 4);
}

#[test]
fn with_no_request_every_packet_goes_to_the_game_and_the_flag_is_clear() {
    let _harness = start();

    assert_eq!(UiFlags::load() & UiFlags::REQUEST, 0);
    handle(OPEN, container_packet(0x50, 5, INVENTORY_TYPE));
    handle(CLOSE, container_packet(0x38, 5, 0));
    handle(RESPONSE, response_packet(&[(-100_001, 0)]));

    assert_eq!(*HANDLED.lock().unwrap(), ["open", "close", "response"]);
    assert!(adapter().book().is_idle());
}

#[test]
fn answer_inside_send_is_held_to_the_next_tick() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    // The answer comes inside the send, through the game's own handler, while the adapter holds no lock.
    port.during_send = Some(Box::new(|| handle(RESPONSE, response_packet(&[(-100_001, 0)]))));
    adapter.request_with(&mut port, 1, &swap(), options(false)).unwrap();

    adapter.tick_with(&mut port, 0);

    // The request is still on its way: no event yet, and the game saw the answer.
    assert!(ENDED.lock().unwrap().is_empty());
    assert_eq!(*HANDLED.lock().unwrap(), ["response"]);
    assert_eq!(adapter.request_with(&mut port, 2, &swap(), options(false)), Err(AmmlStatus::REFUSED));
    adapter.tick_with(&mut port, 0);
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::DONE.0, 1, 1)]);
    assert!(adapter.request_with(&mut port, 2, &swap(), options(false)).is_ok());
}

#[test]
fn a_second_request_while_one_runs_is_refused_and_a_hidden_open_needs_the_hud() {
    let _harness = start();
    let adapter = adapter();
    let mut port = Port::new();
    port.screen = true;

    assert_eq!(adapter.request_with(&mut port, 1, &swap(), options(true)), Err(AmmlStatus::REFUSED));
    assert!(adapter.request_with(&mut port, 1, &swap(), options(false)).is_ok());
    assert_eq!(adapter.request_with(&mut port, 2, &swap(), options(false)), Err(AmmlStatus::REFUSED));
    adapter.disable_owner(1);
    assert_eq!(adapter.request_with(&mut port, 2, &[], options(false)), Err(AmmlStatus::INVALID_ARGUMENT));
    let far_slot = RequestMove::Pick { slot: 9, creative_net_id: 1, count: 1 };
    assert_eq!(adapter.request_with(&mut port, 2, &[far_slot], options(false)), Err(AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn disable_ends_the_request_with_no_event() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    adapter.request_with(&mut port, 1, &swap(), options(false)).unwrap();
    adapter.tick_with(&mut port, 0);

    adapter.disable_owner(1);
    adapter.tick_with(&mut port, 0);

    assert!(ENDED.lock().unwrap().is_empty());
    assert_eq!(UiFlags::load() & UiFlags::REQUEST, 0);
}

#[test]
fn a_call_before_the_game_runs_is_not_ready() {
    let _harness = start();
    let moves = [AmmlInventoryMove { kind: AmmlInventoryMove::SWAP, a: 0, b: 1, count: 0 }];

    let result = adapter().request_moves(1, std::ptr::null(), &moves, options(false));

    assert_eq!(result, Err(AmmlStatus::NOT_READY));
}

#[test]
fn a_world_leave_ends_a_running_request_cancelled_with_an_event_and_a_disable_with_none() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    adapter.request_with(&mut port, 1, &swap(), options(false)).unwrap();
    adapter.tick_with(&mut port, 0);

    adapter.world_left();

    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::CANCELLED.0, 0, 1)]);
    assert_eq!(UiFlags::load() & UiFlags::REQUEST, 0);
    // The next request ends with a disable instead: no event.
    adapter.request_with(&mut port, 1, &swap(), options(false)).unwrap();
    adapter.tick_with(&mut port, 0);
    adapter.disable_owner(1);
    adapter.tick_with(&mut port, 0);
    assert_eq!(ENDED.lock().unwrap().len(), 1);
}

#[test]
fn a_packet_the_game_does_not_take_ends_not_sent_and_predicts_nothing() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    port.refuse_requests = true;
    let predicting = AmmlRequestOptions { hidden_open: 1, predict: 1, _reserved: [0; 2] };
    adapter.request_with(&mut port, 1, &swap(), predicting).unwrap();
    adapter.tick_with(&mut port, 0);
    handle(OPEN, container_packet(0x50, 5, INVENTORY_TYPE));

    adapter.tick_with(&mut port, 0);

    // The requests were refused: no prediction ran, the hidden inventory is closed, the end is NOT_SENT.
    assert_eq!(port.ran.len(), 3, "{:?}", port.ran);
    assert!(matches!(port.ran[1], RequestStep::SendRequests(_)));
    assert_eq!(port.ran[2], RequestStep::SendClose(5));
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::NOT_SENT.0, 0, 1)]);
}

// A block handle the loader holds for the mod 1 in the current world.
fn block_handle(harness: &Harness) -> AmmlBlockHandle {
    let epoch = harness.state.events().gate().epoch();
    AmmlBlockHandle(harness.state.resources().insert(BlockHandles::BLOCK, 1, epoch, 0x7000).unwrap().0)
}

// An `ItemStackResponsePacket` with one answer that names `slot` of the hotbar container with `net_id`: the element's
// containers vector at +18h, a container (name at +0, slots vector at +10h) and a slot info (slot at +1, net id at +4).
fn pick_response_packet(wire: i32, result: u8, slot: u8, net_id: i32) -> usize {
    let info: &'static mut [u8] = Box::leak(vec![0u8; 0x58].into_boxed_slice());
    info[1] = slot;
    info[4..8].copy_from_slice(&net_id.to_le_bytes());
    let info_begin = info.as_ptr().expose_provenance();
    let container: &'static mut [u8] = Box::leak(vec![0u8; 0x28].into_boxed_slice());
    container[0] = 28;
    container[0x10..0x18].copy_from_slice(&info_begin.to_le_bytes());
    container[0x18..0x20].copy_from_slice(&(info_begin + 0x58).to_le_bytes());
    let container_begin = container.as_ptr().expose_provenance();
    let element: &'static mut [u8] = Box::leak(vec![0u8; 0x30].into_boxed_slice());
    element[0] = result;
    element[0x10..0x14].copy_from_slice(&wire.to_le_bytes());
    element[0x18..0x20].copy_from_slice(&container_begin.to_le_bytes());
    element[0x20..0x28].copy_from_slice(&(container_begin + 0x28).to_le_bytes());
    let begin = element.as_ptr().expose_provenance();
    let packet: &'static mut [u8] = Box::leak(vec![0u8; 0x50].into_boxed_slice());
    packet[0x30..0x38].copy_from_slice(&begin.to_le_bytes());
    packet[0x38..0x40].copy_from_slice(&(begin + 0x30).to_le_bytes());
    packet.as_ptr().expose_provenance()
}

#[test]
fn pick_outside_creative_is_refused() {
    let harness = start();
    let adapter = adapter();
    let mut port = Port::new();
    port.creative = false;

    let result = adapter.pick_with(&mut port, 1, block_handle(&harness), 4);

    assert_eq!(result, Err(AmmlStatus::REFUSED));
    // The refusal took nothing: the next request runs.
    assert!(adapter.request_with(&mut port, 1, &swap(), options(false)).is_ok());
}

#[test]
fn pick_with_no_creative_entry_is_refused() {
    let harness = start();
    let adapter = adapter();
    let mut port = Port::new();
    port.entry = None;

    let result = adapter.pick_with(&mut port, 1, block_handle(&harness), 4);

    assert_eq!(result, Err(AmmlStatus::REFUSED));
    assert!(adapter.book().is_idle());
}

#[test]
fn pick_above_the_hotbar_is_refused() {
    let harness = start();
    let mut port = Port::new();

    let result = adapter().pick_with(&mut port, 1, block_handle(&harness), 9);

    assert_eq!(result, Err(AmmlStatus::REFUSED));
}

#[test]
fn stale_block_is_stale() {
    let harness = start();
    let adapter = adapter();
    let mut port = Port::new();
    let handle = block_handle(&harness);
    harness.state.resources().take_owner(1);

    assert_eq!(adapter.pick_with(&mut port, 1, handle, 4), Err(AmmlStatus::STALE));
    assert_eq!(adapter.pick_with(&mut port, 1, AmmlBlockHandle(0xDEAD), 4), Err(AmmlStatus::STALE));
    assert!(adapter.book().is_idle());
}

#[test]
fn pick_prediction_is_undone_on_refused() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    port.empty_slot = Some(4);
    adapter.pick_with(&mut port, 1, block_handle(&harness), 4).unwrap();

    adapter.tick_with(&mut port, 0);
    assert_eq!(port.ran[1], RequestStep::PredictPick { slot: 4, creative_net_id: 300, count: 64 });
    handle(RESPONSE, response_packet(&[(-100_001, 1)]));

    adapter.tick_with(&mut port, 0);

    assert_eq!(port.ran.last(), Some(&RequestStep::UnpredictPick { slot: 4 }));
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::REFUSED.0, 0, 1)]);
}

#[test]
fn an_accepted_pick_takes_the_net_id_of_the_response() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    let mut port = Port::new();
    port.empty_slot = Some(4);
    adapter.pick_with(&mut port, 1, block_handle(&harness), 4).unwrap();
    adapter.tick_with(&mut port, 0);

    handle(RESPONSE, pick_response_packet(-100_001, 0, 4, 77));
    adapter.tick_with(&mut port, 0);

    assert_eq!(port.ran.last(), Some(&RequestStep::AdoptNetId { slot: 4, net_id: 77 }));
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::DONE.0, 1, 1)]);
}

#[test]
fn an_occupied_slot_pick_is_sent_and_a_refusal_gives_the_held_stack_back() {
    let harness = start();
    listen(&harness);
    let adapter = adapter();
    // Every slot of the stand-in game holds a stack, so hotbar slot 4 is occupied.
    let mut port = Port::new();
    adapter.pick_with(&mut port, 1, block_handle(&harness), 4).unwrap();

    adapter.tick_with(&mut port, 0);

    // The request goes out at once with the destroy of the held stack, and the prediction replaces it.
    let RequestStep::SendRequests(bytes) = &port.ran[0] else { panic!("{:?}", port.ran) };
    assert_eq!(bytes[4], 3, "{bytes:?}");
    assert_eq!(port.ran[1], RequestStep::PredictPick { slot: 4, creative_net_id: 300, count: 64 });
    handle(RESPONSE, response_packet(&[(-100_001, 1)]));

    adapter.tick_with(&mut port, 0);

    // The refusal gives the replaced stack back: the port that predicted it restores it at this step.
    assert_eq!(port.ran.last(), Some(&RequestStep::UnpredictPick { slot: 4 }));
    assert_eq!(*ENDED.lock().unwrap(), [(1, AmmlRequestOutcome::REFUSED.0, 0, 1)]);
}
