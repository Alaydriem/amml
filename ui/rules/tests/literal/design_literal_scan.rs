use std::path::Path;

use amml_ui_rules::literal::{AllowedLiteral, DesignLiteralScan, LiteralPattern};
use amml_ui_rules::source::SourceFiles;

use crate::support::SourceTree;

/// One planted line, scanned on its own.
struct Planted;

impl Planted {
    fn patterns(line: &str) -> Vec<LiteralPattern> {
        DesignLiteralScan::scan_text(Path::new("planted.rs"), line).into_iter().map(|hit| hit.pattern).collect()
    }
}

#[test]
fn a_colour_from_numbers_is_a_hit() {
    assert_eq!(Planted::patterns("let c = Color32::from_rgb(1, 2, 3);"), [LiteralPattern::ColourConstructor]);
    assert_eq!(Planted::patterns("ring(Color32::from_white_alpha(20))"), [LiteralPattern::ColourConstructor]);
    assert_eq!(Planted::patterns("let c = egui::Color32::from_hex(\"#fff\");"), [LiteralPattern::ColourConstructor]);
    assert_eq!(
        Planted::patterns("let c = Color32::from_rgba_unmultiplied( 0x10, 0, 0, 9);"),
        [LiteralPattern::ColourConstructor]
    );
}

#[test]
fn a_colour_from_values_is_not_a_hit() {
    assert!(Planted::patterns("let c = Color32::from_rgb(r, g, b);").is_empty());
    assert!(Planted::patterns("let c = Color32::from_gray(level);").is_empty());
}

#[test]
fn a_numeric_font_is_a_hit() {
    assert_eq!(Planted::patterns("let font = FontId::proportional(14.0);"), [LiteralPattern::FontSize]);
    assert!(Planted::patterns("let font = FontId::proportional(size);").is_empty());
}

#[test]
fn a_numeric_radius_is_a_hit() {
    assert_eq!(Planted::patterns("let r = CornerRadius::same(6);"), [LiteralPattern::CornerRadius]);
    assert!(Planted::patterns("let r = CornerRadius::same(radius);").is_empty());
}

#[test]
fn a_shadow_literal_is_a_hit() {
    assert_eq!(
        Planted::patterns("let s = Shadow { offset: [0, 1], ..Default::default() };"),
        [LiteralPattern::ShadowLiteral]
    );
    assert_eq!(Planted::patterns("let s = egui::epaint::Shadow {"), [LiteralPattern::ShadowLiteral]);
    assert!(Planted::patterns("let s = DropShadow { depth };").is_empty());
    assert!(Planted::patterns("let s = Elevation::Floating.shadow();").is_empty());
}

#[test]
fn a_size_vector_in_a_let_is_a_hit() {
    assert_eq!(Planted::patterns("let size = vec2(280.0, 40.0);"), [LiteralPattern::SizeVector]);
    assert_eq!(Planted::patterns("let gap = egui::vec2(-1.5, 2.);"), [LiteralPattern::SizeVector]);
    assert_eq!(Planted::patterns("ui.allocate_space(vec2(0.0, 0.0));"), [LiteralPattern::SizeVector]);
}

#[test]
fn a_size_vector_in_a_const_item_is_not_a_hit() {
    assert!(Planted::patterns("const SIZE: Vec2 = vec2(280.0, 40.0);").is_empty());
    assert!(Planted::patterns("    pub const GRIP: Vec2 = vec2(6.0, 18.0);").is_empty());
    assert!(Planted::patterns("    pub(crate) const STRIPE: egui::Vec2 = egui::vec2(2.0, 2.0);").is_empty());
}

#[test]
fn a_size_vector_in_a_const_fn_body_is_a_hit() {
    assert_eq!(Planted::patterns("    const fn size() -> Vec2 { vec2(4.0, 4.0) }"), [LiteralPattern::SizeVector]);
}

#[test]
fn a_vector_from_values_is_not_a_hit() {
    assert!(Planted::patterns("let size = vec2(width, 24.0);").is_empty());
    assert!(Planted::patterns("let size = vec2(width, height);").is_empty());
    assert!(Planted::patterns("let size = vec2(Self::WIDTH, 24.0);").is_empty());
}

#[test]
fn a_comment_is_not_a_hit() {
    assert!(Planted::patterns("// let c = Color32::from_rgb(1, 2, 3);").is_empty());
    assert!(Planted::patterns("    /// The old size was vec2(280.0, 40.0).").is_empty());
}

#[test]
fn a_hit_names_its_file_and_line() {
    let text = "fn a() {}\r\nfn b() {}\r\nlet c = Color32::from_rgb(1, 2, 3);\r\n";
    let hits = DesignLiteralScan::scan_text(Path::new("src/widget/chip.rs"), text);

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].line, 3);
    assert_eq!(hits[0].file, Path::new("src/widget/chip.rs"));
    assert_eq!(hits[0].text, "let c = Color32::from_rgb(1, 2, 3);");
    assert_eq!(hits[0].to_string(), "src/widget/chip.rs:3: colour constructor: let c = Color32::from_rgb(1, 2, 3);");
}

#[test]
fn two_patterns_on_one_line_are_two_hits() {
    let hits = Planted::patterns("paint(vec2(1.0, 2.0), Color32::from_rgb(1, 2, 3));");
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&LiteralPattern::SizeVector));
    assert!(hits.contains(&LiteralPattern::ColourConstructor));
}

#[test]
fn an_exempt_tokens_folder_is_not_scanned() {
    let tree = SourceTree::new();
    tree.file("tokens/color/accent.rs", "pub const VALUE: Color32 = Color32::from_rgb(1, 2, 3);\n");
    tree.file("tokens/rule/elevation.rs", "let s = Shadow { offset: [0, 1] };\n");

    let scan = DesignLiteralScan::new(SourceFiles::new(tree.root()).exempt_dir("tokens"));

    assert!(scan.scan().unwrap_or_else(|error| panic!("{error}")).is_empty());
}

#[test]
fn a_hit_in_a_mod_rs_is_found() {
    let tree = SourceTree::new();
    tree.file("widget/modal_frame/mod.rs", "fn a() {\n    let pad = vec2(0.0, 0.0);\n}\n");

    let hits = DesignLiteralScan::new(SourceFiles::new(tree.root())).scan().unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].line, 2);
    assert!(hits[0].file.ends_with("widget/modal_frame/mod.rs"));
}

#[test]
fn an_allowed_line_is_not_a_hit() {
    let tree = SourceTree::new();
    tree.file(
        "views/bar.rs",
        "let ring = Color32::from_white_alpha(20);\nlet other = Color32::from_white_alpha(40);\n",
    );
    let allowed = AllowedLiteral { file_suffix: "views/bar.rs", text: "from_white_alpha(20)", reason: "test" };

    let scan = DesignLiteralScan::new(SourceFiles::new(tree.root())).allow(allowed);
    let hits = scan.scan().unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].line, 2);
    assert!(scan.stale_allowances().unwrap_or_else(|error| panic!("{error}")).is_empty());
}

#[test]
fn an_allowance_that_matches_nothing_is_stale() {
    let tree = SourceTree::new();
    tree.file("views/bar.rs", "let ring = Color32::from_white_alpha(40);\n");
    let allowed = AllowedLiteral { file_suffix: "views/bar.rs", text: "from_white_alpha(20)", reason: "test" };

    let scan = DesignLiteralScan::new(SourceFiles::new(tree.root())).allow(allowed);
    let stale = scan.stale_allowances().unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].text, "from_white_alpha(20)");
}
