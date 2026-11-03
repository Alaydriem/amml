use crate::LauncherError;
use amml_gdk::{AmmlPaths, GdkError, InstanceService};
use std::path::PathBuf;

// The folders the launcher core adds to the ones `AmmlPaths` names.
#[derive(Debug, Clone)]
pub struct LauncherPaths {
    gdk: AmmlPaths,
}

impl LauncherPaths {
    pub fn new(gdk: AmmlPaths) -> LauncherPaths {
        LauncherPaths { gdk }
    }

    pub fn gdk(&self) -> &AmmlPaths {
        &self.gdk
    }

    pub fn settings(&self) -> PathBuf {
        self.gdk.root().join("launcher").join("settings.hcl")
    }

    pub fn registry_cache(&self) -> PathBuf {
        self.gdk.root().join("launcher").join("cache").join("registries")
    }

    pub fn default_data_dir(&self) -> PathBuf {
        self.gdk.root().join("data")
    }

    pub fn instance(&self, name: &str) -> PathBuf {
        self.gdk.instances().join(name)
    }

    // `instances\<name>` for a plain instance name: no separator, nothing the instance service refuses, and no leading
    // dot (those folders are hidden from the instance list)
    pub fn checked_instance(&self, name: &str) -> Result<PathBuf, LauncherError> {
        InstanceService::check_name(name)?;
        if name.starts_with('.') {
            return Err(GdkError::BadInstanceName(name.to_string()).into());
        }
        Ok(self.instance(name))
    }
}
