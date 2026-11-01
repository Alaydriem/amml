use amml_studio_ui::rule::LabelTint;
use amml_studio_ui::tokens::color as studio;
use amml_ui::tokens::{ColorToken, color};

#[test]
fn label_tint_sits_between_the_hue_and_the_text_colour() {
    let text = <color::Text as ColorToken>::VALUE;
    for hue in [
        <studio::TrackCamera as ColorToken>::VALUE,
        <studio::TrackFov as ColorToken>::VALUE,
        <studio::Scene1 as ColorToken>::VALUE,
        <studio::StatusMissing as ColorToken>::VALUE,
    ] {
        let tint = LabelTint::of(hue);
        for (channel, (h, t)) in tint.to_array().iter().zip(hue.to_array().iter().zip(text.to_array())).take(3) {
            assert!((*h.min(&t)..=*h.max(&t)).contains(channel), "{tint:?} is not between {hue:?} and {text:?}");
        }
        assert_ne!(tint, hue);
        assert_ne!(tint, text);
    }
}
