/// The part of a section header that maps an RVA to a file offset.
#[derive(Clone, Copy, Debug)]
pub struct SectionInfo {
    pub virtual_address: u32,
    pub virtual_size: u32,
    pub raw_pointer: u32,
    pub raw_size: u32,
}
