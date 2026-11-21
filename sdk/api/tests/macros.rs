use amml_abi::{AmmlStr, ClientTick, GameTick};
use amml_api::{Area, Uses};

const USES: Uses = amml_api::amml_uses! {
    feature "core" { required ClientTick; optional GameTick; }
    feature "extra" { optional ClientTick; }
};

fn text(view: AmmlStr) -> &'static str {
    // SAFETY: the rows point to static strings.
    unsafe { view.as_str() }.unwrap()
}

#[test]
fn the_uses_rows_name_feature_and_requirement() {
    let rows = USES.rows();
    assert_eq!(rows.len(), 3);
    let first = (text(rows[0].feature), rows[0].kind, text(rows[0].name), rows[0].version, rows[0].required);
    assert_eq!(first, ("core", 0, "client.tick", 1, 1));
    assert_eq!((text(rows[1].name), rows[1].required), ("client.game_tick", 0));
    assert_eq!((text(rows[2].feature), text(rows[2].name), rows[2].required), ("extra", "client.tick", 0));
}

#[test]
fn area_rows_name_kind_area() {
    const AREAS: Uses = amml_api::amml_uses! {
        feature "input" { required area Area::KEY_INJECT; optional ClientTick; optional area Area::TURN; }
    };
    let rows = AREAS.rows();

    let keys = (text(rows[0].feature), rows[0].kind, text(rows[0].name), rows[0].version, rows[0].required);
    assert_eq!(keys, ("input", 1, "input.key_inject", 1, 1));
    assert_eq!((rows[1].kind, text(rows[1].name)), (0, "client.tick"));
    assert_eq!((rows[2].kind, text(rows[2].name), rows[2].required), (1, "input.turn", 0));
}
