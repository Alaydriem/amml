mod meta_separator;
mod page_header_lead;
mod page_header_trailing;

pub use meta_separator::MetaSeparator;
pub use page_header_lead::PageHeaderLead;
pub use page_header_trailing::PageHeaderTrailing;

use std::sync::Arc;

use amml_i18n::text::Text;
use egui::{Align, Galley, Layout, Painter, Rect, Response, Sense, Ui, UiBuilder, pos2, vec2};

use crate::draw::{Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::text::UiText;
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Surface, SurfaceLevel};

/// Paper: Mod manager, `Widget / Page header`. A panel bar with a lead tile or avatar, a title
/// with meta items or a subtitle, and trailing parts at the right. With an eyebrow it takes the
/// panel form: a smaller tile, and the meta on the title's line. The title and meta are cut to one
/// line each, so the trailing parts keep their width.
pub struct PageHeader<'a> {
    lead: PageHeaderLead,
    title: Text,
    eyebrow: Option<Text>,
    meta: Vec<Text>,
    subtitle: Option<Text>,
    trailing: Option<PageHeaderTrailing<'a>>,
}

impl<'a> PageHeader<'a> {
    /// Paper: Page header, the bar's left and right padding.
    const PAD_X: f32 = 24.0;
    /// Paper: Page header, the bar's top and bottom padding.
    const PAD_Y: f32 = 20.0;
    /// Paper: Page header, the gap under the eyebrow in the panel form.
    const EYEBROW_GAP: f32 = 2.0;
    const EYEBROW: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const TITLE: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    pub fn new(lead: PageHeaderLead, title: Text) -> Self {
        Self { lead, title, eyebrow: None, meta: Vec::new(), subtitle: None, trailing: None }
    }

    /// The small caps line over the title. It turns on the panel form.
    pub fn eyebrow(mut self, text: Text) -> Self {
        self.eyebrow = Some(text);
        self
    }

    /// Items joined by drawn dots: "freecam · v0.1.0 · by Alaydriem".
    pub fn meta(mut self, items: Vec<Text>) -> Self {
        self.meta = items;
        self
    }

    /// One line under the title, in place of meta items.
    pub fn subtitle(mut self, text: Text) -> Self {
        self.subtitle = Some(text);
        self
    }

    /// The chip, toggle or close button at the right, laid out right to left.
    pub fn trailing(mut self, trailing: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.trailing = Some(Box::new(trailing));
        self
    }

    /// The bar's height: the lead or the text column, whichever is taller, and the padding.
    pub fn height(&self) -> f32 {
        let panel = self.eyebrow.is_some();
        self.lead.side(panel).max(self.column_height(panel)) + 2.0 * Self::PAD_Y
    }

    /// Takes the full width.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let panel = self.eyebrow.is_some();
        let gap = <spacing::L as SpacingToken>::VALUE;
        let lead = self.lead.side(panel);
        let column_height = self.column_height(panel);
        let height = self.height();
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let painter = ui.painter().clone();
        Surface::paint(&painter, rect, SurfaceLevel::Panel, Elevation::Flat);
        let inner = rect.shrink2(vec2(Self::PAD_X, Self::PAD_Y));

        let mut right = inner.right();
        if let Some(trailing) = self.trailing.take() {
            let builder = UiBuilder::new().max_rect(inner).layout(Layout::right_to_left(Align::Center));
            // a child, not a scope: a scope would move the cursor back up to the trailing parts
            let mut child = ui.new_child(builder);
            trailing(&mut child);
            let used = child.min_rect();
            if used.width() > 0.0 {
                right = used.left() - gap;
            }
        }

        let lead_rect = Rect::from_min_size(pos2(inner.left(), inner.center().y - lead / 2.0), vec2(lead, lead));
        self.lead.paint(&painter, lead_rect, panel);
        let left = lead_rect.right() + gap;
        let width = (right - left).max(0.0);
        let mut y = inner.center().y - column_height / 2.0;

        let text = <color::Text as ColorToken>::VALUE;
        let dim = <color::TextDim as ColorToken>::VALUE;
        if let Some(eyebrow) = &self.eyebrow {
            let eyebrow = UiText::resolve(ui.ctx(), eyebrow);
            painter.galley(
                pos2(left, y),
                Draw::galley(&painter, &eyebrow, Self::EYEBROW, dim, TextFit::Truncate(width)),
                dim,
            );
            y += TypeSize::Caption.line_height() + Self::EYEBROW_GAP;
        }
        let title = UiText::resolve(ui.ctx(), &self.title);
        let title = Draw::galley(&painter, &title, Self::TITLE, text, TextFit::Truncate(width));
        let title_width = title.size().x;
        let baseline = y + Self::baseline(&title);
        painter.galley(pos2(left, y), title, text);

        if panel {
            // the meta follows the title on its baseline
            let x = left + title_width + <spacing::M as SpacingToken>::VALUE;
            self.paint_meta(&painter, ui, x, right, |galley| baseline - Self::baseline(galley));
        } else {
            let top = y + TypeSize::Title.line_height() + <spacing::Xs as SpacingToken>::VALUE;
            if let Some(subtitle) = &self.subtitle {
                let subtitle = UiText::resolve(ui.ctx(), subtitle);
                painter.galley(
                    pos2(left, top),
                    Draw::galley(&painter, &subtitle, Self::NOTE, dim, TextFit::Truncate(width)),
                    dim,
                );
            }
            self.paint_meta(&painter, ui, left, right, |_| top);
        }
        response
    }

    // the eyebrow and title line, or the title and the line under it
    fn column_height(&self, panel: bool) -> f32 {
        let title = TypeSize::Title.line_height();
        if panel {
            return TypeSize::Caption.line_height() + Self::EYEBROW_GAP + title;
        }
        if self.meta.is_empty() && self.subtitle.is_none() {
            return title;
        }
        title + <spacing::Xs as SpacingToken>::VALUE + TypeSize::Note.line_height()
    }

    // the meta items from `x`, cut at `right`, each placed at the top `top` gives for its galley
    fn paint_meta(&self, painter: &Painter, ui: &Ui, mut x: f32, right: f32, top: impl Fn(&Galley) -> f32) {
        let dim = <color::TextDim as ColorToken>::VALUE;
        let gap = <spacing::S as SpacingToken>::VALUE;
        for (index, item) in self.meta.iter().enumerate() {
            if index > 0 {
                let dot = x + gap + MetaSeparator::SIDE / 2.0;
                if dot + MetaSeparator::SIDE / 2.0 + gap >= right {
                    return;
                }
                x += 2.0 * gap + MetaSeparator::SIDE;
            }
            let item = UiText::resolve(ui.ctx(), item);
            let galley: Arc<Galley> = Draw::galley(painter, &item, Self::NOTE, dim, TextFit::Truncate(right - x));
            let y = top(&galley);
            if index > 0 {
                MetaSeparator::paint(painter, pos2(x - gap - MetaSeparator::SIDE / 2.0, y + galley.size().y / 2.0));
            }
            let width = galley.size().x;
            painter.galley(pos2(x, y), galley, dim);
            x += width;
        }
    }

    // the first row's baseline, from the galley's top
    fn baseline(galley: &Galley) -> f32 {
        galley.rows.first().and_then(|row| row.glyphs.first().map(|glyph| row.pos.y + glyph.pos.y)).unwrap_or(0.0)
    }
}
