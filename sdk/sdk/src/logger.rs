use amml_abi::AmmlLogLevel;

use crate::Api;

/// Writes lines to the log file of one module through the loader.
#[derive(Clone)]
pub struct Logger {
    api: Api,
    module: String,
}

impl Logger {
    pub fn new(api: &Api, module: &str) -> Self {
        Self { api: *api, module: module.to_string() }
    }

    pub fn log(&self, level: AmmlLogLevel, message: &str) {
        self.api.log(&self.module, level, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(AmmlLogLevel::DEBUG, message);
    }

    pub fn info(&self, message: &str) {
        self.log(AmmlLogLevel::INFO, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(AmmlLogLevel::WARN, message);
    }

    pub fn error(&self, message: &str) {
        self.log(AmmlLogLevel::ERROR, message);
    }
}
