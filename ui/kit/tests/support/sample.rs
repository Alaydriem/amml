use amml_i18n::text::{Text, UserText};

// sample words for widget tests, marked as model data
pub struct Sample;

impl Sample {
    pub fn text(words: &str) -> Text {
        Text::user(UserText::from_model(words.to_owned()))
    }
}
