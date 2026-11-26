// The layout of the MSVC `std::string` the router takes: the small buffer (or the heap pointer in its first word),
// the size and the capacity; `heap` keeps a long text alive.
#[repr(C)]
pub struct RouteText {
    storage: [u8; 16],
    size: usize,
    capacity: usize,
    heap: Vec<u8>,
}

impl RouteText {
    const SMALL: usize = 15;

    pub fn new(text: &str) -> Self {
        let mut route = Self { storage: [0; 16], size: text.len(), capacity: Self::SMALL, heap: Vec::new() };
        if text.len() <= Self::SMALL {
            route.storage[..text.len()].copy_from_slice(text.as_bytes());
        } else {
            route.heap = text.as_bytes().to_vec();
            route.storage[..8].copy_from_slice(&(route.heap.as_ptr() as usize).to_le_bytes());
            route.capacity = text.len();
        }
        route
    }

    pub fn read(route: *const RouteText) -> String {
        // SAFETY: the callers pass a live text.
        let route = unsafe { &*route };
        let bytes = if route.capacity > Self::SMALL {
            let data = usize::from_le_bytes(route.storage[..8].try_into().unwrap()) as *const u8;
            // SAFETY: a long text points to its  bytes.
            unsafe { std::slice::from_raw_parts(data, route.size) }
        } else {
            &route.storage[..route.size]
        };
        String::from_utf8_lossy(bytes).into_owned()
    }
}
