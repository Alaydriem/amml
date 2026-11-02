/// What a CLI run prints and returns; the binary only forwards it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CliOutput {
    pub const CHANGED: i32 = 0;
    pub const FAILED: i32 = 1;
    pub const UNCHANGED: i32 = 10;

    pub fn with_code(code: i32, stdout: String) -> Self {
        Self { code, stdout, stderr: String::new() }
    }

    pub fn failure(message: &str) -> Self {
        Self { code: Self::FAILED, stdout: String::new(), stderr: format!("error: {message}\n") }
    }
}
