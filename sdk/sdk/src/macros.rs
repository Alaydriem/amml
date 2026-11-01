/// Generates the four exports of a mod for a type that implements `AmmlMod`.
///
/// ```ignore
/// struct MyMod;
/// impl amml_sdk::AmmlMod for MyMod { /* ... */ }
/// amml_sdk::amml_mod!(MyMod);
/// ```
#[macro_export]
macro_rules! amml_mod {
    ($mod_type:ty) => {
        /// # Safety
        /// Called by the AMML loader with its API table and a writable info block.
        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn amml_mod_load(
            api: *const $crate::amml_abi::AmmlApi,
            info: *mut $crate::amml_abi::AmmlModInfo,
        ) -> $crate::amml_abi::AmmlStatus {
            // SAFETY: the loader passes the pointers `ModGlue::load` documents.
            unsafe { $crate::ModGlue::load::<$mod_type>(api, info) }
        }

        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "C" fn amml_mod_enable() -> $crate::amml_abi::AmmlStatus {
            $crate::ModGlue::enable::<$mod_type>()
        }

        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "C" fn amml_mod_disable() -> $crate::amml_abi::AmmlStatus {
            $crate::ModGlue::disable::<$mod_type>()
        }

        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "C" fn amml_mod_unload() -> $crate::amml_abi::AmmlStatus {
            $crate::ModGlue::unload::<$mod_type>()
        }

        const _: $crate::amml_abi::ModLoadFn = amml_mod_load;
        const _: $crate::amml_abi::ModEnableFn = amml_mod_enable;
        const _: $crate::amml_abi::ModDisableFn = amml_mod_disable;
        const _: $crate::amml_abi::ModUnloadFn = amml_mod_unload;
    };
}

/// Generates a panic-safe detour in a module of the given name.
///
/// The module holds `GUARD` (give `GUARD.original()` to `Api::hook`) and `detour`
/// (`extern "C"`, give it to `Api::hook` as the detour). The body gets `original`, a closure that
/// calls the next function in the chain with the arguments it is given. A panic in the body is
/// logged once and disables the mod's detours. If the body had already called `original`, that
/// result is returned and the original does not run again; otherwise the original runs with the
/// same arguments. The return type must be `Copy`.
///
/// A game original that throws a C++ exception is not caught: the exception ends the process.
///
/// ```ignore
/// amml_sdk::amml_detour!(pub mod update(this: usize, dt: f32) -> () |original| {
///     original(this, dt * 2.0)
/// });
/// ```
#[macro_export]
macro_rules! amml_detour {
    ($vis:vis mod $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty |$orig:ident| $body:block) => {
        $vis mod $name {
            #[allow(unused_imports)]
            use super::*;

            const _: fn() = || {
                fn assert_copy<T: Copy>() {}
                assert_copy::<$ret>();
            };

            pub static GUARD: $crate::DetourGuard = $crate::DetourGuard::new(stringify!($name));

            pub extern "C" fn detour($($arg: $ty),*) -> $ret {
                let done: ::std::cell::Cell<::std::option::Option<$ret>> = ::std::cell::Cell::new(::std::option::Option::None);
                let call_original = |$($arg: $ty),*| -> $ret {
                    let next: extern "C" fn($($ty),*) -> $ret = GUARD.original_fn();
                    let result = next($($arg),*);
                    done.set(::std::option::Option::Some(result));
                    result
                };
                GUARD.run(
                    || {
                        let $orig = &call_original;
                        $body
                    },
                    || match done.take() {
                        ::std::option::Option::Some(result) => result,
                        ::std::option::Option::None => call_original($($arg),*),
                    },
                )
            }
        }
    };
}
