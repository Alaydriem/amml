mod drawn_icon;
mod icon_shape;
mod icon_size;

use egui::{Align2, Color32, Painter, Rect};

pub use drawn_icon::DrawnIcon;
pub use icon_shape::IconShape;
pub use icon_size::IconSize;

use crate::font::FontFace;

/// Every icon on the Paper Assets page (`Icon / NAME (CODE)`), in the order of the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Add,
    Copy,
    Hide,
    Show,
    Edit,
    Delete,
    Camera,
    Share,
    Search,
    Close,
    Chevron,
    Check,
    Pin,
    Info,
    Warning,
    Settings,
    Keyboard,
    Lock,
    Blocked,
    Apps,
    Rename,
    ArrowRight,
    ChevronUp,
    Minus,
    People,
    Fit,
    Mods,
    Play,
    Pause,
    Previous,
    Next,
    Rewind,
    FastForward,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Video,
    Stop,
    Flag,
    Grid,
    ChevronDown,
    Snap,
    Loop,
    Folder,
    List,
    Refresh,
    Back,
    Checklist,
    SkipBack,
    SkipForward,
    Clear,
    /// The grip on a grid placer's piece chip (Paper: Widgets, `Widget / Grid placer`).
    Grip,
    /// The unplace button on a filled grid placer cell (Paper: Widgets, `Widget / Grid placer`).
    Cancel,
    /// The Layer control of the HUD carousel (Paper: Stratum, `Stratum · 15 HUD · full controls`).
    Layers,
    /// The Rotate control of the HUD carousel (Paper: Stratum, `Stratum · 15 HUD · full controls`).
    Rotate,
    /// The Opacity control of the HUD carousel (Paper: Stratum, `Stratum · 15 HUD · full controls`).
    Brightness,
    /// The Show control of the HUD carousel (Paper: Stratum, `Stratum · 15 HUD · full controls`).
    View,
    /// The left arrow key cap of the HUD (Paper: Stratum, `Stratum · 16 HUD · quick mode`).
    ChevronLeft,
}

impl Icon {
    pub const ALL: [Icon; 59] = [
        Self::Add,
        Self::Copy,
        Self::Hide,
        Self::Show,
        Self::Edit,
        Self::Delete,
        Self::Camera,
        Self::Share,
        Self::Search,
        Self::Close,
        Self::Chevron,
        Self::Check,
        Self::Pin,
        Self::Info,
        Self::Warning,
        Self::Settings,
        Self::Keyboard,
        Self::Lock,
        Self::Blocked,
        Self::Apps,
        Self::Rename,
        Self::ArrowRight,
        Self::ChevronUp,
        Self::Minus,
        Self::People,
        Self::Fit,
        Self::Mods,
        Self::Play,
        Self::Pause,
        Self::Previous,
        Self::Next,
        Self::Rewind,
        Self::FastForward,
        Self::Undo,
        Self::Redo,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::Video,
        Self::Stop,
        Self::Flag,
        Self::Grid,
        Self::ChevronDown,
        Self::Snap,
        Self::Loop,
        Self::Folder,
        Self::List,
        Self::Refresh,
        Self::Back,
        Self::Checklist,
        Self::SkipBack,
        Self::SkipForward,
        Self::Clear,
        Self::Grip,
        Self::Cancel,
        Self::Layers,
        Self::Rotate,
        Self::Brightness,
        Self::View,
        Self::ChevronLeft,
    ];

    pub fn shape(self) -> IconShape {
        let code = match self {
            Self::Snap => return IconShape::Drawn(DrawnIcon::Snap),
            Self::Add => '\u{E710}',
            Self::Copy => '\u{E8C8}',
            Self::Hide => '\u{ED1A}',
            Self::Show => '\u{E7B3}',
            Self::Edit => '\u{E70F}',
            Self::Delete => '\u{E74D}',
            Self::Camera => '\u{E722}',
            Self::Share => '\u{E72D}',
            Self::Search => '\u{E721}',
            Self::Close => '\u{E8BB}',
            Self::Chevron => '\u{E76C}',
            Self::Check => '\u{E73E}',
            Self::Pin => '\u{E707}',
            Self::Info => '\u{E946}',
            Self::Warning => '\u{E7BA}',
            Self::Settings => '\u{E713}',
            Self::Keyboard => '\u{E765}',
            Self::Lock => '\u{E72E}',
            Self::Blocked => '\u{E733}',
            Self::Apps => '\u{ECAA}',
            Self::Rename => '\u{E8AC}',
            Self::ArrowRight => '\u{E72A}',
            Self::ChevronUp => '\u{E70E}',
            Self::Minus => '\u{E738}',
            Self::People => '\u{E716}',
            Self::Fit => '\u{E9A6}',
            Self::Mods => '\u{EA86}',
            Self::Play => '\u{E768}',
            Self::Pause => '\u{E769}',
            Self::Previous => '\u{E892}',
            Self::Next => '\u{E893}',
            Self::Rewind => '\u{EB9E}',
            Self::FastForward => '\u{EB9D}',
            Self::Undo => '\u{E7A7}',
            Self::Redo => '\u{E7A6}',
            Self::ZoomIn => '\u{E8A3}',
            Self::ZoomOut => '\u{E71F}',
            Self::Video => '\u{E714}',
            Self::Stop => '\u{E71A}',
            Self::Flag => '\u{E7C1}',
            Self::Grid => '\u{F0E2}',
            Self::ChevronDown => '\u{E70D}',
            Self::Loop => '\u{E8EE}',
            Self::Folder => '\u{E8B7}',
            Self::List => '\u{EA37}',
            Self::Refresh => '\u{E72C}',
            Self::Back => '\u{E72B}',
            Self::Checklist => '\u{E9D5}',
            Self::SkipBack => '\u{E100}',
            Self::SkipForward => '\u{E101}',
            Self::Clear => '\u{E894}',
            Self::Grip => '\u{E76F}',
            Self::Cancel => '\u{E711}',
            Self::Layers => '\u{E81E}',
            Self::Rotate => '\u{E7AD}',
            Self::Brightness => '\u{E793}',
            Self::View => '\u{E890}',
            Self::ChevronLeft => '\u{E76B}',
        };
        IconShape::Font(code)
    }

    /// Paints the icon at `size`, centred on `rect`. The rect only places the icon; it does not size it.
    pub fn paint(self, painter: &Painter, rect: Rect, size: IconSize, color: Color32) {
        match self.shape() {
            IconShape::Font(code) => {
                let font = FontFace::Icons.id(size.points());
                painter.text(rect.center(), Align2::CENTER_CENTER, code, font, color);
            }
            IconShape::Drawn(drawn) => drawn.paint(painter, rect, size, color),
        }
    }
}
