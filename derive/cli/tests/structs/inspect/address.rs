use amml_derive::testing::ImageFixture;
use amml_derive::{Address, CliError};

const BASE: u64 = 0x1_4000_0000;

fn rva_of(text: &str) -> Result<u32, CliError> {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0x90; 0x200]);
    Address::parse(text)?.rva(&fx.image())
}

#[test]
fn address_accepts_rva_va_and_upper_case() {
    for text in ["0x10A0", "10a0", "0X10a0", "0x1400010A0", "1400010a0"] {
        assert_eq!(rva_of(text).unwrap(), 0x10a0, "{text}");
    }
}

#[test]
fn address_outside_sections_is_rejected() {
    assert!(matches!(rva_of("0xFFFFFFF"), Err(CliError::AddressUnmapped(0x0FFF_FFFF))));
}

#[test]
fn text_that_is_not_hex_is_rejected() {
    assert!(matches!(Address::parse("0xZZ"), Err(CliError::BadAddress(_))));
}
