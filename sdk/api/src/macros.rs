/// Generates the four exports of a v3 mod for a type that implements `AmmlMod`. `amml_mod_load`
/// writes a v3 info block whose `uses` are the rows of `AmmlMod::USES`.
///
/// ```ignore
/// struct MyMod;
/// impl amml_api::AmmlMod for MyMod { /* ... */ }
/// amml_api::amml_mod!(MyMod);
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

/// Builds the `Uses` of a mod: per feature, the events and areas it needs (`required`) or can do
/// without (`optional`). An area row is written `required area <Area>;`. Give the result to
/// `AmmlMod::USES`.
///
/// ```ignore
/// const USES: amml_api::Uses = amml_api::amml_uses! {
///     feature "core" { required ClientTick; optional GameTick; }
///     feature "input" { required area amml_api::Area::KEY_INJECT; optional area amml_api::Area::TURN; }
/// };
/// ```
#[macro_export]
macro_rules! amml_uses {
    (@rows [$($rows:tt)*]) => {
        $crate::Uses::new(&[$($rows)*])
    };
    (@rows [$($rows:tt)*] feature $feature:literal { $($body:tt)* } $($rest:tt)*) => {
        $crate::amml_uses!(@body [$($rows)*] $feature { $($body)* } $($rest)*)
    };
    (@body [$($rows:tt)*] $feature:literal { } $($rest:tt)*) => {
        $crate::amml_uses!(@rows [$($rows)*] $($rest)*)
    };
    (@body [$($rows:tt)*] $feature:literal { $need:ident area $area:expr; $($more:tt)* } $($rest:tt)*) => {
        $crate::amml_uses!(
            @body [$($rows)* $crate::amml_uses!(@area $feature, $need, $area),] $feature { $($more)* } $($rest)*
        )
    };
    (@body [$($rows:tt)*] $feature:literal { $need:ident $event:ty; $($more:tt)* } $($rest:tt)*) => {
        $crate::amml_uses!(
            @body [$($rows)* $crate::amml_uses!(@row $feature, $need, $event),] $feature { $($more)* } $($rest)*
        )
    };
    (@area $feature:literal, required, $area:expr) => {
        $crate::Uses::area_row($feature, ($area).id, ($area).version, true)
    };
    (@area $feature:literal, optional, $area:expr) => {
        $crate::Uses::area_row($feature, ($area).id, ($area).version, false)
    };
    (@area $feature:literal, $other:ident, $area:expr) => {
        ::core::compile_error!("amml_uses!: write `required` or `optional` before each area")
    };
    (@row $feature:literal, required, $event:ty) => {
        $crate::Uses::event_row($feature, <$event as $crate::Event>::ID, <$event as $crate::Event>::VERSION, true)
    };
    (@row $feature:literal, optional, $event:ty) => {
        $crate::Uses::event_row($feature, <$event as $crate::Event>::ID, <$event as $crate::Event>::VERSION, false)
    };
    (@row $feature:literal, $other:ident, $event:ty) => {
        ::core::compile_error!("amml_uses!: write `required` or `optional` before each event")
    };
    ($(feature $feature:literal { $($body:tt)* })*) => {
        $crate::amml_uses!(@rows [] $(feature $feature { $($body)* })*)
    };
}
