use amml_i18n::message::{Domain, Message};

/// The words amml-ui widgets show on their own, the same in every mod. Everything else a widget
/// shows comes from the caller as `Text`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmmlUiMessage {
    Close,
    Dismiss,
    ClearSearch,
    Now,
    SecondsAgo,
    MoreItems,
    PressAKey,
    EscCancels,
    AlsoBoundBy,
    ChangedInGame,
    InvalidColour,
    EnterANumber,
    Cancel,
    NotPlaced,
    GridCell,
    DropHere,
    PlaceHere,
    Taken,
    Unplace,
}

impl Message for AmmlUiMessage {
    const DOMAIN: Domain = Domain::new("amml-ui");
    const ALL: &'static [Self] = &[
        Self::Close,
        Self::Dismiss,
        Self::ClearSearch,
        Self::Now,
        Self::SecondsAgo,
        Self::MoreItems,
        Self::PressAKey,
        Self::EscCancels,
        Self::AlsoBoundBy,
        Self::ChangedInGame,
        Self::InvalidColour,
        Self::EnterANumber,
        Self::Cancel,
        Self::NotPlaced,
        Self::GridCell,
        Self::DropHere,
        Self::PlaceHere,
        Self::Taken,
        Self::Unplace,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn msgid(self) -> &'static str {
        match self {
            Self::Close => "Close",
            Self::Dismiss => "Dismiss",
            Self::ClearSearch => "Clear search",
            Self::Now => "now",
            Self::SecondsAgo => "{count} s",
            Self::MoreItems => "+{count}",
            Self::PressAKey => "Press a key",
            Self::EscCancels => "Esc cancels",
            Self::AlsoBoundBy => "Also bound by {name}",
            Self::ChangedInGame => "Changed in game to {value}",
            Self::InvalidColour => "Use #RRGGBB or #RRGGBBAA.",
            Self::EnterANumber => "Enter a number",
            Self::Cancel => "Cancel",
            Self::NotPlaced => "NOT PLACED · {count}",
            Self::GridCell => "Cell {number}",
            Self::DropHere => "Drop here",
            Self::PlaceHere => "Place here",
            Self::Taken => "Taken",
            Self::Unplace => "Back to the tray",
        }
    }

    fn context(self) -> Option<&'static str> {
        None
    }

    fn plural(self) -> Option<&'static str> {
        match self {
            Self::SecondsAgo => Some("{count} s"),
            _ => None,
        }
    }

    fn note(self) -> Option<&'static str> {
        match self {
            Self::Now => Some("A toast's age when it is less than one second old"),
            Self::SecondsAgo => Some("A toast's age in whole seconds"),
            Self::MoreItems => Some("A chip that stands for the items that do not fit in the row"),
            Self::PressAKey => Some("Shown in a key chord cell while the key is being captured"),
            Self::AlsoBoundBy => Some("{name} is the other mod that uses the same key chord"),
            Self::ChangedInGame => Some("{value} is the setting value the game wrote while an edit was open"),
            Self::NotPlaced => {
                Some("The grid placer's tray heading, in upper case; {count} is the number of pieces not on the grid")
            }
            Self::GridCell => Some("The label of one grid placer cell; {number} counts the cells from 1, row by row"),
            Self::DropHere => Some("On an empty grid cell while a piece is dragged over it"),
            Self::PlaceHere => Some("On the grid cell the keyboard reached while a piece is held"),
            Self::Taken => {
                Some("On a filled grid cell while another piece is dragged over it; the cell refuses the drop")
            }
            Self::Unplace => Some("The tooltip of the button that takes a piece off a grid cell"),
            _ => None,
        }
    }
}
