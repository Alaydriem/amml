use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use amml_abi::{ConnectionStarted, PacketWire};
use amml_api::testing::FakeGame;
use amml_api::{ConnectionStartedExt, HookOrder, PacketWireExt, WireConnection, WireDir, WireKind};

thread_local! {
    // The copies the listeners keep past their call; the fake calls them on the test thread.
    static KEPT_WIRE: RefCell<Option<PacketWire>> = const { RefCell::new(None) };
    static KEPT_STARTED: RefCell<Option<ConnectionStarted>> = const { RefCell::new(None) };
}

const REMOTE: WireConnection = WireConnection { id: 7, kind: WireKind::Remote, covered: true };

#[test]
fn fake_wire_bytes_reach_the_listener() {
    let fake = FakeGame::new();
    fake.offer::<PacketWire>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PacketWire>(HookOrder::NORMAL, move |event, ctx| {
            let bytes = event.bytes().map(<[u8]>::to_vec);
            record.lock().unwrap().push((event.dir(), event.connection(), bytes, ctx.thread));
        })
        .unwrap();

    let out = fake.raise_packet_wire(WireDir::Out, REMOTE, &[0xc1, 0x01, 0x00, 0x00, 0x08, 0xb1]);
    fake.raise_packet_wire(WireDir::In, REMOTE, &[0x8f, 0x01, 0x02]);

    assert!(!out.panicked);
    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            (WireDir::Out, REMOTE, Some(vec![0xc1, 0x01, 0x00, 0x00, 0x08, 0xb1]), amml_abi::AmmlThread::GAME),
            (WireDir::In, REMOTE, Some(vec![0x8f, 0x01, 0x02]), amml_abi::AmmlThread::NETWORK),
        ]
    );
}

#[test]
fn copy_of_the_event_has_no_bytes() {
    let fake = FakeGame::new();
    fake.offer::<PacketWire>();
    let inside = Arc::new(Mutex::new(Vec::new()));
    let record = inside.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<PacketWire>(HookOrder::NORMAL, move |event, _| {
            let copy = *event;
            // A copy made inside the call has no bytes either: only the event the listener got lends them.
            record.lock().unwrap().push((event.bytes().is_some(), copy.bytes().is_some()));
            KEPT_WIRE.with(|kept| *kept.borrow_mut() = Some(copy));
        })
        .unwrap();

    fake.raise_packet_wire(WireDir::Out, REMOTE, &[1, 2, 3]);

    assert_eq!(*inside.lock().unwrap(), vec![(true, false)]);
    let copy = KEPT_WIRE.with(|kept| kept.borrow_mut().take()).unwrap();
    assert_eq!(copy.bytes(), None);
    assert_eq!(copy.connection(), REMOTE);
}

#[test]
fn connection_started_gives_the_protocol_and_the_host_in_the_call() {
    let fake = FakeGame::new();
    fake.offer::<ConnectionStarted>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ConnectionStarted>(HookOrder::NORMAL, move |event, _| {
            let host = event.host().map(str::to_string);
            record.lock().unwrap().push((event.connection(), event.protocol(), event.port(), host));
            KEPT_STARTED.with(|kept| *kept.borrow_mut() = Some(*event));
        })
        .unwrap();
    let local = WireConnection { id: 8, kind: WireKind::Local, covered: false };

    fake.raise_connection_started(REMOTE, 2225, "play.example.net", 19132);
    fake.raise_connection_started(local, 2225, "", 0);

    assert_eq!(
        *seen.lock().unwrap(),
        vec![(REMOTE, 2225, 19132, Some("play.example.net".to_string())), (local, 2225, 0, None)]
    );
    assert_eq!(KEPT_STARTED.with(|kept| kept.borrow_mut().take()).unwrap().host(), None);
}
