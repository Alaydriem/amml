use amml_abi::{AmmlMoveIntent, PlayerMoveInput};

use crate::MoveIntent;

/// Typed reads and writes of a `PlayerMoveInput`.
pub trait PlayerMoveInputExt {
    fn intent(&self) -> MoveIntent;
    /// Changes the movement the game gets this frame; later listeners see the new value.
    fn set_intent(&mut self, intent: MoveIntent);
}

impl PlayerMoveInputExt for PlayerMoveInput {
    fn intent(&self) -> MoveIntent {
        MoveIntent::from(self.intent)
    }

    fn set_intent(&mut self, intent: MoveIntent) {
        self.intent = AmmlMoveIntent::from(intent);
    }
}
