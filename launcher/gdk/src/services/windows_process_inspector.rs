use super::ProcessInspector;
use crate::GdkError;
use crate::structs::OwnedHandle;
use std::path::PathBuf;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::PWSTR;

pub struct WindowsProcessInspector;

impl WindowsProcessInspector {
    pub fn new() -> WindowsProcessInspector {
        WindowsProcessInspector
    }

    fn windows_error(call: &'static str) -> impl Fn(windows::core::Error) -> GdkError {
        move |error| GdkError::Windows { call, text: error.to_string() }
    }

    #[allow(unsafe_code)]
    fn image_path(pid: u32) -> Option<PathBuf> {
        // SAFETY: OpenProcess takes plain values; a failure is returned as an error and ends the lookup for this pid.
        let handle = OwnedHandle::new(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?);
        let mut buffer = vec![0u16; 32_768];
        let mut size = buffer.len() as u32;
        // SAFETY: `buffer` holds `size` UTF-16 units and outlives the call; `handle` is open for the whole call.
        unsafe {
            QueryFullProcessImageNameW(handle.raw(), PROCESS_NAME_FORMAT(0), PWSTR(buffer.as_mut_ptr()), &mut size)
        }
        .ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..size as usize])))
    }
}

impl Default for WindowsProcessInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessInspector for WindowsProcessInspector {
    #[allow(unsafe_code)]
    fn image_paths(&self, exe_name: &str) -> Result<Vec<(u32, PathBuf)>, GdkError> {
        // SAFETY: the snapshot call takes plain values and returns an owned handle that `OwnedHandle` closes.
        let snapshot = OwnedHandle::new(
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
                .map_err(Self::windows_error("CreateToolhelp32Snapshot"))?,
        );
        let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut found = Vec::new();
        // SAFETY: `entry.dwSize` is set and `entry` outlives the call; `snapshot` is open.
        let mut more = unsafe { Process32FirstW(snapshot.raw(), &mut entry) }.is_ok();
        while more {
            let length = entry.szExeFile.iter().position(|&unit| unit == 0).unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
            if name.eq_ignore_ascii_case(exe_name) {
                // a process whose image path cannot be read is reported with an empty path
                let path = Self::image_path(entry.th32ProcessID).unwrap_or_default();
                found.push((entry.th32ProcessID, path));
            }
            // SAFETY: same invariants as the first call.
            more = unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_ok();
        }
        Ok(found)
    }
}
