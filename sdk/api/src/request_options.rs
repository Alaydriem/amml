use amml_abi::AmmlRequestOptions;

/// The options of `Inventory::request_moves`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RequestOptions {
    /// The loader opens the player inventory itself with the packets the game's own inventory key sends, hides the
    /// server's answer from the game and closes it at the end. The call is refused while a screen is open. Without
    /// it the moves are sent as they are: the caller has the inventory open.
    pub hidden_open: bool,
    /// The loader shows each move it can show (a swap, or a whole stack into an empty slot) on the client at once,
    /// and puts it back when the server refuses it or does not answer.
    pub predict: bool,
}

impl From<RequestOptions> for AmmlRequestOptions {
    fn from(value: RequestOptions) -> Self {
        Self { hidden_open: u8::from(value.hidden_open), predict: u8::from(value.predict), _reserved: [0; 2] }
    }
}

impl From<AmmlRequestOptions> for RequestOptions {
    fn from(value: AmmlRequestOptions) -> Self {
        Self { hidden_open: value.hidden_open != 0, predict: value.predict != 0 }
    }
}
