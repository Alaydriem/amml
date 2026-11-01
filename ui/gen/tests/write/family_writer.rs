use amml_ui_gen::write::FamilyWriter;

fn member(module: &str) -> (String, String) {
    (module.to_owned(), module.to_uppercase())
}

#[test]
fn numbered_modules_are_in_version_order() {
    let members = ["k10", "k2", "k1", "k_b", "ka", "scene_camera", "scene_10", "scene_9"].map(member);
    let out = FamilyWriter::render(&members);
    let order: Vec<&str> = out
        .lines()
        .filter_map(|line| line.strip_prefix("pub use "))
        .map(|line| line.split("::").next().unwrap_or(""))
        .collect();
    assert_eq!(order, vec!["k1", "k2", "k10", "k_b", "ka", "scene_9", "scene_10", "scene_camera"]);
}
