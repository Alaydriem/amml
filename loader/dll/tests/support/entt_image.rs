use std::ffi::c_void;

/// The null entity of a sparse page slot.
pub const NULL_SLOT: u32 = u32::MAX;

/// A storage image the entt lookup reads: the sparse page vector at 08h and the payload page vector at 50h. The page
/// vectors are kept so that the addresses in the image stay valid.
#[allow(dead_code)]
pub struct FakeStorage {
    pub image: Vec<u64>,
    pub component_size: usize,
    pub sparse: Vec<Vec<u32>>,
    pub sparse_pointers: Vec<*const u32>,
    pub payload: Vec<Vec<u8>>,
    pub payload_pointers: Vec<*const u8>,
}

// SAFETY: the image and its pages are owned by the value and read only on the test's own threads under its locks.
unsafe impl Send for FakeStorage {}

impl FakeStorage {
    /// One sparse page per entry of `sparse_pages` and one payload page of `component_size` components per entry of
    /// `payload_pages`; a `false` entry is a null page. Every sparse slot is the null entity.
    pub fn new(sparse_pages: &[bool], payload_pages: &[bool], component_size: usize) -> Self {
        let sparse: Vec<Vec<u32>> = sparse_pages.iter().map(|_| vec![NULL_SLOT; 2048]).collect();
        let sparse_pointers: Vec<*const u32> = sparse_pages
            .iter()
            .zip(&sparse)
            .map(|(present, page)| if *present { page.as_ptr() } else { std::ptr::null() })
            .collect();
        let payload: Vec<Vec<u8>> = payload_pages.iter().map(|_| vec![0u8; 128 * component_size]).collect();
        let payload_pointers: Vec<*const u8> = payload_pages
            .iter()
            .zip(&payload)
            .map(|(present, page)| if *present { page.as_ptr() } else { std::ptr::null() })
            .collect();
        let mut image = vec![0u64; 0x68 / 8];
        let sparse_first = sparse_pointers.as_ptr() as u64;
        let sparse_end = sparse_first + (sparse_pointers.len() * 8) as u64;
        image[1] = sparse_first;
        image[2] = sparse_end;
        image[3] = sparse_end;
        let payload_first = payload_pointers.as_ptr() as u64;
        let payload_end = payload_first + (payload_pointers.len() * 8) as u64;
        image[0x50 / 8] = payload_first;
        image[0x58 / 8] = payload_end;
        image[0x60 / 8] = payload_end;
        Self { image, component_size, sparse, sparse_pointers, payload, payload_pointers }
    }

    pub fn address(&self) -> *const c_void {
        self.image.as_ptr().cast()
    }

    /// Stores `entity` in the sparse set at `packed`.
    pub fn insert(&mut self, entity: u32, packed: u32) {
        let index = entity & 0x3_FFFF;
        self.sparse[(index >> 11) as usize][(index & 0x7FF) as usize] = (entity & !0x3_FFFF) | packed;
    }

    /// The bytes of the component at `packed`.
    pub fn component_mut(&mut self, packed: u32) -> &mut [u8] {
        let size = self.component_size;
        let at = (packed & 0x7F) as usize * size;
        &mut self.payload[(packed >> 7) as usize][at..at + size]
    }
}

/// A registry image: the bucket vector at 48h, the node vector at 68h (20h-byte nodes: next index, type id, pool
/// pointer).
#[allow(dead_code)]
pub struct FakeRegistry {
    pub image: Vec<u64>,
    pub buckets: Vec<u64>,
    pub nodes: Vec<[u64; 4]>,
}

// SAFETY: as `FakeStorage`.
unsafe impl Send for FakeRegistry {}

impl FakeRegistry {
    pub fn new(pools: &[(u32, &FakeStorage)], bucket_count: usize) -> Self {
        let mut buckets = vec![u64::MAX; bucket_count];
        let mut nodes: Vec<[u64; 4]> = Vec::new();
        for (index, (type_id, storage)) in pools.iter().enumerate() {
            let bucket = *type_id as usize & (bucket_count - 1);
            nodes.push([buckets[bucket], u64::from(*type_id), storage.address() as u64, 0]);
            buckets[bucket] = index as u64;
        }
        let mut registry = Self { image: vec![0u64; 0x78 / 8], buckets, nodes };
        registry.point();
        registry
    }

    /// Points the image at the bucket and node vectors as they are now.
    pub fn point(&mut self) {
        let buckets_first = self.buckets.as_ptr() as u64;
        let buckets_end = buckets_first + (self.buckets.len() * 8) as u64;
        self.image[0x48 / 8] = buckets_first;
        self.image[0x50 / 8] = buckets_end;
        self.image[0x58 / 8] = buckets_end;
        let nodes_first = self.nodes.as_ptr() as u64;
        let nodes_end = nodes_first + (self.nodes.len() * 0x20) as u64;
        self.image[0x68 / 8] = nodes_first;
        self.image[0x70 / 8] = nodes_end;
    }

    pub fn address(&self) -> *const c_void {
        self.image.as_ptr().cast()
    }
}
