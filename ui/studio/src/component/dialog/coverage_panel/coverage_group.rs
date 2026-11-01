use amml_i18n::text::Text;

use super::CoverageRow;

/// A group of coverage items under its head (as in "A. Library and open replay").
#[derive(Debug, Clone)]
pub struct CoverageGroup {
    pub title: Text,
    pub rows: Vec<CoverageRow>,
}
