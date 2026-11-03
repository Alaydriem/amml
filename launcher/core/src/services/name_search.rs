use crate::NameHit;
use amml_registry::NamesFile;

pub struct NameSearch;

impl NameSearch {
    const ID_SCORE: u32 = 300;
    const TITLE_SCORE: u32 = 200;
    const TAG_SCORE: u32 = 100;

    // Case-insensitive. An id that holds the query scores highest, then a title, then a tag.
    pub fn search(names: &NamesFile, query: &str) -> Vec<NameHit> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<NameHit> = names
            .rows
            .iter()
            .filter_map(|row| {
                let score = if row.0.to_lowercase().contains(&needle) {
                    Self::ID_SCORE
                } else if row.1.to_lowercase().contains(&needle) {
                    Self::TITLE_SCORE
                } else if row.2.iter().any(|tag| tag.to_lowercase().contains(&needle)) {
                    Self::TAG_SCORE
                } else {
                    return None;
                };
                Some(NameHit {
                    id: row.0.clone(),
                    title: row.1.clone(),
                    api: row.3.first().copied().unwrap_or(0),
                    page: row.4,
                    score,
                })
            })
            .collect();
        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
        hits
    }
}
