use amml_abi::{AmmlLogLevel, AmmlStr, LogFieldsFn};

/// Writes lines with fields to the mod's log through the loader. Changing parts of a line go in the
/// fields, not in the message.
#[derive(Clone, Copy)]
pub struct Logger {
    log: LogFieldsFn,
    module: &'static str,
}

impl Logger {
    pub(crate) fn new(log: LogFieldsFn, module: &'static str) -> Self {
        Self { log, module }
    }

    pub fn log(&self, level: AmmlLogLevel, message: &str, fields: &[(&str, &str)]) {
        let keys: Vec<AmmlStr> = fields.iter().map(|(key, _)| AmmlStr::from_str(key)).collect();
        let values: Vec<AmmlStr> = fields.iter().map(|(_, value)| AmmlStr::from_str(value)).collect();
        // SAFETY: every view and both arrays live for the call, and the loader copies what it keeps.
        #[allow(unsafe_code)]
        unsafe {
            (self.log)(
                AmmlStr::from_str(self.module),
                level,
                AmmlStr::from_str(message),
                keys.as_ptr(),
                values.as_ptr(),
                fields.len(),
            )
        }
    }

    pub fn debug(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(AmmlLogLevel::DEBUG, message, fields);
    }

    pub fn info(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(AmmlLogLevel::INFO, message, fields);
    }

    pub fn warn(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(AmmlLogLevel::WARN, message, fields);
    }

    pub fn error(&self, message: &str, fields: &[(&str, &str)]) {
        self.log(AmmlLogLevel::ERROR, message, fields);
    }
}
