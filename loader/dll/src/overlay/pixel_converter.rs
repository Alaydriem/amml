use super::CapturedImage;

/// Turns the pixels of a captured image into the formats the loader hands out: tightly packed 8-bit RGB rows for a
/// PNG file, and BGRA8 rows for a capture stream frame, scaled down by a whole factor. The picture is opaque: alpha is
/// dropped for the PNG and written as 255 for BGRA.
///
/// The back buffer formats read are `R8G8B8A8_UNORM` (and its sRGB twin), `B8G8R8A8_UNORM` (and its sRGB twin) and
/// `R10G10B10A2_UNORM`, each 4 bytes per pixel; any other format (an HDR float buffer, for example) is not read.
pub struct PixelConverter;

impl PixelConverter {
    pub const DXGI_R10G10B10A2_UNORM: u32 = 24;
    pub const DXGI_R8G8B8A8_UNORM: u32 = 28;
    pub const DXGI_R8G8B8A8_UNORM_SRGB: u32 = 29;
    pub const DXGI_B8G8R8A8_UNORM: u32 = 87;
    pub const DXGI_B8G8R8A8_UNORM_SRGB: u32 = 91;
    pub const BYTES_PER_PIXEL: usize = 4;
    const TEN_BITS: u32 = 0x3FF;

    /// True when a back buffer of `format` can be read.
    pub fn supports(format: u32) -> bool {
        matches!(
            format,
            Self::DXGI_R10G10B10A2_UNORM
                | Self::DXGI_R8G8B8A8_UNORM
                | Self::DXGI_R8G8B8A8_UNORM_SRGB
                | Self::DXGI_B8G8R8A8_UNORM
                | Self::DXGI_B8G8R8A8_UNORM_SRGB
        )
    }

    /// The RGB bytes of `image`, `width * 3` per row. `None` for a format that is not read, an empty image or too few
    /// pixel bytes.
    pub fn to_rgb(image: &CapturedImage) -> Option<Vec<u8>> {
        let pixels = Self::checked(image)?;
        let mut out = Vec::with_capacity(pixels.len() / Self::BYTES_PER_PIXEL * 3);
        let (rows, _) = pixels.as_chunks::<{ Self::BYTES_PER_PIXEL }>();
        for pixel in rows {
            out.extend_from_slice(&Self::rgb(image.format, *pixel));
        }
        Some(out)
    }

    /// The smallest whole factor that brings `width` x `height` within `max_width` x `max_height` (a 0 limit is no
    /// limit). Each side is divided by it and rounded down.
    pub fn scale_factor(width: u32, height: u32, max_width: u32, max_height: u32) -> u32 {
        let fits = |factor: u32| {
            (max_width == 0 || width / factor <= max_width) && (max_height == 0 || height / factor <= max_height)
        };
        let largest = width.max(height).max(1);
        (1..=largest).find(|factor| fits(*factor)).unwrap_or(largest)
    }

    /// Writes `image` into `out` as BGRA8 rows of `width / factor` pixels (no padding), each pixel the average of a
    /// `factor` x `factor` block, and gives the size written. `None` for a format that is not read, too few pixel
    /// bytes, or a side that `factor` brings to 0.
    pub fn to_bgra(image: &CapturedImage, factor: u32, out: &mut Vec<u8>) -> Option<(u32, u32)> {
        let pixels = Self::checked(image)?;
        let factor = factor.max(1);
        let (width, height) = (image.width / factor, image.height / factor);
        if width == 0 || height == 0 {
            return None;
        }
        let source_row = image.width as usize * Self::BYTES_PER_PIXEL;
        let block = factor as usize;
        let count = u32::try_from(block * block).ok()?;
        out.clear();
        out.reserve(width as usize * height as usize * Self::BYTES_PER_PIXEL);
        for y in 0..height as usize {
            for x in 0..width as usize {
                let mut sum = [0u32; 3];
                for dy in 0..block {
                    let row = (y * block + dy) * source_row;
                    for dx in 0..block {
                        let at = row + (x * block + dx) * Self::BYTES_PER_PIXEL;
                        let pixel = [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]];
                        let [r, g, b] = Self::rgb(image.format, pixel);
                        sum[0] += u32::from(r);
                        sum[1] += u32::from(g);
                        sum[2] += u32::from(b);
                    }
                }
                let average = |total: u32| u8::try_from((total + count / 2) / count).unwrap_or(u8::MAX);
                out.extend_from_slice(&[average(sum[2]), average(sum[1]), average(sum[0]), u8::MAX]);
            }
        }
        Some((width, height))
    }

    // The pixel bytes of a readable image, exactly `width * height * 4` of them.
    fn checked(image: &CapturedImage) -> Option<&[u8]> {
        if !Self::supports(image.format) || image.width == 0 || image.height == 0 {
            return None;
        }
        let len = (image.width as usize).checked_mul(image.height as usize)?.checked_mul(Self::BYTES_PER_PIXEL)?;
        image.pixels.get(..len)
    }

    fn rgb(format: u32, [a, b, c, d]: [u8; 4]) -> [u8; 3] {
        match format {
            Self::DXGI_B8G8R8A8_UNORM | Self::DXGI_B8G8R8A8_UNORM_SRGB => [c, b, a],
            Self::DXGI_R10G10B10A2_UNORM => {
                let word = u32::from_le_bytes([a, b, c, d]);
                [Self::scale(word), Self::scale(word >> 10), Self::scale(word >> 20)]
            }
            _ => [a, b, c],
        }
    }

    // A 10-bit channel scaled to 8 bits, rounded to the nearest value.
    fn scale(word: u32) -> u8 {
        let value = word & Self::TEN_BITS;
        u8::try_from((value * 255 + Self::TEN_BITS / 2) / Self::TEN_BITS).unwrap_or(u8::MAX)
    }
}
