use crate::CatalogGroupView;
use amml_registry::NameRow;

// Splits a registry's names into the letter groups the browse list shows.
pub struct BrowseLayout;

impl BrowseLayout {
    // the group of a name that does not start with a letter
    pub const OTHER: &'static str = "#";

    // The rows that list `api`, in file order (the registry sorts them by lower-case id), cut into runs that share
    // the first letter after the `@`. `start` counts rows that list `api` only.
    pub fn build(rows: &[NameRow], api: u32) -> Vec<CatalogGroupView> {
        let mut groups: Vec<CatalogGroupView> = Vec::new();
        for (ordinal, row) in (0_u32..).zip(rows.iter().filter(|row| row.3.contains(&api))) {
            let letter = Self::letter(&row.0);
            match groups.last_mut() {
                Some(last) if last.id == letter => last.total += 1,
                _ => groups.push(CatalogGroupView { id: letter.clone(), label: letter, start: ordinal, total: 1 }),
            }
        }
        groups
    }

    // the upper-case first letter of the namespace, or `OTHER`
    fn letter(id: &str) -> String {
        match id.trim_start_matches('@').chars().next() {
            Some(first) if first.is_ascii_alphabetic() => first.to_ascii_uppercase().to_string(),
            _ => Self::OTHER.to_string(),
        }
    }
}
