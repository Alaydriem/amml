use crate::{OptionRestore, TransientHold, TransientOptionRefusal};

/// The transient integer option values the mods hold.
///
/// Per key one mod holds a transient value: the first `set` keeps the value the option held before it, a later `set` of
/// the same mod only changes the transient value, and a `set` of another mod is refused. A `release` gives the value to
/// write back only while the option still holds the transient value; when the user changed it meanwhile the value is
/// theirs and nothing is written. The hold keeps the game option id of the first set, because a key can name a
/// different game option at a later time (the effective max frame rate option); the restore goes to that option. The
/// book makes no game call: the option values are passed in, and the game thread and
/// the game are checked by the loader.
#[derive(Debug, Default)]
pub struct TransientOptionBook {
    holds: Vec<TransientHold>,
}

impl TransientOptionBook {
    pub fn new() -> Self {
        Self { holds: Vec::new() }
    }

    /// The mod that holds a transient value of `key`.
    pub fn holder(&self, key: u32) -> Option<usize> {
        self.holds.iter().find(|hold| hold.key == key).map(|hold| hold.owner)
    }

    /// The game option id the first transient set of `key` wrote; `None` when nobody holds the key.
    pub fn held_option(&self, key: u32) -> Option<i32> {
        self.holds.iter().find(|hold| hold.key == key).map(|hold| hold.option)
    }

    /// The keys the mod `owner` holds, in the order it took them.
    pub fn keys_of(&self, owner: usize) -> Vec<u32> {
        self.holds.iter().filter(|hold| hold.owner == owner).map(|hold| hold.key).collect()
    }

    /// Forgets every hold of the mod `owner`, with no write. For a disable when no game call is possible.
    pub fn forget_owner(&mut self, owner: usize) {
        self.holds.retain(|hold| hold.owner != owner);
    }

    /// A transient set of `key` to `value` by the mod `owner` on the game option `option`, while that option holds
    /// `current`. A later set of the same mod keeps the option and the value of the first set. `Taken` when another mod
    /// holds the key.
    pub fn set(
        &mut self,
        owner: usize,
        key: u32,
        option: i32,
        current: i32,
        value: i32,
    ) -> Result<(), TransientOptionRefusal> {
        match self.holds.iter_mut().find(|hold| hold.key == key) {
            Some(hold) if hold.owner != owner => Err(TransientOptionRefusal::Taken),
            Some(hold) => {
                hold.transient = value;
                Ok(())
            }
            None => {
                self.holds.push(TransientHold { key, option, owner, before: current, transient: value });
                Ok(())
            }
        }
    }

    /// The mod `owner` ends its transient value of `key`, while the option holds `current`. The hold is forgotten in
    /// every `Ok` case. `Taken` when another mod holds the key.
    pub fn release(&mut self, owner: usize, key: u32, current: i32) -> Result<OptionRestore, TransientOptionRefusal> {
        let Some(at) = self.holds.iter().position(|hold| hold.key == key) else {
            return Ok(OptionRestore::Nothing);
        };
        let hold = self.holds[at];
        if hold.owner != owner {
            return Err(TransientOptionRefusal::Taken);
        }
        self.holds.remove(at);
        Ok(if current == hold.transient { OptionRestore::Restore(hold.before) } else { OptionRestore::Keep })
    }
}
