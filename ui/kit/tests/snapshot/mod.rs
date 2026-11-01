mod sheet_snapshot;

use amml_ui::gallery::GallerySheet;
use sheet_snapshot::SheetSnapshot;

#[test]
fn snapshot_text_button() {
    SheetSnapshot::check(GallerySheet::TextButton);
}

#[test]
fn snapshot_icon_button() {
    SheetSnapshot::check(GallerySheet::IconButton);
}

#[test]
fn snapshot_toggle() {
    SheetSnapshot::check(GallerySheet::Toggle);
}

#[test]
fn snapshot_slider() {
    SheetSnapshot::check(GallerySheet::Slider);
}

#[test]
fn snapshot_checkbox() {
    SheetSnapshot::check(GallerySheet::Checkbox);
}

#[test]
fn snapshot_chip() {
    SheetSnapshot::check(GallerySheet::Chip);
}

#[test]
fn snapshot_status_pill() {
    SheetSnapshot::check(GallerySheet::StatusPill);
}

#[test]
fn snapshot_segmented() {
    SheetSnapshot::check(GallerySheet::Segmented);
}

#[test]
fn snapshot_dropdown() {
    SheetSnapshot::check(GallerySheet::Dropdown);
}

#[test]
fn snapshot_menu_item() {
    SheetSnapshot::check(GallerySheet::MenuItem);
}

#[test]
fn snapshot_text_field() {
    SheetSnapshot::check(GallerySheet::TextField);
}

#[test]
fn snapshot_drag_field() {
    SheetSnapshot::check(GallerySheet::DragField);
}

#[test]
fn snapshot_colour_field() {
    SheetSnapshot::check(GallerySheet::ColourField);
}

#[test]
fn snapshot_tab_row() {
    SheetSnapshot::check(GallerySheet::TabRow);
}

#[test]
fn snapshot_card_frame() {
    SheetSnapshot::check(GallerySheet::CardFrame);
}

#[test]
fn snapshot_modal_frame() {
    SheetSnapshot::check(GallerySheet::ModalFrame);
}

#[test]
fn snapshot_section_header() {
    SheetSnapshot::check(GallerySheet::SectionHeader);
}

#[test]
fn snapshot_empty_state() {
    SheetSnapshot::check(GallerySheet::EmptyState);
}

#[test]
fn snapshot_loading_state() {
    SheetSnapshot::check(GallerySheet::LoadingState);
}

#[test]
fn snapshot_page_header() {
    SheetSnapshot::check(GallerySheet::PageHeader);
}

#[test]
fn snapshot_avatar() {
    SheetSnapshot::check(GallerySheet::Avatar);
}

#[test]
fn snapshot_banner() {
    SheetSnapshot::check(GallerySheet::Banner);
}

#[test]
fn snapshot_toast() {
    SheetSnapshot::check(GallerySheet::Toast);
}

#[test]
fn snapshot_list_row() {
    SheetSnapshot::check(GallerySheet::ListRow);
}

#[test]
fn snapshot_nav_entry() {
    SheetSnapshot::check(GallerySheet::NavEntry);
}

#[test]
fn snapshot_section_card() {
    SheetSnapshot::check(GallerySheet::SectionCard);
}

#[test]
fn snapshot_table() {
    SheetSnapshot::check(GallerySheet::Table);
}

#[test]
fn snapshot_splitter_and_scroll() {
    SheetSnapshot::check(GallerySheet::SplitterAndScroll);
}

#[test]
fn snapshot_corner_button() {
    SheetSnapshot::check(GallerySheet::CornerButton);
}

#[test]
fn snapshot_key_glyphs() {
    SheetSnapshot::check(GallerySheet::KeyGlyphs);
}

#[test]
fn snapshot_confirm_popover() {
    SheetSnapshot::check(GallerySheet::ConfirmPopover);
}

#[test]
fn snapshot_value_row() {
    SheetSnapshot::check(GallerySheet::ValueRow);
}

#[test]
fn snapshot_swatch_row() {
    SheetSnapshot::check(GallerySheet::SwatchRow);
}

#[test]
fn snapshot_legend_card() {
    SheetSnapshot::check(GallerySheet::LegendCard);
}

#[test]
fn snapshot_toast_line() {
    SheetSnapshot::check(GallerySheet::ToastLine);
}

#[test]
fn snapshot_empty_state_card() {
    SheetSnapshot::check(GallerySheet::EmptyStateCard);
}

#[test]
fn snapshot_text_field_swatch() {
    SheetSnapshot::check(GallerySheet::TextFieldSwatch);
}

#[test]
fn snapshot_segmented_grid() {
    SheetSnapshot::check(GallerySheet::SegmentedGrid);
}

#[test]
fn snapshot_menu_item_radio() {
    SheetSnapshot::check(GallerySheet::MenuItemRadio);
}

#[test]
fn snapshot_grid_placer() {
    SheetSnapshot::check(GallerySheet::GridPlacer);
}

#[test]
fn snapshot_caption() {
    SheetSnapshot::check(GallerySheet::Caption);
}

#[test]
fn snapshot_carousel() {
    SheetSnapshot::check(GallerySheet::Carousel);
}
