use amml::HandleStamp;

#[test]
fn a_sealed_address_opens_with_the_same_counter() {
    let handle = HandleStamp::seal(0x7FF6_1234_5678, 7).unwrap();

    assert_eq!(HandleStamp::open(handle, 7), Some(0x7FF6_1234_5678));
    assert_ne!(handle, 0x7FF6_1234_5678);
}

#[test]
fn another_counter_or_a_null_address_does_not_open() {
    let handle = HandleStamp::seal(0x1000, 7).unwrap();

    assert_eq!(HandleStamp::open(handle, 8), None);
    assert_eq!(HandleStamp::open(0x1000, 7), None);
    assert_eq!(HandleStamp::open(0, 0), None);
    assert_eq!(HandleStamp::open(HandleStamp::seal(0, 7).unwrap(), 7), None);
}

#[test]
fn only_the_low_16_bits_of_the_counter_count() {
    let handle = HandleStamp::seal(0x1000, 3).unwrap();

    assert_eq!(HandleStamp::open(handle, 3 + 0x1_0000), Some(0x1000));
}

#[test]
fn an_address_over_48_bits_is_not_sealed() {
    assert_eq!(HandleStamp::seal(1 << 48, 1), None);
}
