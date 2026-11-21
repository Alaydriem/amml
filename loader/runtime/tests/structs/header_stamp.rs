use amml_abi::{AmmlEventHeader, AmmlOrigin, AmmlPhase};
use amml_runtime::HeaderStamp;

fn loaded() -> AmmlEventHeader {
    AmmlEventHeader {
        size: 24,
        version: 1,
        phase: AmmlPhase::BEFORE,
        origin: AmmlOrigin::GAME,
        cancelled: 0,
        injected: 1,
        _reserved: 0,
    }
}

#[test]
fn a_header_with_an_origin_changes_only_the_origin() {
    let stamp = HeaderStamp::new(loaded(), false);

    let header = stamp.with_origin(AmmlOrigin::OTHER_MOD);

    assert_eq!(header.origin, AmmlOrigin::OTHER_MOD);
    assert_eq!(
        (header.size, header.version, header.phase, header.injected, header.cancelled),
        (24, 1, AmmlPhase::BEFORE, 1, 0)
    );
}

#[test]
fn a_header_with_an_origin_keeps_a_cancel() {
    let mut stamp = HeaderStamp::new(loaded(), true);
    let mut header = loaded();
    header.cancelled = 1;
    stamp.after_call(&mut header);

    assert_eq!(stamp.with_origin(AmmlOrigin::THIS_MOD).cancelled, 1);
}
