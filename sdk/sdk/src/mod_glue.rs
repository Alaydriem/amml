#![allow(unsafe_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use amml_abi::{AmmlApi, AmmlModInfo, AmmlStatus, AmmlStr};

use crate::{AmmlMod, Api, ModRuntime, PanicText, SdkError};

/// The bodies of the four exports `amml_mod!` generates. Each runs inside `catch_unwind` and maps
/// every error to an `AmmlStatus`.
pub struct ModGlue;

impl ModGlue {
    /// # Safety
    /// `api` and `info` are the pointers the loader passes to `amml_mod_load`.
    pub unsafe fn load<M: AmmlMod>(api: *const AmmlApi, info: *mut AmmlModInfo) -> AmmlStatus {
        Self::guard("load", || {
            if info.is_null() {
                return Err(SdkError::NullApi);
            }
            // SAFETY: the loader passes the table it owns, and the contract of `Api::from_raw` is the same.
            let api = unsafe { Api::from_raw(api, M::API_VERSION) }?;
            ModRuntime::install(api, M::NAME);
            M::load(&api)?;
            let block = AmmlModInfo {
                size: AmmlModInfo::SIZE_V1,
                api_version: M::API_VERSION,
                name: AmmlStr::from_str(M::NAME),
            };
            // SAFETY: the loader passes a writable AmmlModInfo; `name` points to a static string.
            unsafe { info.write(block) };
            Ok(())
        })
    }

    pub fn enable<M: AmmlMod>() -> AmmlStatus {
        Self::with_api("enable", |api| M::enable(api))
    }

    pub fn disable<M: AmmlMod>() -> AmmlStatus {
        Self::with_api("disable", |api| {
            M::disable(api);
            Ok(())
        })
    }

    pub fn unload<M: AmmlMod>() -> AmmlStatus {
        Self::with_api("unload", |api| {
            M::unload(api);
            Ok(())
        })
    }

    fn with_api(call: &str, body: impl FnOnce(&Api) -> Result<(), SdkError>) -> AmmlStatus {
        Self::guard(call, || match ModRuntime::get() {
            Some(runtime) => body(runtime.api()),
            None => Err(SdkError::Mod("amml_mod_load did not run".to_string())),
        })
    }

    fn guard(call: &str, body: impl FnOnce() -> Result<(), SdkError>) -> AmmlStatus {
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(Ok(())) => AmmlStatus::OK,
            Ok(Err(error)) => {
                Self::log(&format!("{call} failed: {error}"));
                error.status()
            }
            Err(payload) => {
                ModRuntime::disable();
                Self::log(&format!("{call} panicked: {}", PanicText::of(payload.as_ref())));
                AmmlStatus::PANICKED
            }
        }
    }

    fn log(message: &str) {
        if let Some(runtime) = ModRuntime::get() {
            runtime.logger().error(message);
        }
    }
}
