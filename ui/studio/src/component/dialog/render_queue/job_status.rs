use amml_i18n::text::Text;
use amml_ui::widget::PillTone;

use crate::message::StudioUiMessage;

/// Paper: `Render queue` (`9A9-0`, `9D1-0`, `Dialog / Render queue — preparing, finishing, reorder menu`). Where a
/// render job is. A queued job of a paused queue is still Queued; the queue's subtitle says it is paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobStatus {
    Queued,
    /// The render takes the game: pause, seek to the in point, settle.
    Preparing,
    Rendering,
    /// The encoder writes the end of the file.
    Finishing,
    Cancelled,
    Done,
    Failed,
}

impl JobStatus {
    /// The status pill's colour.
    pub fn tone(self) -> PillTone {
        match self {
            Self::Preparing | Self::Rendering | Self::Finishing => PillTone::Accent,
            Self::Queued => PillTone::Dim,
            Self::Cancelled => PillTone::Faint,
            Self::Done => PillTone::Success,
            Self::Failed => PillTone::Danger,
        }
    }

    pub fn label(self) -> Text {
        Text::message(match self {
            Self::Queued => StudioUiMessage::JobQueued,
            Self::Preparing => StudioUiMessage::JobPreparing,
            Self::Rendering => StudioUiMessage::JobRendering,
            Self::Finishing => StudioUiMessage::JobFinishing,
            Self::Cancelled => StudioUiMessage::JobCancelled,
            Self::Done => StudioUiMessage::JobDone,
            Self::Failed => StudioUiMessage::JobFailed,
        })
    }

    /// The job renders now: its row has the accent border, and its trailing button cancels it.
    pub fn is_running(self) -> bool {
        matches!(self, Self::Preparing | Self::Rendering | Self::Finishing)
    }

    /// Done, failed or cancelled: the trailing button takes the job off the queue.
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Cancelled | Self::Done | Self::Failed)
    }
}
