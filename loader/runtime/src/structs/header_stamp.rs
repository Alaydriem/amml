use amml_abi::{AmmlEventHeader, AmmlOrigin};

/// The header fields the loader set for one raise, written back over the event header before and
/// after every listener call.
///
/// A listener cannot change `size`, `version`, `phase`, `origin` or `injected` for the listeners
/// after it, or for the loader. For an event that can be cancelled, a cancel is kept for the rest of
/// the raise: later listeners see `cancelled = 1` and cannot undo it. For any other event
/// `cancelled` is always 0.
#[derive(Clone, Copy, Debug)]
pub struct HeaderStamp {
    loaded: AmmlEventHeader,
    cancellable: bool,
    cancelled: bool,
}

impl HeaderStamp {
    /// `loaded` is the header as the loader filled it before the raise.
    pub fn new(loaded: AmmlEventHeader, cancellable: bool) -> Self {
        Self { loaded, cancellable, cancelled: cancellable && loaded.cancelled != 0 }
    }

    /// Writes the loader's fields and the cancel state into `header`.
    pub fn stamp(&self, header: &mut AmmlEventHeader) {
        *header = AmmlEventHeader { cancelled: u8::from(self.cancelled), ..self.loaded };
    }

    /// The header for one listener of the raise: the loader's fields and the cancel state, with
    /// `origin` in place of the loaded origin.
    pub fn with_origin(&self, origin: AmmlOrigin) -> AmmlEventHeader {
        AmmlEventHeader { origin, cancelled: u8::from(self.cancelled), ..self.loaded }
    }

    /// Reads a cancel from `header` after a listener call, then stamps it again. Gives whether the
    /// event is cancelled.
    pub fn after_call(&mut self, header: &mut AmmlEventHeader) -> bool {
        if self.cancellable && header.cancelled != 0 {
            self.cancelled = true;
        }
        self.stamp(header);
        self.cancelled
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
}
