mod modal_response;

pub use modal_response::ModalResponse;

use amml_i18n::text::Text;
use egui::{Align, Context, Frame, Id, Layout, Margin, Modal, Rect, ScrollArea, Sense, Ui, UiBuilder, pos2, vec2};

use crate::AmmlUiMessage;
use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::Icon;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};
use crate::widget::{IconButton, Surface, SurfaceLevel};

/// Paper: Widgets, `Widget / Modal frame`. A floating raised card `--container-dialog` wide over a
/// `--color-bg-overlay` scrim: a header with the title, the subtitle and the close button, the
/// caller's body, and a footer with the caller's buttons. The card is as tall as its body, and
/// keeps `--spacing-lane` from the screen edges; a taller body scrolls.
pub struct ModalFrame {
    id: Id,
    title: Text,
    subtitle: Option<Text>,
    width: f32,
}

impl ModalFrame {
    const TITLE: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);
    const SUBTITLE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(id: Id, title: Text) -> Self {
        Self { id, title, subtitle: None, width: <container::Dialog as ContainerToken>::VALUE }
    }

    /// A card `width` wide instead of `--container-dialog`, for a dialog Paper draws wider (the studio's export,
    /// render queue and coverage dialogs). It still keeps `--spacing-lane` from the screen edges.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn subtitle(mut self, text: Text) -> Self {
        self.subtitle = Some(text);
        self
    }

    /// Shows the modal over the whole screen. `body` adds the content, with `--spacing-m` between
    /// parts. `footer` adds the buttons from right to left: the main action first.
    pub fn show(self, ctx: &Context, body: impl FnOnce(&mut Ui), footer: impl FnOnce(&mut Ui)) -> ModalResponse {
        let screen = ctx.content_rect();
        let lane = <spacing::Lane as SpacingToken>::VALUE;
        let width = self.width.min(screen.width() - lane * 2.0);
        let max_height = (screen.height() - lane * 2.0).max(0.0);
        let modal = Modal::new(self.id).frame(Frame::NONE).backdrop_color(<color::BgOverlay as ColorToken>::VALUE);
        let shown = modal.show(ctx, |ui| self.card(ui, width, max_height, body, footer));
        let inner = shown.inner;
        ModalResponse { closed: inner.closed || shown.should_close(), card: inner.card }
    }

    /// Shows the card where `ui` places it, with no scrim and no Escape, for the gallery.
    pub fn show_card(self, ui: &mut Ui, body: impl FnOnce(&mut Ui), footer: impl FnOnce(&mut Ui)) -> ModalResponse {
        self.card(ui, self.width, f32::INFINITY, body, footer)
    }

    fn card(
        &self,
        ui: &mut Ui,
        width: f32,
        max_height: f32,
        body: impl FnOnce(&mut Ui),
        footer: impl FnOnce(&mut Ui),
    ) -> ModalResponse {
        let border = <spacing::Border as SpacingToken>::VALUE;
        let padding = <spacing::L as SpacingToken>::VALUE;
        let button = <spacing::Button as SpacingToken>::VALUE;
        let divider = <spacing::Divider as SpacingToken>::VALUE;
        let frame = Surface::frame(SurfaceLevel::Raised, Elevation::Floating).inner_margin(Margin::ZERO);
        let shown = frame.show(ui, |ui| {
            let content = width - border * 2.0;
            ui.set_width(content);
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let closed = self.header(ui, content);
            Self::divider(ui);
            let header_height = ui.min_rect().height();
            // the footer is its padding round one button row, under its divider
            let footer_height = divider + padding * 2.0 + button;
            let body_height = (max_height - border * 2.0 - header_height - footer_height - padding * 2.0).max(0.0);
            Frame::new().inner_margin(Draw::margin(padding, padding)).show(ui, |ui| {
                ui.set_width(content - padding * 2.0);
                let gaps = vec2(<spacing::S as SpacingToken>::VALUE, <spacing::M as SpacingToken>::VALUE);
                if body_height.is_finite() {
                    ScrollArea::vertical().max_height(body_height).auto_shrink([false, true]).show(ui, |ui| {
                        ui.spacing_mut().item_spacing = gaps;
                        body(ui);
                    });
                } else {
                    // a card with no screen to fit (the gallery) shows its whole body
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = gaps;
                        body(ui);
                    });
                }
            });
            Self::divider(ui);
            Frame::new().inner_margin(Draw::margin(padding, padding)).show(ui, |ui| {
                let size = vec2(content - padding * 2.0, button);
                ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(<spacing::S as SpacingToken>::VALUE, 0.0);
                    footer(ui);
                });
            });
            closed
        });
        ModalResponse { closed: shown.inner, card: shown.response.rect }
    }

    // the title and subtitle, wrapped, with the close button at the top right; true when it was clicked
    fn header(&self, ui: &mut Ui, content: f32) -> bool {
        let padding = <spacing::L as SpacingToken>::VALUE;
        let row = <spacing::Row as SpacingToken>::VALUE;
        let inner = content - padding * 2.0;
        let room = (inner - <spacing::M as SpacingToken>::VALUE - row).max(0.0);
        Frame::new()
            .inner_margin(Draw::margin(padding, padding))
            .show(ui, |ui| {
                let painter = ui.painter().clone();
                let text = <color::Text as ColorToken>::VALUE;
                let dim = <color::TextDim as ColorToken>::VALUE;
                let title = UiText::resolve(ui.ctx(), &self.title);
                let title = Draw::galley(&painter, &title, Self::TITLE, text, TextFit::Wrap(room));
                let subtitle = self.subtitle.as_ref().map(|subtitle| {
                    let subtitle = UiText::resolve(ui.ctx(), subtitle);
                    Draw::galley(&painter, &subtitle, Self::SUBTITLE, dim, TextFit::Wrap(room))
                });
                let gap = <spacing::S as SpacingToken>::VALUE;
                let text_height = title.size().y + subtitle.as_ref().map_or(0.0, |galley| gap + galley.size().y);
                let (rect, _) = ui.allocate_exact_size(vec2(inner, text_height.max(row)), Sense::hover());
                let title_height = title.size().y;
                painter.galley(rect.min, title, text);
                if let Some(subtitle) = subtitle {
                    painter.galley(pos2(rect.left(), rect.top() + title_height + gap), subtitle, dim);
                }
                let close = Rect::from_min_size(pos2(rect.right() - row, rect.top()), vec2(row, row));
                let button = IconButton::new(Icon::Close).tooltip(Text::message(AmmlUiMessage::Close));
                ui.scope_builder(UiBuilder::new().max_rect(close), |ui| button.show(ui)).inner.clicked()
            })
            .inner
    }

    // a 1 px `--color-line` across the card, taking its own height
    fn divider(ui: &mut Ui) {
        let height = <spacing::Divider as SpacingToken>::VALUE;
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        Draw::divider(ui.painter(), ui.max_rect().x_range(), rect.top(), <color::Line as ColorToken>::VALUE);
    }
}
