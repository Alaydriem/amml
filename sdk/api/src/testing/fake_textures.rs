use std::rc::Rc;

use crate::TextureId;
use crate::testing::FakeState;

/// The textures of a `FakeGame`, as `FakeGame::textures` gives them: what the mod asked for, the ready flag the
/// loader would set once the game gave the data, the world leave, and the queue of frees made on other threads.
pub struct FakeTextures {
    state: Rc<FakeState>,
}

impl FakeTextures {
    pub(crate) fn new(state: Rc<FakeState>) -> Self {
        Self { state }
    }

    /// The uploads so far: the texture, its width and its height.
    pub fn uploads(&self) -> Vec<(TextureId, u32, u32)> {
        self.state.textures().uploads()
    }

    /// The file textures asked for so far, as `mod_file <path>` or `vanilla <path>`.
    pub fn loads(&self) -> Vec<String> {
        self.state.textures().loads()
    }

    /// The textures freed so far, by an `unload` on this thread or by `pump`.
    pub fn unloads(&self) -> Vec<TextureId> {
        self.state.textures().unloads()
    }

    /// Sets whether the game gave the data of `texture`. A texture is not ready until a test says so.
    pub fn set_ready(&self, texture: TextureId, ready: bool) {
        self.state.textures().set_ready(texture, ready);
    }

    /// A world leave: every texture is freed, and a later use of its id gives `Stale`.
    pub fn leave_world(&self) {
        self.state.textures().leave_world();
    }

    /// Applies the frees that were asked for on other threads, as the loader does at its next UI draw. Gives the
    /// number of textures freed.
    pub fn pump(&self) -> usize {
        self.state.textures().pump()
    }
}
