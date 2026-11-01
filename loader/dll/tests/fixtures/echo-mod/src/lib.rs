//! A mod that reports what the loader gave it through its own log, for the loader tests.

use amml_sdk::{AmmlMod, Api, Logger, ModDirs, SdkError, amml_mod};

struct Echo;

// Adds 1000 to the result of the hooked function.
#[cfg(feature = "hook-target")]
mod add_thousand {
    amml_sdk::amml_detour!(pub mod detour(x: i32) -> i32 |original| { original(x) + 1000 });
}

// Adds a million; it is installed only from `disable`, where the loader refuses it.
#[cfg(feature = "hook-target")]
mod add_million {
    amml_sdk::amml_detour!(pub mod detour(x: i32) -> i32 |original| { original(x) + 1_000_000 });
}

impl Echo {
    #[allow(dead_code)]
    fn report(api: &Api) -> Result<(), SdkError> {
        let logger = Logger::new(api, &<Self as AmmlMod>::id());
        match api.resolve("Echo.found") {
            Ok(address) => logger.info(&format!("found={address:#x}")),
            Err(error) => logger.info(&format!("found failed: {error}")),
        }
        match api.resolve("Echo.absent") {
            Ok(address) => logger.info(&format!("absent={address:#x}")),
            Err(error) => logger.info(&format!("absent status={:?}", error.status())),
        }
        let block = api.heap_new(16);
        logger.info(&format!("heap block null={}", block.is_null()));
        if !block.is_null() {
            // SAFETY: `block` came from `heap_new(16)` just above and is freed once.
            unsafe { api.heap_delete(block, 16) };
        }
        let dirs = ModDirs::new(api, &<Self as AmmlMod>::id())?;
        logger.info(&format!("mod_dir={}", dirs.mod_dir.display()));
        logger.info(&format!("data_dir={}", dirs.data_dir.display()));
        logger.info("enabled");
        Ok(())
    }
}

impl Echo {
    #[cfg(feature = "hook-target")]
    fn hook_target(api: &Api) -> Result<(), SdkError> {
        let logger = Logger::new(api, &<Self as AmmlMod>::id());
        let target = api.resolve("Echo.target")?;
        // SAFETY: `Echo.target` is an `extern "C" fn(i32) -> i32`, the signature of the detour, and
        // the original slot belongs to this one hook.
        let hook = unsafe {
            api.hook(
                target,
                add_thousand::detour::detour as *const std::ffi::c_void,
                0,
                add_thousand::detour::GUARD.original(),
            )
        }?;
        logger.info(&format!("hooked handle={}", hook.0));
        Ok(())
    }

    #[cfg(feature = "hook-target")]
    fn hook_in_disable(api: &Api) {
        let logger = Logger::new(api, &<Self as AmmlMod>::id());
        let Ok(target) = api.resolve("Echo.target") else {
            return;
        };
        // SAFETY: the same target and signature as `hook_target`, with a second detour and slot.
        let result = unsafe {
            api.hook(
                target,
                add_million::detour::detour as *const std::ffi::c_void,
                0,
                add_million::detour::GUARD.original(),
            )
        };
        match result {
            Ok(hook) => logger.info(&format!("disable hook accepted handle={}", hook.0)),
            Err(error) => logger.info(&format!("disable hook status={:?}", error.status())),
        }
    }

    #[cfg(feature = "steal-hook")]
    fn steal_hook(api: &Api) -> Result<(), SdkError> {
        let logger = Logger::new(api, &<Self as AmmlMod>::id());
        match api.unhook(amml_sdk::HookHandle(1)) {
            Ok(()) => logger.info("unhook accepted"),
            Err(error) => logger.info(&format!("unhook status={:?}", error.status())),
        }
        Ok(())
    }

    #[allow(unreachable_code)]
    fn enable_variant(api: &Api) -> Result<(), SdkError> {
        #[cfg(feature = "hook-target")]
        return Self::hook_target(api);
        #[cfg(feature = "steal-hook")]
        return Self::steal_hook(api);
        Self::report(api)
    }
}

impl AmmlMod for Echo {
    const NAMESPACE: &'static str = "echo";
    #[cfg(feature = "panic-enable")]
    const NAME: &'static str = "echo-panic";
    #[cfg(all(feature = "newer-api", not(feature = "panic-enable")))]
    const NAME: &'static str = "echo-newer";
    #[cfg(all(feature = "old-info", not(any(feature = "panic-enable", feature = "newer-api"))))]
    const NAME: &'static str = "echo-old";
    #[cfg(all(
        feature = "hook-target",
        not(any(feature = "panic-enable", feature = "newer-api", feature = "old-info"))
    ))]
    const NAME: &'static str = "echo-hook";
    #[cfg(all(
        feature = "steal-hook",
        not(any(feature = "panic-enable", feature = "newer-api", feature = "old-info", feature = "hook-target"))
    ))]
    const NAME: &'static str = "echo-thief";
    #[cfg(not(any(
        feature = "panic-enable",
        feature = "newer-api",
        feature = "old-info",
        feature = "hook-target",
        feature = "steal-hook"
    )))]
    const NAME: &'static str = "echo";

    #[cfg(feature = "newer-api")]
    const API_VERSION: u32 = 3;

    fn enable(api: &Api) -> Result<(), SdkError> {
        #[cfg(feature = "panic-enable")]
        {
            let _ = api;
            panic!("echo-mod enable panics on purpose");
        }
        #[cfg(not(feature = "panic-enable"))]
        Self::enable_variant(api)
    }

    fn disable(api: &Api) {
        Logger::new(api, &Self::id()).info("disabled");
        #[cfg(feature = "hook-target")]
        Self::hook_in_disable(api);
    }

    fn unload(api: &Api) {
        Logger::new(api, &Self::id()).info("unloaded");
    }
}

#[cfg(not(feature = "old-info"))]
amml_mod!(Echo);

// A mod built before the namespace: its info block is the 24-byte v1 block.
#[cfg(feature = "old-info")]
mod old_info {
    use amml_sdk::amml_abi::{AmmlApi, AmmlStatus, AmmlStr};
    use amml_sdk::{AmmlMod, ModGlue};

    use super::Echo;

    #[repr(C)]
    struct InfoV1 {
        size: u32,
        api_version: u32,
        name: AmmlStr,
    }

    /// # Safety
    /// Called by the AMML loader with its API table and a writable info block.
    #[allow(unsafe_code)]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn amml_mod_load(
        _api: *const AmmlApi,
        info: *mut amml_sdk::amml_abi::AmmlModInfo,
    ) -> AmmlStatus {
        let block = InfoV1 { size: 24, api_version: 1, name: AmmlStr::from_str(Echo::NAME) };
        // SAFETY: the loader passes a writable AmmlModInfo, which is larger than the v1 block.
        unsafe { info.cast::<InfoV1>().write(block) };
        AmmlStatus::OK
    }

    #[allow(unsafe_code)]
    #[unsafe(no_mangle)]
    pub extern "C" fn amml_mod_enable() -> AmmlStatus {
        ModGlue::enable::<Echo>()
    }

    #[allow(unsafe_code)]
    #[unsafe(no_mangle)]
    pub extern "C" fn amml_mod_disable() -> AmmlStatus {
        ModGlue::disable::<Echo>()
    }

    #[allow(unsafe_code)]
    #[unsafe(no_mangle)]
    pub extern "C" fn amml_mod_unload() -> AmmlStatus {
        ModGlue::unload::<Echo>()
    }
}
