use amml_ui::brand::{AppIcon, AppIconSize};

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn header_number(bytes: &[u8], at: usize) -> u32 {
    let mut word = [0; 4];
    word.copy_from_slice(&bytes[at..at + 4]);
    u32::from_be_bytes(word)
}

#[test]
fn each_png_has_its_size() {
    for size in AppIconSize::ALL {
        let png = AppIcon::png(size);
        assert_eq!(png[..8], PNG_SIGNATURE, "{size:?} is not a PNG");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(header_number(png, 16), size.pixels(), "{size:?} width");
        assert_eq!(header_number(png, 20), size.pixels(), "{size:?} height");
    }
}

#[test]
fn sizes_are_distinct() {
    let mut pixels: Vec<u32> = AppIconSize::ALL.iter().map(|size| size.pixels()).collect();
    pixels.dedup();
    assert_eq!(pixels.len(), AppIconSize::ALL.len());
}
