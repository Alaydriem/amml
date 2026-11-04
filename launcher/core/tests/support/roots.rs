pub struct Roots;

impl Roots {
    // launcher paths over a temp folder
    pub fn paths(root: &std::path::Path) -> amml_launcher_core::LauncherPaths {
        amml_launcher_core::LauncherPaths::new(amml_gdk::AmmlPaths::new(root.to_path_buf()))
    }
}
