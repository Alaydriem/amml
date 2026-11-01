use amml_i18n::text::Text;

use super::{CoverageFilter, CoverageGroup};

/// What the coverage panel shows: the line under its title, the status filter and the items by group. The search
/// text is the caller's, passed to `CoveragePanel::new`.
#[derive(Debug, Clone)]
pub struct CoveragePanelView {
    pub subtitle: Text,
    pub filter: CoverageFilter,
    pub groups: Vec<CoverageGroup>,
}
