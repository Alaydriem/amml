mod bookmark_flag;
mod camera_segment;
mod checkpoint_tick;
mod edit_lock_pill;
mod name_tag;
mod path_map_cone;
mod playhead;
mod scene_camera_frustum;

use amml_i18n::text::{Text, UserText};

// a name or label as the user typed it
pub fn words(text: &str) -> Text {
    Text::user(UserText::from_model(text.to_owned()))
}
