/// A value that comes from the model, not from a message: a scene name, a file name, a
/// player name. It is shown as written and never translated.
///
/// It is built only from an owned `String`, so a string literal cannot be passed off as
/// user data:
///
/// ```compile_fail
/// let _: amml_i18n::text::UserText = "x".into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserText(String);

impl UserText {
    pub fn from_model(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
