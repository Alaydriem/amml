/// A colour as a setting stores it: RGBA bytes, and whether the alpha is part of the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColourValue {
    pub rgba: [u8; 4],
    /// The value is `#RRGGBBAA`; otherwise `#RRGGBB` and fully opaque.
    pub alpha: bool,
}

impl ColourValue {
    /// Reads `#RRGGBB` or `#RRGGBBAA`, in either case. Anything else is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.trim().strip_prefix('#')?;
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |index: usize| u8::from_str_radix(digits.get(index * 2..index * 2 + 2)?, 16).ok();
        match digits.len() {
            6 => Some(Self { rgba: [byte(0)?, byte(1)?, byte(2)?, u8::MAX], alpha: false }),
            8 => Some(Self { rgba: [byte(0)?, byte(1)?, byte(2)?, byte(3)?], alpha: true }),
            _ => None,
        }
    }

    /// `#RRGGBB` or `#RRGGBBAA`, upper case.
    pub fn hex(self) -> String {
        let [r, g, b, a] = self.rgba;
        if self.alpha { format!("#{r:02X}{g:02X}{b:02X}{a:02X}") } else { format!("#{r:02X}{g:02X}{b:02X}") }
    }

    pub fn color32(self) -> egui::Color32 {
        let [r, g, b, a] = self.rgba;
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}
