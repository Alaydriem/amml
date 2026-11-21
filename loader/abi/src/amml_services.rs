use crate::{
    AmmlActorTable, AmmlCameraTable, AmmlClientTable, AmmlCoreTable, AmmlEventsTable, AmmlInputTable,
    AmmlInventoryTable, AmmlLightingTable, AmmlNetworkTable, AmmlOverlayTable, AmmlRegistryTable, AmmlRenderTable,
    AmmlUiTable, AmmlWindowTable, AmmlWorldTable,
};

/// The service tables of API v3, reached through `AmmlApi::services`.
///
/// It only grows: a table added later is appended, and a reader treats a table that `size` does not
/// cover, or a null pointer, as an area the loader does not have (`NOT_AVAILABLE` when it is used),
/// never as a reason to refuse the whole table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlServices {
    pub size: u32,
    pub version: u32,
    pub events: *const AmmlEventsTable,
    pub core: *const AmmlCoreTable,
    pub registry: *const AmmlRegistryTable,
    pub client: *const AmmlClientTable,
    pub input: *const AmmlInputTable,
    pub window: *const AmmlWindowTable,
    pub overlay: *const AmmlOverlayTable,
    pub actor: *const AmmlActorTable,
    pub inventory: *const AmmlInventoryTable,
    pub lighting: *const AmmlLightingTable,
    pub ui: *const AmmlUiTable,
    pub network: *const AmmlNetworkTable,
    pub world: *const AmmlWorldTable,
    pub render: *const AmmlRenderTable,
    pub camera: *const AmmlCameraTable,
}

impl AmmlServices {
    /// `size` of a v1 services table. A literal, so it keeps its meaning when a later version adds
    /// fields: a reader accepts any `size` at or above it and reads a later field only when `size`
    /// covers it.
    pub const SIZE_V1: u32 = 32;
    /// `size` of a v2 services table: v1 plus the `client` and `input` tables. Its `version` is 2. A reader
    /// still judges the table by `size`, never by `version`.
    pub const SIZE_V2: u32 = 48;
    /// `size` of a v3 services table: v2 plus the `window` and `overlay` tables. Its `version` is 3. A reader
    /// still judges the table by `size`, never by `version`.
    pub const SIZE_V3: u32 = 64;
    /// `size` of a v4 services table: v3 plus the `actor` table. Its `version` is 4. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V4: u32 = 72;
    /// `size` of a v5 services table: v4 plus the `inventory` table. Its `version` is 5. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V5: u32 = 80;
    /// `size` of a v6 services table: v5 plus the `lighting` table. Its `version` is 6. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V6: u32 = 88;
    /// `size` of a v7 services table: v6 plus the `ui` table. Its `version` is 7. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V7: u32 = 96;
    /// `size` of a v8 services table: v7 plus the `network` table. Its `version` is 8. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V8: u32 = 104;
    /// `size` of a v9 services table: v8 plus the `world` table. Its `version` is 9. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V9: u32 = 112;
    /// `size` of a v10 services table: v9 plus the `render` table. Its `version` is 10. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V10: u32 = 120;
    /// `size` of a v11 services table: v10 plus the `camera` table. Its `version` is 11. A reader still judges the
    /// table by `size`, never by `version`.
    pub const SIZE_V11: u32 = 128;

    /// The client table of `services`, when `size` covers it, the pointer is not null and the table's
    /// own `size` covers its v1 functions. Reads only the fields `size` covers.
    ///
    /// # Safety
    ///
    /// `services` points to a table whose first `size` bytes are readable, and a non-null `client`
    /// points to a readable table of at least its own `size` bytes.
    #[allow(unsafe_code)]
    pub unsafe fn client(services: *const AmmlServices) -> Option<*const AmmlClientTable> {
        // SAFETY: `area` calls this only when `size` covers the `client` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).client).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v2.
        unsafe { Self::area(services, Self::SIZE_V2, AmmlClientTable::SIZE_V1, read) }
    }

    /// The input table of `services`, under the same rules as `client`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `input` table.
    #[allow(unsafe_code)]
    pub unsafe fn input(services: *const AmmlServices) -> Option<*const AmmlInputTable> {
        // SAFETY: `area` calls this only when `size` covers the `input` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).input).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v2.
        unsafe { Self::area(services, Self::SIZE_V2, AmmlInputTable::SIZE_V1, read) }
    }

    /// The window table of `services`, under the same rules as `client`; it needs a v3 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `window` table.
    #[allow(unsafe_code)]
    pub unsafe fn window(services: *const AmmlServices) -> Option<*const AmmlWindowTable> {
        // SAFETY: `area` calls this only when `size` covers the `window` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).window).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v3.
        unsafe { Self::area(services, Self::SIZE_V3, AmmlWindowTable::SIZE_V1, read) }
    }

    /// The overlay table of `services`, under the same rules as `client`; it needs a v3 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `overlay` table.
    #[allow(unsafe_code)]
    pub unsafe fn overlay(services: *const AmmlServices) -> Option<*const AmmlOverlayTable> {
        // SAFETY: `area` calls this only when `size` covers the `overlay` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).overlay).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v3.
        unsafe { Self::area(services, Self::SIZE_V3, AmmlOverlayTable::SIZE_V1, read) }
    }

    /// The actor table of `services`, under the same rules as `client`; it needs a v4 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `actor` table.
    #[allow(unsafe_code)]
    pub unsafe fn actor(services: *const AmmlServices) -> Option<*const AmmlActorTable> {
        // SAFETY: `area` calls this only when `size` covers the `actor` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).actor).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v4.
        unsafe { Self::area(services, Self::SIZE_V4, AmmlActorTable::SIZE_V1, read) }
    }

    /// The inventory table of `services`, under the same rules as `client`; it needs a v5 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `inventory` table.
    #[allow(unsafe_code)]
    pub unsafe fn inventory(services: *const AmmlServices) -> Option<*const AmmlInventoryTable> {
        // SAFETY: `area` calls this only when `size` covers the `inventory` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).inventory).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v5.
        unsafe { Self::area(services, Self::SIZE_V5, AmmlInventoryTable::SIZE_V1, read) }
    }

    /// The lighting table of `services`, under the same rules as `client`; it needs a v6 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `lighting` table.
    #[allow(unsafe_code)]
    pub unsafe fn lighting(services: *const AmmlServices) -> Option<*const AmmlLightingTable> {
        // SAFETY: `area` calls this only when `size` covers the `lighting` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).lighting).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v6.
        unsafe { Self::area(services, Self::SIZE_V6, AmmlLightingTable::SIZE_V1, read) }
    }

    /// The ui table of `services`, under the same rules as `client`; it needs a v7 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `ui` table.
    #[allow(unsafe_code)]
    pub unsafe fn ui(services: *const AmmlServices) -> Option<*const AmmlUiTable> {
        // SAFETY: `area` calls this only when `size` covers the `ui` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).ui).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v7.
        unsafe { Self::area(services, Self::SIZE_V7, AmmlUiTable::SIZE_V1, read) }
    }

    /// The network table of `services`, under the same rules as `client`; it needs a v8 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `network` table.
    #[allow(unsafe_code)]
    pub unsafe fn network(services: *const AmmlServices) -> Option<*const AmmlNetworkTable> {
        // SAFETY: `area` calls this only when `size` covers the `network` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).network).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v8.
        unsafe { Self::area(services, Self::SIZE_V8, AmmlNetworkTable::SIZE_V1, read) }
    }

    /// The world table of `services`, under the same rules as `client`; it needs a v9 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `world` table.
    #[allow(unsafe_code)]
    pub unsafe fn world(services: *const AmmlServices) -> Option<*const AmmlWorldTable> {
        // SAFETY: `area` calls this only when `size` covers the `world` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).world).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v9.
        unsafe { Self::area(services, Self::SIZE_V9, AmmlWorldTable::SIZE_V1, read) }
    }

    /// The render table of `services`, under the same rules as `client`; it needs a v10 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `render` table.
    #[allow(unsafe_code)]
    pub unsafe fn render(services: *const AmmlServices) -> Option<*const AmmlRenderTable> {
        // SAFETY: `area` calls this only when `size` covers the `render` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).render).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v10.
        unsafe { Self::area(services, Self::SIZE_V10, AmmlRenderTable::SIZE_V1, read) }
    }

    /// The camera table of `services`, under the same rules as `client`; it needs a v11 `size`.
    ///
    /// # Safety
    ///
    /// As for `client`, for the `camera` table.
    #[allow(unsafe_code)]
    pub unsafe fn camera(services: *const AmmlServices) -> Option<*const AmmlCameraTable> {
        // SAFETY: `area` calls this only when `size` covers the `camera` field.
        let read = |services: *const AmmlServices| unsafe { (&raw const (*services).camera).read() };
        // SAFETY: the caller's guarantee is passed on; `read` runs only when `size` covers v11.
        unsafe { Self::area(services, Self::SIZE_V11, AmmlCameraTable::SIZE_V1, read) }
    }

    // The area table that `read` gives, when `size` is at least `needed`, the pointer is not null and the
    // table's own `size` is at least `table_size`.
    //
    // Safety: `services` is readable for its first `size` bytes; `read` reads one field that `needed`
    // covers; a non-null table pointer points to a table readable for its own `size` bytes.
    #[allow(unsafe_code)]
    unsafe fn area<T>(
        services: *const AmmlServices,
        needed: u32,
        table_size: u32,
        read: impl FnOnce(*const AmmlServices) -> *const T,
    ) -> Option<*const T> {
        // SAFETY: the caller guarantees `services` is readable for `size` bytes, and the first four
        // bytes hold `size`.
        let size = unsafe { services.cast::<u32>().read() };
        if size < needed {
            return None;
        }
        let table = read(services);
        if table.is_null() {
            return None;
        }
        // SAFETY: the table is non-null and the caller guarantees it is readable; its first four bytes hold
        // its `size`.
        let own_size = unsafe { table.cast::<u32>().read() };
        (own_size >= table_size).then_some(table)
    }
}

const _: () = assert!(core::mem::size_of::<AmmlServices>() == AmmlServices::SIZE_V11 as usize);
