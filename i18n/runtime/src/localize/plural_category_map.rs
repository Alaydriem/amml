use icu_plurals::PluralCategory;

/// Maps a CLDR plural category to the form index a pack stores it at.
///
/// The indices are worked out once from the pack's `plural` list, so a draw-time lookup
/// compares no strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluralCategoryMap {
    indices: [usize; 6],
}

impl PluralCategoryMap {
    const ORDER: [PluralCategory; 6] = [
        PluralCategory::Zero,
        PluralCategory::One,
        PluralCategory::Two,
        PluralCategory::Few,
        PluralCategory::Many,
        PluralCategory::Other,
    ];

    // A category absent from `names` falls to the last form, as `LanguagePack::plural_index` does.
    pub fn new(names: &[String]) -> Self {
        let last = names.len().saturating_sub(1);
        let indices =
            Self::ORDER.map(|category| names.iter().position(|name| name == Self::name(category)).unwrap_or(last));
        Self { indices }
    }

    pub fn index(&self, category: PluralCategory) -> usize {
        let slot = Self::ORDER.iter().position(|known| *known == category).unwrap_or(Self::ORDER.len() - 1);
        self.indices[slot]
    }

    // Spelled out rather than derived from Debug, because these names are the contract the
    // pack compiler wrote into `plural` and a formatting change upstream would break the
    // match silently.
    pub fn name(category: PluralCategory) -> &'static str {
        match category {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}
