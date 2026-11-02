use amml_gdk::{GdkError, ProcessInspector};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Running processes by image path. Clones share the list.
#[derive(Clone, Default)]
pub struct FakeProcesses {
    running: Rc<RefCell<Vec<(u32, PathBuf)>>>,
}

impl FakeProcesses {
    pub fn new() -> FakeProcesses {
        FakeProcesses::default()
    }

    pub fn start(&self, pid: u32, image: PathBuf) {
        self.running.borrow_mut().push((pid, image));
    }
}

impl ProcessInspector for FakeProcesses {
    fn image_paths(&self, exe_name: &str) -> Result<Vec<(u32, PathBuf)>, GdkError> {
        Ok(self
            .running
            .borrow()
            .iter()
            .filter(|(_, image)| {
                // an empty path is a process whose image path could not be read
                image.as_os_str().is_empty()
                    || image.file_name().is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(exe_name))
            })
            .cloned()
            .collect())
    }
}
