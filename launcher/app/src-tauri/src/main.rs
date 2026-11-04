// Keeps a console window from opening behind the app in a release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    amml_launcher_app_lib::LauncherApp::run();
}
