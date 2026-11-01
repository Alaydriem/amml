use std::sync::Arc;

use amml_i18n::localize::Localizer;
use egui::{Context, Rect, Theme as EguiTheme, Visuals};

use crate::draw::{Corner, Draw};
use crate::font::{FontLoader, FontSource};
use crate::motion::Motion;
use crate::text::UiLocale;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::ScrollBar;

/// Installs the Paper look on an egui context: fonts, dark visuals from tokens, the motion time
/// and English text.
pub struct Theme;

impl Theme {
    pub fn install(ctx: &Context, source: FontSource) {
        FontLoader::install(ctx, source);
        ctx.set_theme(EguiTheme::Dark);
        ctx.style_mut_of(EguiTheme::Dark, |style| {
            style.visuals = Self::visuals();
            style.animation_time = Motion::DURATION;
            // egui's menu frame (submenus) is the Paper menu panel: these visuals plus this padding
            let padding = <spacing::S as SpacingToken>::VALUE;
            style.spacing.menu_margin = Draw::margin(padding, padding);
            style.spacing.scroll = ScrollBar::style();
        });
        // the host owns pixels_per_point; a keyboard zoom would change egui's scale behind it
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        UiLocale::set(ctx, Arc::new(Localizer::english()));
    }

    fn visuals() -> Visuals {
        let control = Corner::Control.radius(Rect::NOTHING);
        let panel = Corner::Panel.radius(Rect::NOTHING);
        let mut visuals = Visuals::dark();
        visuals.panel_fill = <color::BgPanel as ColorToken>::VALUE;
        visuals.window_fill = <color::BgRaised as ColorToken>::VALUE;
        visuals.extreme_bg_color = <color::BgField as ColorToken>::VALUE;
        visuals.window_stroke = Draw::border(<color::LineStrong as ColorToken>::VALUE);
        visuals.window_corner_radius = panel;
        visuals.menu_corner_radius = panel;
        visuals.window_shadow = Elevation::Floating.shadow();
        visuals.popup_shadow = Elevation::Floating.shadow();
        visuals.selection.bg_fill = <color::Selection as ColorToken>::VALUE;
        visuals.selection.stroke = Draw::border(<color::Accent as ColorToken>::VALUE);
        visuals.text_cursor.stroke = Draw::border(<color::Caret as ColorToken>::VALUE);
        visuals.override_text_color = Some(<color::Text as ColorToken>::VALUE);
        let widgets = &mut visuals.widgets;
        for state in [&mut widgets.inactive, &mut widgets.hovered, &mut widgets.active, &mut widgets.open] {
            state.corner_radius = control;
        }
        widgets.inactive.bg_fill = <color::BgField as ColorToken>::VALUE;
        widgets.inactive.weak_bg_fill = <color::BgField as ColorToken>::VALUE;
        widgets.inactive.bg_stroke = Draw::border(<color::LineStrong as ColorToken>::VALUE);
        widgets.hovered.bg_fill = <color::BgHover as ColorToken>::VALUE;
        widgets.hovered.weak_bg_fill = <color::BgHover as ColorToken>::VALUE;
        widgets.hovered.bg_stroke = Draw::border(<color::LineStrong as ColorToken>::VALUE);
        widgets.active.bg_fill = <color::BgActive as ColorToken>::VALUE;
        widgets.active.weak_bg_fill = <color::BgActive as ColorToken>::VALUE;
        widgets.active.bg_stroke = Draw::border(<color::Accent as ColorToken>::VALUE);
        visuals
    }
}
