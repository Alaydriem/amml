use amml_runtime::StackIdentity;

fn stone(aux: i16, count: u8, user_data: &[u8]) -> StackIdentity<'_> {
    StackIdentity { type_name: "minecraft:stone", aux, count, user_data }
}

#[test]
fn identity_is_equal_for_mergeable_stacks() {
    assert_eq!(stone(0, 1, &[]).value(), stone(0, 1, &[]).value());
    assert_eq!(stone(0, 1, &[]).value(), 0xbb81_b29a_8f70_34b9);
}

#[test]
fn identity_differs_by_aux_and_user_data() {
    let plain = stone(0, 1, &[]).value();

    assert_ne!(plain, stone(1, 1, &[]).value());
    assert_ne!(plain, stone(0, 1, &[0x0a, 0x00]).value());
    assert_eq!(stone(5, 1, &[0x0a, 0x00]).value(), 0xa198_7cea_e3e8_91ee);
    assert_ne!(stone(0, 1, &[0x0a]).value(), stone(0, 1, &[0x0b]).value());
}

#[test]
fn identity_ignores_count() {
    assert_eq!(stone(0, 1, &[]).value(), stone(0, 64, &[]).value());
}
