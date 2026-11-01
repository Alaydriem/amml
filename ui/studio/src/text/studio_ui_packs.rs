use amml_i18n::service::PackSource;

/// The compiled language packs of `StudioUiMessage`, built into the binary. The studio adds this
/// source to its `LocalizationService` so the components' own words follow its language. Each file
/// in `i18n/packs/` is listed here; a test fails when one is missing.
pub struct StudioUiPacks;

impl StudioUiPacks {
    pub const SOURCE: PackSource = PackSource::Embedded(&[("en_XA", include_bytes!("../../i18n/packs/en_XA.json"))]);
}
