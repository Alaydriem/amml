use crate::message::Message;

use super::{ArgValue, MessageRef, UserText};

/// Named placeholder values, in the order they were added.
#[derive(Debug, Clone, Default)]
pub struct Args {
    values: Vec<(&'static str, ArgValue)>,
}

impl Args {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn int(self, name: &'static str, value: i64) -> Self {
        self.with(name, ArgValue::Int(value))
    }

    pub fn uint(self, name: &'static str, value: u64) -> Self {
        self.with(name, ArgValue::UInt(value))
    }

    pub fn float(self, name: &'static str, value: f64, decimals: u8) -> Self {
        self.with(name, ArgValue::Float(value, decimals))
    }

    pub fn user(self, name: &'static str, value: UserText) -> Self {
        self.with(name, ArgValue::Text(value))
    }

    pub fn message(self, name: &'static str, value: impl Message) -> Self {
        self.with(name, ArgValue::Message(MessageRef::of(value)))
    }

    pub fn get(&self, name: &str) -> Option<&ArgValue> {
        self.values.iter().find(|(known, _)| *known == name).map(|(_, value)| value)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    fn with(mut self, name: &'static str, value: ArgValue) -> Self {
        self.values.push((name, value));
        self
    }
}
