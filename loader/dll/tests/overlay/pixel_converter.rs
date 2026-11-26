use amml::{CapturedImage, PixelConverter};

fn image(width: u32, height: u32, format: u32, pixel: impl Fn(u32, u32) -> [u8; 4]) -> CapturedImage {
    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&pixel(x, y));
        }
    }
    CapturedImage { width, height, format, pixels }
}

#[test]
fn the_factor_is_the_smallest_whole_one_that_fits() {
    assert_eq!(PixelConverter::scale_factor(640, 360, 320, 200), 2);
    assert_eq!(PixelConverter::scale_factor(640, 360, 0, 0), 1);
    assert_eq!(PixelConverter::scale_factor(640, 360, 640, 360), 1);
    assert_eq!(PixelConverter::scale_factor(1920, 1080, 500, 0), 4);
    assert_eq!(PixelConverter::scale_factor(1920, 1080, 0, 100), 11);
}

#[test]
fn a_scaled_frame_averages_each_block_into_opaque_bgra() {
    // RGBA: a 2 x 2 block of red 0 and 200, green 100, blue 50, alpha 0.
    let pixel = |x: u32, _| if x.is_multiple_of(2) { [0, 100, 50, 0] } else { [200, 100, 50, 0] };
    let source = image(4, 2, PixelConverter::DXGI_R8G8B8A8_UNORM, pixel);
    let mut out = Vec::new();

    let size = PixelConverter::to_bgra(&source, 2, &mut out);

    assert_eq!(size, Some((2, 1)));
    assert_eq!(out, vec![50, 100, 100, 255, 50, 100, 100, 255]);
}

#[test]
fn bgra_and_ten_bit_buffers_read_as_rgb() {
    let bgra = image(1, 1, PixelConverter::DXGI_B8G8R8A8_UNORM_SRGB, |_, _| [1, 2, 3, 4]);
    // Red 1023, green 0, blue 512.
    let word: u32 = 1023 | (512 << 20);
    let ten = image(1, 1, PixelConverter::DXGI_R10G10B10A2_UNORM, |_, _| word.to_le_bytes());

    assert_eq!(PixelConverter::to_rgb(&bgra), Some(vec![3, 2, 1]));
    assert_eq!(PixelConverter::to_rgb(&ten), Some(vec![255, 0, 128]));
}

#[test]
fn an_unread_format_or_short_buffer_gives_nothing() {
    let float = image(1, 1, 10, |_, _| [0; 4]);
    let mut short = image(2, 2, PixelConverter::DXGI_R8G8B8A8_UNORM, |_, _| [0; 4]);
    short.pixels.truncate(12);
    let mut out = Vec::new();

    assert!(!PixelConverter::supports(10));
    assert_eq!(PixelConverter::to_rgb(&float), None);
    assert_eq!(PixelConverter::to_bgra(&short, 1, &mut out), None);
    let tiny = image(1, 1, PixelConverter::DXGI_R8G8B8A8_UNORM, |_, _| [0; 4]);
    assert_eq!(PixelConverter::to_bgra(&tiny, 2, &mut out), None);
}
