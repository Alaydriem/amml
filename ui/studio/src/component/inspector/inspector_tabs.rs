use amml_ui::widget::{TabRow, TabRowSize};
use egui::{Id, Ui};

use super::InspectorTab;
use crate::id::StudioUiId;

/// Paper: Menus + Dialogs, `Key active` (`4JQ-0`). The inspector's tab strip: the amml-ui `TabRow` at the Inspector
/// size, with the three tab names.
pub struct InspectorTabs {
    selected: InspectorTab,
    id: Id,
}

impl InspectorTabs {
    pub fn new(selected: InspectorTab) -> Self {
        Self { selected, id: Id::unique(StudioUiId::InspectorTabs) }
    }

    /// Another id, for a second strip on one screen (the gallery shows several).
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Shows the strip; returns the tab that was picked, when it changed.
    pub fn show(&self, ui: &mut Ui) -> Option<InspectorTab> {
        let mut index = self.selected.index();
        let tabs = InspectorTab::ALL.iter().map(|tab| tab.label()).collect();
        let response = TabRow::new(self.id, &mut index, tabs).size(TabRowSize::Inspector).show(ui);
        response.changed().then(|| InspectorTab::ALL[index])
    }
}
