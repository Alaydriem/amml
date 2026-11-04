use std::path::PathBuf;

// How to start one instance: the executable, the folder it runs in, its arguments and the extra environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub exe: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}
