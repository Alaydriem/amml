use amml_studio_ui::component::library::ReplayPlaceholder;
use egui::Context;

#[test]
fn placeholder_decodes_to_16_by_9() {
    let decoder = png::Decoder::new(std::io::Cursor::new(ReplayPlaceholder::PNG));
    let reader = decoder.read_info().expect("the placeholder is a PNG");
    let info = reader.info();
    let ratio = info.width as f32 / info.height as f32;
    assert!((ratio - 16.0 / 9.0).abs() < 0.01, "ratio was {ratio}");

    let ctx = Context::default();
    assert_eq!(ReplayPlaceholder::texture(&ctx), ReplayPlaceholder::texture(&ctx));
}
