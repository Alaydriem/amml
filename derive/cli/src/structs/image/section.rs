#[derive(Clone, Debug)]
pub struct Section {
    pub name: String,
    pub rva: u32,
    pub data: Vec<u8>,
    pub executable: bool,
}

impl Section {
    pub fn new(name: String, rva: u32, data: Vec<u8>, executable: bool) -> Self {
        Self { name, rva, data, executable }
    }

    pub fn contains(&self, rva: u32) -> bool {
        rva >= self.rva && ((rva - self.rva) as usize) < self.data.len()
    }
}
