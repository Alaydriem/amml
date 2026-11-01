use amml_studio_ui::id::StudioUiId;
use amml_ui::id::AmmlUiId;
use egui::Id;

/// An id enum from another crate, with a derived `Hash`.
#[derive(Debug, Clone, Copy, Hash)]
enum OtherId {
    First,
    Second,
}

#[test]
fn a_studio_ui_id_never_equals_the_same_numbered_variant_of_a_amml_ui_id() {
    assert_ne!(Id::unique(StudioUiId::InspectorTabs), Id::unique(AmmlUiId::Locale));
    assert_ne!(Id::unique(StudioUiId::InspectorDrawer), Id::unique(AmmlUiId::SegmentSlide));
}

#[test]
fn a_studio_ui_id_never_equals_the_same_numbered_variant_of_another_enum() {
    assert_ne!(Id::unique(StudioUiId::InspectorTabs), Id::unique(OtherId::First));
    assert_ne!(Id::unique(StudioUiId::InspectorDrawer), Id::unique(OtherId::Second));
}

#[test]
fn studio_ui_id_variants_keep_their_own_ids() {
    assert_ne!(Id::unique(StudioUiId::InspectorTabs), Id::unique(StudioUiId::InspectorDrawer));
}
