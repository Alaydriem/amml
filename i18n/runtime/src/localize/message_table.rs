use std::marker::PhantomData;

use crate::message::Message;
use crate::pack::{ContextKey, LanguagePack, PackEntry};

use super::Localizer;

/// A typed view of one domain's translations. A lookup is one index into a table built
/// once per locale; it does not hash or allocate.
pub struct MessageTable<'a, M: Message> {
    localizer: &'a Localizer,
    message: PhantomData<M>,
}

impl<'a, M: Message> MessageTable<'a, M> {
    pub fn new(localizer: &'a Localizer) -> Self {
        Self { localizer, message: PhantomData }
    }

    /// The translation, or the English msgid when there is none.
    pub fn text(&self, message: M) -> &'a str {
        self.localizer.translated(M::DOMAIN, message.index(), Self::entries, None).unwrap_or(message.msgid())
    }

    /// The form for `count`. Untranslated messages use the English singular for 1 and the
    /// English plural for every other count.
    pub fn plural(&self, message: M, count: u64) -> &'a str {
        self.localizer
            .translated(M::DOMAIN, message.index(), Self::entries, Some(count))
            .unwrap_or_else(|| Self::english_plural(message, count))
    }

    pub fn english_plural(message: M, count: u64) -> &'static str {
        match message.plural() {
            Some(plural) if count != 1 => plural,
            _ => message.msgid(),
        }
    }

    /// Looks every message of `M` up in `pack` once, indexed by `Message::index`.
    pub fn entries(pack: &LanguagePack) -> Vec<Option<PackEntry>> {
        let mut entries = vec![None; M::ALL.len()];
        for message in M::ALL {
            let key = ContextKey::join(message.context(), message.msgid());
            if let Some(slot) = entries.get_mut(message.index()) {
                *slot = pack.m.get(&key).cloned();
            }
        }
        entries
    }
}
