use std::path::PathBuf;

/// The PE files the patcher tests start from.
pub struct PeFixture;

impl PeFixture {
    pub fn small_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("derive")
            .join("cli")
            .join("tests")
            .join("fixtures")
            .join("pe")
            .join("small.dll")
    }

    pub fn small() -> Vec<u8> {
        std::fs::read(Self::small_path()).unwrap()
    }
}
