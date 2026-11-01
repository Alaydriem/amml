/// Paper: Widgets, `Widget / Status badge + Placed badge` (`42V-0`), section "RIGHT / ABOVE / BELOW". Where a
/// placed badge sits against its control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgePlacement {
    /// After the control, centred on its height.
    Right,
    /// Over the control, on its left edge.
    Above,
    /// Under the control, on its left edge.
    Below,
}
