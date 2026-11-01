/// A hook the loader installed; pass it to `Api::unhook`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookHandle(pub u64);
