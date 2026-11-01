use amml_i18n::text::{Text, UserText};

/// The gallery's sample words ("Render now", "Camera"). The gallery is a design check and is not
/// shipped, so its samples are marked as model data rather than given a message enum.
pub struct Sample;

impl Sample {
    pub fn text(words: &str) -> Text {
        Text::user(UserText::from_model(words.to_owned()))
    }
}
