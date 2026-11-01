use std::collections::HashMap;

/// The functions of a build, indexed by what they hold.
#[derive(Default)]
pub(crate) struct FunctionIndex {
    pub(crate) strings: HashMap<String, Vec<u32>>,
    pub(crate) imports: HashMap<String, Vec<u32>>,
    pub(crate) constants: HashMap<u64, Vec<u32>>,
    // function start to (anchor count, size)
    pub(crate) functions: HashMap<u32, (usize, u32)>,
}
