use amml_i18n::text::Text;
use egui::Color32;

use crate::icon::Icon;

/// The mark of a `CarouselItem`: an icon, or a letter in the colour of its tone.
#[derive(Debug, Clone)]
pub enum CarouselMark {
    Icon(Icon),
    /// The letter, and the tone colour the caller reads from a colour token.
    Letter(Text, Color32),
}
