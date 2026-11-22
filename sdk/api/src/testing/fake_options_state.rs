use std::cell::RefCell;

use amml_abi::{AmmlOptionKey, AmmlStatus};
use amml_runtime::{OptionRestore, TransientOptionBook};

use crate::OptionKey;
use crate::testing::FakeOptions;

/// The options side of a `FakeGame`: the options, the transient values of the fake mod and the writes that went out, one
/// line each. The fake has one mod, so no other mod holds a transient value.
pub(crate) struct FakeOptionsState {
    options: RefCell<FakeOptions>,
    book: RefCell<TransientOptionBook>,
    writes: RefCell<Vec<String>>,
}

impl Default for FakeOptionsState {
    fn default() -> Self {
        Self {
            options: RefCell::new(FakeOptions::default()),
            book: RefCell::new(TransientOptionBook::new()),
            writes: RefCell::new(Vec::new()),
        }
    }
}

impl FakeOptionsState {
    // The owner of every transient value: the fake has one mod.
    const OWNER: usize = 1;

    /// Replaces the options. The transient values of the mod are kept: a release then compares with the new values.
    pub(crate) fn set(&self, options: FakeOptions) {
        *self.options.borrow_mut() = options;
    }

    pub(crate) fn view_distance(&self) -> u32 {
        self.options.borrow().view_distance
    }

    pub(crate) fn writes(&self) -> Vec<String> {
        self.writes.borrow().clone()
    }

    // The key and its value: `INVALID_ARGUMENT` for an unknown key, `NOT_AVAILABLE` for an option the fake does not
    // have.
    fn value(&self, key: AmmlOptionKey) -> Result<(OptionKey, i32), AmmlStatus> {
        let key = OptionKey::from_abi(key).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        let value = self.options.borrow().ints.get(&key).copied().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok((key, value))
    }

    pub(crate) fn get_int(&self, key: AmmlOptionKey) -> Result<i32, AmmlStatus> {
        self.value(key).map(|(_, value)| value)
    }

    /// A transient set of the fake mod.
    pub(crate) fn set_int(&self, key: AmmlOptionKey, value: i32) -> Result<(), AmmlStatus> {
        let (key, current) = self.value(key)?;
        // The fake has one game option per key, so the option id is the key's.
        let option = i32::try_from(key.to_abi().0).unwrap_or(-1);
        self.book
            .borrow_mut()
            .set(Self::OWNER, key.to_abi().0, option, current, value)
            .map_err(|_| AmmlStatus::REFUSED)?;
        self.options.borrow_mut().ints.insert(key, value);
        self.writes.borrow_mut().push(format!("set_int {}={value} transient", key.name()));
        Ok(())
    }

    /// A `release_transient` of the fake mod.
    pub(crate) fn release(&self, key: AmmlOptionKey) -> Result<(), AmmlStatus> {
        let (key, current) = self.value(key)?;
        let restore =
            self.book.borrow_mut().release(Self::OWNER, key.to_abi().0, current).map_err(|_| AmmlStatus::REFUSED)?;
        match restore {
            OptionRestore::Restore(value) => {
                self.options.borrow_mut().ints.insert(key, value);
                self.writes.borrow_mut().push(format!("release {} restored={value}", key.name()));
            }
            OptionRestore::Keep => self.writes.borrow_mut().push(format!("release {} kept={current}", key.name())),
            OptionRestore::Nothing => {}
        }
        Ok(())
    }

    /// The fake mod is disabled: each key it holds is released, as `release_transient` does.
    pub(crate) fn disable_owner(&self) {
        let keys = self.book.borrow().keys_of(Self::OWNER);
        for key in keys {
            let _ = self.release(AmmlOptionKey(key));
        }
    }
}
