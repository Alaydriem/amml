use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use amml_i18n::message::Message;

use super::CatalogProblem;
use crate::error::ToolError;
use crate::pot::PotWriter;

/// The checks that every message enum must pass (`message_catalog`), and the check that its `.pot` file is current
/// (`pot_up_to_date`). Each UI crate runs both in its tests.
pub struct CatalogCheck;

impl CatalogCheck {
    /// Every problem in `M`, in the order of `M::ALL`. An empty list means the catalog is sound.
    pub fn problems<M: Message>() -> Vec<CatalogProblem> {
        let mut problems = Vec::new();
        let mut keys = HashSet::new();
        for (position, message) in M::ALL.iter().enumerate() {
            let msgid = message.msgid();
            if msgid.trim().is_empty() {
                problems.push(CatalogProblem::EmptyMsgid { index: position });
            }
            if !keys.insert((message.context(), msgid)) {
                problems.push(CatalogProblem::RepeatedKey { index: position, msgid });
            }
            if message.plural().is_some_and(|plural| Self::placeholders(msgid) != Self::placeholders(plural)) {
                problems.push(CatalogProblem::PlaceholderMismatch { index: position, msgid });
            }
            if message.index() != position {
                problems.push(CatalogProblem::IndexOutOfPlace { position, index: message.index() });
            }
        }
        problems
    }

    /// True when `<i18n_dir>/<domain>.pot` holds what `PotWriter` writes for `M` now. A missing file is not current.
    pub fn pot_is_current<M: Message>(i18n_dir: &Path) -> Result<bool, ToolError> {
        let path = i18n_dir.join(format!("{}.pot", M::DOMAIN.name()));
        Ok(!PotWriter::write::<M>(&path, true)?)
    }

    fn placeholders(text: &'static str) -> BTreeSet<&'static str> {
        text.split('{').skip(1).filter_map(|part| part.split_once('}').map(|(name, _)| name)).collect()
    }
}
