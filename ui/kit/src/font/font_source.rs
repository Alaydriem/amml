// where the UI fonts come from: the embedded Nordwand Mono plus the system icon font, or only egui's built-in fonts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSource {
    Embedded,
    BuiltIn,
}
