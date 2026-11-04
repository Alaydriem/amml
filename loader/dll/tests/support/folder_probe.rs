use std::path::PathBuf;
use std::process::Command;

use amml::Library;

type KnownFolder = unsafe extern "C" fn(*mut u16, u32) -> u32;
type FolderPath = unsafe extern "C" fn(*mut u16) -> i32;
type Env = unsafe extern "C" fn(*const u16, *mut u16, u32) -> u32;
type EnvAppdata = unsafe extern "C" fn(*mut u16, u32) -> u32;
type TempPath = unsafe extern "C" fn(*mut u16, u32) -> u32;

/// The folder-probe fixture, built and loaded into the test process.
pub struct FolderProbe {
    library: Library,
}

impl FolderProbe {
    pub const MAX_PATH: usize = 260;

    /// Builds the fixture (a cdylib workspace member) and maps it into this process.
    pub fn load() -> Self {
        let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("folder-probe");
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .arg("build")
            .arg("-p")
            .arg("folder-probe")
            .arg("--target-dir")
            .arg(&target)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .unwrap();
        assert!(status.success(), "building the folder-probe fixture failed");
        let library = Library::open(&target.join("debug").join("folder_probe.dll")).unwrap();
        Self { library }
    }

    pub fn base(&self) -> usize {
        self.library.base()
    }

    pub fn known_folder_roaming(&self) -> String {
        self.known_folder("probe_known_folder_roaming")
    }

    pub fn known_folder_local(&self) -> String {
        self.known_folder("probe_known_folder_local")
    }

    pub fn folder_path_appdata(&self) -> (i32, String) {
        self.folder_path("probe_folder_path_appdata", None)
    }

    pub fn folder_path_personal(&self) -> (i32, String) {
        self.folder_path("probe_folder_path_personal", None)
    }

    /// Like `folder_path_appdata`, with the buffer pre-filled with `fill` so a write shows.
    pub fn folder_path_appdata_filled(&self, fill: u16) -> (i32, Vec<u16>) {
        let mut buffer = vec![fill; Self::MAX_PATH];
        // SAFETY: the export has this signature and `buffer` holds `MAX_PATH` units.
        let function = unsafe { self.library.symbol::<FolderPath>("probe_folder_path_appdata") }.unwrap();
        // SAFETY: as above.
        let status = unsafe { function(buffer.as_mut_ptr()) };
        (status, buffer)
    }

    /// `GetEnvironmentVariableW(name)` with a buffer of `cap` units pre-filled with `fill`.
    pub fn env(&self, name: &str, cap: u32, fill: u16) -> (u32, Vec<u16>) {
        let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut buffer = vec![fill; cap as usize];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds `cap` units.
        let function = unsafe { self.library.symbol::<Env>("probe_env") }.unwrap();
        // SAFETY: as above.
        let result = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), cap) };
        (result, buffer)
    }

    /// `GetEnvironmentVariableW("APPDATA", NULL, 0)`: the size query.
    pub fn env_size(&self) -> u32 {
        // SAFETY: the export has this signature.
        let function = unsafe { self.library.symbol::<unsafe extern "C" fn() -> u32>("probe_env_size") }.unwrap();
        // SAFETY: as above.
        unsafe { function() }
    }

    pub fn env_appdata(&self) -> String {
        let mut buffer = vec![0u16; 32768];
        // SAFETY: the export has this signature and `buffer` holds the stated units.
        let function = unsafe { self.library.symbol::<EnvAppdata>("probe_env_appdata") }.unwrap();
        // SAFETY: as above.
        let length = unsafe { function(buffer.as_mut_ptr(), buffer.len() as u32) };
        String::from_utf16_lossy(&buffer[..length as usize])
    }

    pub fn temp_path(&self) -> String {
        let mut buffer = vec![0u16; 1024];
        // SAFETY: the export has this signature and `buffer` holds the stated units.
        let function = unsafe { self.library.symbol::<TempPath>("probe_temp_path") }.unwrap();
        // SAFETY: as above.
        let length = unsafe { function(buffer.as_mut_ptr(), buffer.len() as u32) };
        String::from_utf16_lossy(&buffer[..length as usize])
    }

    fn known_folder(&self, export: &str) -> String {
        let mut buffer = vec![0u16; 32768];
        // SAFETY: the export has this signature and `buffer` holds the stated units.
        let function = unsafe { self.library.symbol::<KnownFolder>(export) }.unwrap();
        // SAFETY: as above.
        let length = unsafe { function(buffer.as_mut_ptr(), buffer.len() as u32) };
        String::from_utf16_lossy(&buffer[..length as usize])
    }

    fn folder_path(&self, export: &str, fill: Option<u16>) -> (i32, String) {
        let mut buffer = vec![fill.unwrap_or(0); Self::MAX_PATH];
        // SAFETY: the export has this signature and `buffer` holds `MAX_PATH` units.
        let function = unsafe { self.library.symbol::<FolderPath>(export) }.unwrap();
        // SAFETY: as above.
        let status = unsafe { function(buffer.as_mut_ptr()) };
        let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
        (status, String::from_utf16_lossy(&buffer[..end]))
    }

    /// `GetEnvironmentVariableA` through the probe: the result and the text.
    #[cfg(feature = "folder-trace")]
    pub fn env_a(&self, name: &str) -> (u32, String) {
        let name = Self::narrow_name(name);
        let mut buffer = vec![0u8; 32768];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds the stated bytes.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*const u8, *mut u8, u32) -> u32>("probe_env_a") }
                .unwrap();
        // SAFETY: as above.
        let length = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        (length, Self::narrow_text(&buffer))
    }

    /// `GetTempPathA` through the probe.
    #[cfg(feature = "folder-trace")]
    pub fn temp_path_a(&self) -> String {
        let mut buffer = vec![0u8; 1024];
        // SAFETY: the export has this signature and `buffer` holds the stated bytes.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*mut u8, u32) -> u32>("probe_temp_path_a") }.unwrap();
        // SAFETY: as above.
        unsafe { function(buffer.as_mut_ptr(), buffer.len() as u32) };
        Self::narrow_text(&buffer)
    }

    /// `getenv` through the probe: `None` when the variable is unset.
    #[cfg(feature = "folder-trace")]
    pub fn getenv(&self, name: &str) -> Option<String> {
        self.narrow_probe("probe_getenv", name).map(|(length, text)| {
            assert_eq!(length as usize, text.len());
            text
        })
    }

    /// `getenv_s` through the probe: the errno result and the text.
    #[cfg(feature = "folder-trace")]
    pub fn getenv_s(&self, name: &str) -> (i32, String) {
        let name = Self::narrow_name(name);
        let mut buffer = vec![0u8; 32768];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds the stated bytes.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*const u8, *mut u8, u32) -> i32>("probe_getenv_s") }
                .unwrap();
        // SAFETY: as above.
        let status = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        (status, Self::narrow_text(&buffer))
    }

    /// `_dupenv_s` through the probe: the errno result and the text.
    #[cfg(feature = "folder-trace")]
    pub fn dupenv_s(&self, name: &str) -> (i32, String) {
        let name = Self::narrow_name(name);
        let mut buffer = vec![0u8; 32768];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds the stated bytes.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*const u8, *mut u8, u32) -> i32>("probe_dupenv_s") }
                .unwrap();
        // SAFETY: as above.
        let status = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        (status, Self::narrow_text(&buffer))
    }

    /// `_wdupenv_s` through the probe: the errno result and the text.
    #[cfg(feature = "folder-trace")]
    pub fn wdupenv_s(&self, name: &str) -> (i32, String) {
        let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut buffer = vec![0u16; 32768];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds the stated units.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*const u16, *mut u16, u32) -> i32>("probe_wdupenv_s") }
                .unwrap();
        // SAFETY: as above.
        let status = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
        (status, String::from_utf16_lossy(&buffer[..end]))
    }

    #[cfg(feature = "folder-trace")]
    fn narrow_probe(&self, export: &str, name: &str) -> Option<(u32, String)> {
        let name = Self::narrow_name(name);
        let mut buffer = vec![0u8; 32768];
        // SAFETY: the export has this signature; `name` is NUL-terminated and `buffer` holds the stated bytes.
        let function =
            unsafe { self.library.symbol::<unsafe extern "C" fn(*const u8, *mut u8, u32) -> u32>(export) }.unwrap();
        // SAFETY: as above.
        let length = unsafe { function(name.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        (length != u32::MAX).then(|| (length, Self::narrow_text(&buffer)))
    }

    #[cfg(feature = "folder-trace")]
    fn narrow_name(name: &str) -> Vec<u8> {
        name.bytes().chain(std::iter::once(0)).collect()
    }

    #[cfg(feature = "folder-trace")]
    fn narrow_text(buffer: &[u8]) -> String {
        let end = buffer.iter().position(|byte| *byte == 0).unwrap_or(buffer.len());
        String::from_utf8_lossy(&buffer[..end]).into_owned()
    }
}
