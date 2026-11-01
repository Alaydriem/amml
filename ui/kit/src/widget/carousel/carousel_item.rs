use amml_i18n::text::Text;

use super::CarouselMark;

/// One item of a `Carousel`: its mark, its name and its current value.
#[derive(Debug, Clone)]
pub struct CarouselItem {
    pub mark: CarouselMark,
    pub name: Text,
    pub value: Text,
}
