use amml_abi::{
    BlockBreakContinue, BlockBreakStart, GamepadPoll, IdleScreenOpen, ItemUse, ItemUseOnBlock, KeyInput, LookDelta,
    MouseGrab, MouseInput, PlayerAttack, UiPointer, UiScroll, WindowChar, WindowFocus, WindowPointer,
};

use crate::Event;

/// An event a listener may cancel. Only the cancellable events implement it, so a cancel of any
/// other event does not compile.
pub trait Cancel: Event {
    /// Cancels the event: the game does not get it, and later listeners see it cancelled and cannot
    /// undo it. The loader ignores a cancel of an injected event, of a key or button up whose down
    /// reached the game, and of a `WindowFocus` gain or replay.
    fn cancel(&mut self);
}

macro_rules! cancellable {
    ($($event:ty),* $(,)?) => {
        $(
            const _: () = assert!(<$event as Event>::CANCELLABLE);

            impl Cancel for $event {
                fn cancel(&mut self) {
                    self.header.cancelled = 1;
                }
            }
        )*
    };
}

cancellable!(
    GamepadPoll,
    IdleScreenOpen,
    KeyInput,
    LookDelta,
    MouseGrab,
    MouseInput,
    WindowChar,
    WindowFocus,
    WindowPointer,
    BlockBreakStart,
    BlockBreakContinue,
    ItemUse,
    PlayerAttack,
    ItemUseOnBlock,
    UiPointer,
    UiScroll
);
