use crate::app::LanguagePacks;
use amml_i18n::pack::LanguagePack;
use tauri::State;

// Tauri commands are free functions by framework rule. Each one only delegates.

#[tauri::command]
pub fn i18n_locales(packs: State<'_, LanguagePacks>) -> Vec<String> {
    packs.available()
}

#[tauri::command]
pub fn i18n_load(requested: String, packs: State<'_, LanguagePacks>) -> Option<LanguagePack> {
    packs.load_for(&requested)
}
