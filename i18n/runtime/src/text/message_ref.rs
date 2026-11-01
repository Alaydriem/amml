use crate::localize::{Localizer, MessageTable};
use crate::message::{Domain, Message};
use crate::pack::{LanguagePack, PackEntry};

/// A message with its type erased, so `Text` is not generic.
///
/// It keeps the English text for the fallback and the table builder of its enum.
#[derive(Debug, Clone, Copy)]
pub struct MessageRef {
    domain: Domain,
    index: usize,
    msgid: &'static str,
    plural: Option<&'static str>,
    build: fn(&LanguagePack) -> Vec<Option<PackEntry>>,
}

impl MessageRef {
    pub fn of<M: Message>(message: M) -> Self {
        Self {
            domain: M::DOMAIN,
            index: message.index(),
            msgid: message.msgid(),
            plural: message.plural(),
            build: MessageTable::<'static, M>::entries,
        }
    }

    pub fn msgid(&self) -> &'static str {
        self.msgid
    }

    pub fn text<'a>(&self, localizer: &'a Localizer) -> &'a str {
        localizer.translated(self.domain, self.index, self.build, None).unwrap_or(self.msgid)
    }

    pub fn plural<'a>(&self, localizer: &'a Localizer, count: u64) -> &'a str {
        localizer.translated(self.domain, self.index, self.build, Some(count)).unwrap_or(match self.plural {
            Some(plural) if count != 1 => plural,
            _ => self.msgid,
        })
    }
}
