use std::sync::Mutex;

use crate::{ModLoader, ModPhase};

/// Mods whose listener panicked, waiting to be disabled.
///
/// A mod is not disabled inside the dispatch that saw the panic: the disable waits for the mod's
/// listeners, and the dispatch may run inside one of them. The session adapter drains the list at
/// the start of each `FrameUpdate`, on the game thread.
#[derive(Debug, Default)]
pub struct DeferredDisable {
    owners: Mutex<Vec<usize>>,
}

impl DeferredDisable {
    pub fn new() -> Self {
        Self { owners: Mutex::new(Vec::new()) }
    }

    /// Queues the mod at index `owner`; a mod already queued is queued once.
    pub fn push(&self, owner: usize) {
        let mut owners = self.owners.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }

    /// True when the mod at index `owner` waits to be disabled.
    pub fn is_queued(&self, owner: usize) -> bool {
        self.owners.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).contains(&owner)
    }

    /// Disables every queued mod that is still enabled.
    pub fn drain(&self, loader: &ModLoader) {
        let owners = std::mem::take(&mut *self.owners.lock().unwrap_or_else(|poisoned| poisoned.into_inner()));
        if owners.is_empty() {
            return;
        }
        let phases = loader.phases();
        for owner in owners {
            let Some((id, ModPhase::Enabled)) = phases.get(owner) else {
                continue;
            };
            loader.disable_one(owner);
            curia::error!("mod disabled after a listener panic", { mod_name: id.as_str() });
        }
    }
}
