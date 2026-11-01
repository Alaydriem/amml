use amml_ui::id::AmmlUiId;
use egui::Id;

/// An id enum from another crate, with a derived `Hash`.
#[derive(Debug, Clone, Copy, Hash)]
enum OtherId {
    First,
    Second,
}

#[test]
fn a_amml_ui_id_never_equals_the_same_numbered_variant_of_another_enum() {
    assert_ne!(Id::unique(AmmlUiId::Locale), Id::unique(OtherId::First));
    assert_ne!(Id::unique(AmmlUiId::SegmentSlide), Id::unique(OtherId::Second));
}

#[test]
fn amml_ui_id_variants_keep_their_own_ids() {
    assert_ne!(Id::unique(AmmlUiId::Locale), Id::unique(AmmlUiId::SegmentSlide));
}
