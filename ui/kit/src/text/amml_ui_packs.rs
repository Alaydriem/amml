use amml_i18n::service::PackSource;

/// The compiled language packs of `AmmlUiMessage`, built into the binary. A mod adds this source
/// to its `LocalizationService` so the widgets' own words follow its language. Each file in
/// `i18n/packs/` is listed here; a test fails when one is missing.
pub struct AmmlUiPacks;

impl AmmlUiPacks {
    pub const SOURCE: PackSource = PackSource::Embedded(&[("en_XA", include_bytes!("../../i18n/packs/en_XA.json"))]);
}
