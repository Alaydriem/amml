/// Paper: `Lanes` (`6WK-0`). How a stretch between two keys looks on its lane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpanLook {
    /// A filled bar that grows stronger with `strength` (0 to 1), its label centred: the timelapse speed, the time
    /// of day.
    Fill { strength: f32 },
    /// A clip with a border and its label at the left: the weather.
    Clip { strength: f32 },
    /// The camera bar (dashed for a hold) with the easing name in a pill.
    Ease { hold: bool },
}
