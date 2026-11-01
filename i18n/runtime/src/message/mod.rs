pub mod domain;

pub use domain::Domain;

/// A translatable message. Each crate that shows text has one enum that implements this.
///
/// The msgid is the English text, so a message with no translation shows English.
pub trait Message: Copy + 'static {
    /// The catalog (.pot file and packs) this message belongs to.
    const DOMAIN: Domain;
    /// Every variant, in the order the .pot file lists them.
    const ALL: &'static [Self];

    /// A dense index, below `ALL.len()`, used to look the message up at draw time.
    fn index(self) -> usize;
    /// The English source text.
    fn msgid(self) -> &'static str;
    /// The gettext context, for two messages with the same English text.
    fn context(self) -> Option<&'static str>;
    /// The English plural text, for a message that changes with a count.
    fn plural(self) -> Option<&'static str>;
    /// A note for translators.
    fn note(self) -> Option<&'static str>;
}
