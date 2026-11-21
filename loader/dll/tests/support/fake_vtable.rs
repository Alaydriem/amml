/// A stand-in vtable of `N` function addresses, for stand-in game objects whose first qword points to it.
#[repr(transparent)]
pub struct FakeVtable<const N: usize>(pub [*const (); N]);

// SAFETY: the table is an immutable static of code addresses.
unsafe impl<const N: usize> Sync for FakeVtable<N> {}
