/// The handle of a clock hold: `id` names the clock (1, the one clock of the process; 0 is no hold), `generation`
/// the hold. A handle of an earlier hold is stale.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlTimeHold {
    pub id: u32,
    pub generation: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlTimeHold>() == 8);
