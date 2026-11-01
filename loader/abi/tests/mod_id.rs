use amml_abi::ModId;

#[test]
fn the_id_is_at_namespace_slash_name() {
    assert_eq!(ModId::format("alaydriem", "zoom"), "@alaydriem/zoom");
}

#[test]
fn a_segment_takes_no_separator_at_sign_or_leading_dot() {
    for bad in ["", "a/b", "a\\b", "@a", ".hidden", "a b"] {
        assert!(!ModId::is_valid_segment(bad), "{bad}");
    }
    for good in ["zoom", "mod-manager", "a_b.c", "Zoom2"] {
        assert!(ModId::is_valid_segment(good), "{good}");
    }
}

#[test]
fn amml_loader_is_reserved_in_any_case() {
    assert!(ModId::is_reserved("AMML", "Loader"));
    assert!(!ModId::is_reserved("alaydriem", "loader"));
}
