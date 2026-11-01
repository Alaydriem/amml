use amml_derive::VtableReader;
use amml_derive::testing::ImageFixture;

const BASE: u64 = 0x1_4000_0000;

fn qword(rva: u32) -> [u8; 8] {
    (BASE + u64::from(rva)).to_le_bytes()
}

#[test]
fn vtable_stops_at_first_non_code_pointer() {
    let mut fx = ImageFixture::new(BASE);
    let first = fx.put_text(&[0xC3]);
    let second = fx.put_text(&[0xC3]);
    // the third slot points into the middle of a function
    let third = fx.put_text(&[0x90, 0xC3]) + 1;
    fx.put_function(first, first + 1);
    fx.put_function(second, second + 1);
    let data = fx.put_data(&[0; 8]);
    let mut table = Vec::new();
    for rva in [first, second, third, data, first] {
        table.extend(qword(rva));
    }
    let vtable = fx.put_rdata(&table);
    let image = fx.image();

    let rows = VtableReader::new(&image).slots(vtable, 0, 40);
    assert_eq!(rows, vec![(0, Some(first), true), (1, Some(second), true), (2, Some(third), false)]);

    let window = VtableReader::new(&image).slots(vtable, 1, 2);
    assert_eq!(window, vec![(1, Some(second), true)]);
}
