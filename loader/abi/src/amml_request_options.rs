/// The options of `request_moves`.
///
/// `hidden_open` 1: the loader opens the player inventory with the packets the game's own inventory key sends,
/// hides the server's answer from the game and closes it at the end; it refuses while a screen is open. 0: the
/// moves are sent as they are (the caller has the inventory open). `predict` 1: the loader shows each move it can
/// show (a swap, or a whole stack into an empty slot) on the client at once, and puts it back when the server
/// refuses it or does not answer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlRequestOptions {
    pub hidden_open: u8,
    pub predict: u8,
    pub _reserved: [u8; 2],
}

const _: () = assert!(core::mem::size_of::<AmmlRequestOptions>() == 4);
