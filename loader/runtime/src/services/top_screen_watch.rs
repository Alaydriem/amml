use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::structs::{ScreenChange, WatchState};

/// Finds the change of the top screen at each client tick.
///
/// The game draws every scene each frame, and not in stack order: on 26.63 the HUD and the toast are drawn after
/// an open inventory. So the top screen is not the last view drawn. It is the screen drawn since the previous tick
/// whose root control name is the game's top screen name (`getTopScreenName`), or that name followed by ` - ` and
/// a detail (`world_loading_progress_screen - local_world_load`). The root control name is what
/// `UIScene::getRawScreenName` gives. When two drawn screens match, the last one drawn is taken.
///
/// `tick` reports a change once. A toast name changes nothing. When no view was drawn the top screen is none. When
/// views were drawn but none matches, the first such tick changes nothing (the new screen is drawn one frame after
/// the name changes) and a second tick in a row with the same name makes the top screen none.
pub struct TopScreenWatch {
    state: Mutex<WatchState>,
}

impl Default for TopScreenWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl TopScreenWatch {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WatchState {
                drawn: Vec::new(),
                last_order: Vec::new(),
                names: HashMap::new(),
                unmatched: None,
                kept: 0,
            }),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, WatchState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A view of the screen `screen` was drawn. Gives whether the screen has no name yet, so the caller reads it
    /// (`set_name`) while the view is live.
    pub fn note_drawn(&self, screen: u64) -> bool {
        let mut state = self.lock();
        if state.drawn.last() != Some(&screen) {
            state.drawn.retain(|known| *known != screen);
            state.drawn.push(screen);
        }
        !state.names.contains_key(&screen)
    }

    /// The root control name of `screen`.
    pub fn set_name(&self, screen: u64, name: &str) {
        self.lock().names.insert(screen, name.to_string());
    }

    /// The root control name of `screen`, when it was read.
    pub fn name_of(&self, screen: u64) -> Option<String> {
        self.lock().names.get(&screen).cloned()
    }

    /// The controller of `screen` was freed: its name is dropped.
    pub fn forget(&self, screen: u64) {
        self.lock().names.remove(&screen);
    }

    /// Ends one tick. `top` is the game's top screen name and `toast` whether it is a notice: the tick then changes
    /// nothing and the kept screen stays. Otherwise gives the change when the top screen differs from the kept one.
    pub fn tick(&self, top: &str, toast: bool) -> Option<ScreenChange> {
        let mut state = self.lock();
        let drawn = std::mem::take(&mut state.drawn);
        let matched = drawn
            .iter()
            .rev()
            .find(|screen| state.names.get(screen).is_some_and(|name| Self::matches(name, top)))
            .copied();
        state.last_order = drawn;
        if toast {
            return None;
        }
        let new = match matched {
            Some(screen) => {
                state.unmatched = None;
                screen
            }
            None if state.last_order.is_empty() => {
                state.unmatched = None;
                0
            }
            None => {
                if state.unmatched.as_deref() != Some(top) {
                    state.unmatched = Some(top.to_string());
                    return None;
                }
                0
            }
        };
        if new == state.kept {
            return None;
        }
        let change = ScreenChange { old: state.kept, new };
        state.kept = new;
        Some(change)
    }

    /// The screens drawn in the tick that ended last, each once, in the order of its last draw.
    pub fn drawn_order(&self) -> Vec<u64> {
        self.lock().last_order.clone()
    }

    // Whether the root control name `name` is the screen of the top screen name `top`.
    fn matches(name: &str, top: &str) -> bool {
        !name.is_empty() && top.strip_prefix(name).is_some_and(|rest| rest.is_empty() || rest.starts_with(" - "))
    }
}
