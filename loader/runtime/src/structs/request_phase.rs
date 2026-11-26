/// Where a request is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RequestPhase {
    /// Taken; the next tick starts it.
    Starting,
    /// `Interact` was sent; the server's open is waited for until `until`.
    Opening { until: f64 },
    /// The server opened the inventory as `container_id` (hidden from the game); the moves go out at the next tick.
    Opened { container_id: i8 },
    /// The requests were sent at `sent_at`; the answers are waited for. `container_id` is the hidden inventory.
    Sent { container_id: Option<i8>, sent_at: f64 },
}
