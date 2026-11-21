use amml_abi::{AmmlInputTable, AmmlLeaseMode, AmmlLeaseRequest, AmmlStr};

use crate::{ApiError, InputLease, KeyCode, LeaseRequest, Modifiers};

/// The loader's input service: the keys the user holds, and input leases. Usable from any thread.
/// On a loader without an input table every call gives `None`, an empty value or `NotAvailable`.
#[derive(Clone, Copy)]
pub struct Input {
    table: Option<AmmlInputTable>,
    module: &'static str,
}

impl Input {
    pub(crate) fn new(table: Option<AmmlInputTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// The modifier keys the user holds, from the real key feed. F3 counts as a modifier while it is
    /// down. Injected keys never show here.
    pub fn modifiers(&self) -> Modifiers {
        let Some(table) = &self.table else {
            return Modifiers::NONE;
        };
        // SAFETY: the loader's function takes no arguments.
        #[allow(unsafe_code)]
        let raw = unsafe { (table.modifiers)() };
        Modifiers::from(raw).with_f3(self.is_down(KeyCode::F3) == Some(true))
    }

    /// Whether the user holds `key`, from the real key feed. `None` before the first key event, for a
    /// code above 0xFF and without an input table.
    pub fn is_down(&self, key: KeyCode) -> Option<bool> {
        let table = self.table.as_ref()?;
        let mut out = 0u8;
        // SAFETY: the loader accepts any key, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.is_down)(key.0, &mut out) };
        status.is_ok().then_some(out != 0)
    }

    /// Takes an input lease. A request that conflicts with the active leases gives `Refused`.
    pub fn acquire(&self, request: LeaseRequest) -> Result<InputLease, ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let abi = AmmlLeaseRequest {
            size: AmmlLeaseRequest::SIZE_V1,
            mode: AmmlLeaseMode::from(request.mode),
            preempt: u8::from(request.preempt),
            _reserved: [0; 2],
            blocks: request.blocks.into(),
            _reserved2: 0,
            passthrough: request.passthrough.as_ptr().cast::<u16>(),
            passthrough_count: request.passthrough.len(),
        };
        let mut lease = 0u64;
        // SAFETY: `abi` lives for the call; `passthrough` points to `passthrough_count` static key
        // codes, which are `#[repr(transparent)]` over `u16`; the module id lives for the call and
        // `lease` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.acquire)(AmmlStr::from_str(self.module), &abi, &mut lease) };
        ApiError::check(status).map(|()| InputLease::new(table, lease))
    }

    /// Sets the lease ranking, highest first: a ranked mod's exclusive lease suspends the exclusive
    /// leases of mods below it, and ranked mods beat unranked ones. The first mod to set it owns the
    /// list; another mod gives `Refused`.
    pub fn set_ranking(&self, mods: &[&str]) -> Result<(), ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let views: Vec<AmmlStr> = mods.iter().map(|id| AmmlStr::from_str(id)).collect();
        // SAFETY: the module id and every view live for the call, and `views` holds `len` views.
        #[allow(unsafe_code)]
        let status = unsafe { (table.set_ranking)(AmmlStr::from_str(self.module), views.as_ptr(), views.len()) };
        ApiError::check(status)
    }

    /// Sets the pairs of mods whose shared and exclusive leases run together, in either order. Without a
    /// pair, an exclusive lease suspends the shared leases of the other mod while it is held, and a
    /// shared lease is refused while the other mod holds an exclusive one. The owner of the ranking owns
    /// the pairs too (the first mod to set either); another mod gives `Refused`. A loader whose input table
    /// has no coexist function gives `NotAvailable`.
    pub fn set_coexist(&self, pairs: &[(&str, &str)]) -> Result<(), ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let set_coexist = table.set_coexist.ok_or(ApiError::NotAvailable)?;
        let views: Vec<AmmlStr> =
            pairs.iter().flat_map(|(first, second)| [AmmlStr::from_str(first), AmmlStr::from_str(second)]).collect();
        // SAFETY: the module id and every view live for the call, and `views` holds `len` views, two per pair.
        #[allow(unsafe_code)]
        let status = unsafe { set_coexist(AmmlStr::from_str(self.module), views.as_ptr(), views.len()) };
        ApiError::check(status)
    }
}
