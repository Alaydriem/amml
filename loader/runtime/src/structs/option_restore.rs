/// What the loader writes at the end of a transient option value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionRestore {
    /// No mod held a transient value of the key: nothing is written.
    Nothing,
    /// The option still held the transient value: the value from before the first transient set goes back, and is
    /// saved.
    Restore(i32),
    /// The option holds a value the user set meanwhile: nothing is written.
    Keep,
}
