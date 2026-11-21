#![allow(unsafe_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

use amml_abi::{AMML_API_VERSION, AmmlApi, AmmlModInfo, AmmlStatus, AmmlStr, AmmlUses};

use crate::{AmmlMod, ApiError, Ctx, PanicText};

// The `Ctx` of this DLL, for the exports only; the crate is linked into each mod, so each mod has
// its own. The first `amml_mod_load` wins.
static CTX: OnceLock<Ctx> = OnceLock::new();
static MODULE: OnceLock<String> = OnceLock::new();

/// The bodies of the four exports `amml_mod!` generates. Each runs inside `catch_unwind` and maps
/// every error to an `AmmlStatus`.
pub struct ModGlue;

impl ModGlue {
    /// Checks the loader table, writes the v3 info block (with the `uses` rows) and calls `M::load`.
    ///
    /// # Safety
    /// `api` and `info` are the pointers the loader passes to `amml_mod_load`: `info` is writable
    /// for one `AmmlModInfo`.
    pub unsafe fn load<M: AmmlMod>(api: *const AmmlApi, info: *mut AmmlModInfo) -> AmmlStatus {
        Self::guard("load", || {
            if info.is_null() {
                return Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT));
            }
            let module = MODULE.get_or_init(M::id).as_str();
            // SAFETY: the loader passes the table it owns; the contract of `Ctx::from_raw` is the same.
            let ctx = unsafe { Ctx::from_raw(api, module, M::USES) }?;
            let ctx = *CTX.get_or_init(|| ctx);
            // SAFETY: a v3 loader (checked by `from_raw`) passes a full `AmmlModInfo`.
            unsafe { Self::write_v3::<M>(info) };
            M::load(&ctx)
        })
    }

    /// Writes every field of a v3 info block. `name`, `namespace` and the uses block are static.
    ///
    /// # Safety
    /// `info` must be valid for writes of one `AmmlModInfo`, suitably aligned.
    unsafe fn write_v3<M: AmmlMod>(info: *mut AmmlModInfo) {
        let uses: &'static AmmlUses = const {
            &AmmlUses {
                size: AmmlUses::SIZE_V1,
                version: 1,
                rows: M::USES.rows().as_ptr(),
                count: M::USES.rows().len(),
            }
        };
        // SAFETY: the caller guarantees the block is writable.
        unsafe {
            info.write(AmmlModInfo {
                size: AmmlModInfo::SIZE_V3,
                api_version: AMML_API_VERSION,
                name: AmmlStr::from_str(M::NAME),
                namespace: AmmlStr::from_str(M::NAMESPACE),
                uses,
            });
        }
    }

    pub fn enable<M: AmmlMod>() -> AmmlStatus {
        Self::with_ctx("enable", |ctx| M::enable(ctx))
    }

    /// Calls `M::disable` with whether the game still runs, read from the loader.
    pub fn disable<M: AmmlMod>() -> AmmlStatus {
        Self::with_ctx("disable", |ctx| {
            M::disable(ctx, &ctx.disable_ctx(ctx.game_alive()));
            Ok(())
        })
    }

    pub fn unload<M: AmmlMod>() -> AmmlStatus {
        Self::with_ctx("unload", |ctx| {
            M::unload(ctx);
            Ok(())
        })
    }

    fn with_ctx(call: &'static str, body: impl FnOnce(&Ctx) -> Result<(), ApiError>) -> AmmlStatus {
        Self::guard(call, || match CTX.get() {
            Some(ctx) => body(ctx),
            None => Err(ApiError::Loader(AmmlStatus::FAILED)),
        })
    }

    fn guard(call: &'static str, body: impl FnOnce() -> Result<(), ApiError>) -> AmmlStatus {
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(Ok(())) => AmmlStatus::OK,
            Ok(Err(error)) => {
                let status = error.status();
                Self::log("a mod callback failed", call, &error.to_string());
                if status.is_ok() { AmmlStatus::FAILED } else { status }
            }
            Err(payload) => {
                PanicText::consume(payload, |text| Self::log("a mod callback panicked", call, text));
                AmmlStatus::PANICKED
            }
        }
    }

    fn log(message: &str, call: &str, reason: &str) {
        let Some(ctx) = CTX.get() else {
            return;
        };
        let _ = catch_unwind(AssertUnwindSafe(|| ctx.log().error(message, &[("call", call), ("reason", reason)])));
    }
}
