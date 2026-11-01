use std::path::Path;

use hcl::expr::{Operation, UnaryOperator};
use hcl::{Body, Expression};

use crate::errors::DataError;

// Typed access to the attributes of one HCL body, with errors that name the file and entry.
pub(crate) struct BlockReader<'a> {
    path: &'a Path,
    entry: Option<&'a str>,
    context: String,
    body: &'a Body,
}

impl<'a> BlockReader<'a> {
    pub fn new(path: &'a Path, entry: Option<&'a str>, context: &str, body: &'a Body) -> Self {
        Self { path, entry, context: context.to_string(), body }
    }

    pub fn fail(&self, message: String) -> DataError {
        DataError::Parse {
            path: self.path.to_path_buf(),
            entry: self.entry.map(str::to_string),
            message: format!("{}: {message}", self.context),
        }
    }

    // Fails on the first attribute whose name is not in `allowed`.
    pub fn only(&self, allowed: &[&str]) -> Result<(), DataError> {
        match self.body.attributes().find(|attribute| !allowed.contains(&attribute.key())) {
            Some(attribute) => Err(self.fail(format!("unknown attribute `{}`", attribute.key()))),
            None => Ok(()),
        }
    }

    fn find(&self, name: &str) -> Option<&'a Expression> {
        self.body.attributes().find(|attribute| attribute.key() == name).map(|attribute| attribute.expr())
    }

    pub fn string(&self, name: &str) -> Result<String, DataError> {
        self.optional_string(name)?.ok_or_else(|| self.fail(format!("missing attribute `{name}`")))
    }

    pub fn optional_string(&self, name: &str) -> Result<Option<String>, DataError> {
        match self.find(name) {
            None => Ok(None),
            Some(Expression::String(text)) => Ok(Some(text.clone())),
            Some(_) => Err(self.fail(format!("attribute `{name}` must be a string"))),
        }
    }

    pub fn optional_bool(&self, name: &str) -> Result<Option<bool>, DataError> {
        match self.find(name) {
            None => Ok(None),
            Some(Expression::Bool(value)) => Ok(Some(*value)),
            Some(_) => Err(self.fail(format!("attribute `{name}` must be true or false"))),
        }
    }

    pub fn string_list(&self, name: &str) -> Result<Vec<String>, DataError> {
        match self.find(name) {
            None => Ok(Vec::new()),
            Some(Expression::Array(items)) => items
                .iter()
                .map(|item| match item {
                    Expression::String(text) => Ok(text.clone()),
                    _ => Err(self.fail(format!("attribute `{name}` must be a list of strings"))),
                })
                .collect(),
            Some(_) => Err(self.fail(format!("attribute `{name}` must be a list of strings"))),
        }
    }

    pub fn i64(&self, name: &str) -> Result<i64, DataError> {
        self.optional_i64(name)?.ok_or_else(|| self.fail(format!("missing attribute `{name}`")))
    }

    pub fn optional_i64(&self, name: &str) -> Result<Option<i64>, DataError> {
        match self.find(name) {
            None => Ok(None),
            Some(expression) => match Self::signed(expression) {
                Some(value) => Ok(Some(value)),
                None => Err(self.fail(format!("attribute `{name}` must be an integer"))),
            },
        }
    }

    pub fn u64(&self, name: &str) -> Result<u64, DataError> {
        match self.find(name) {
            None => Err(self.fail(format!("missing attribute `{name}`"))),
            Some(Expression::Number(number)) => {
                number.as_u64().ok_or_else(|| self.fail(format!("attribute `{name}` must be a non-negative integer")))
            }
            Some(_) => Err(self.fail(format!("attribute `{name}` must be a non-negative integer"))),
        }
    }

    pub fn u32(&self, name: &str) -> Result<u32, DataError> {
        self.optional_u32(name)?.ok_or_else(|| self.fail(format!("missing attribute `{name}`")))
    }

    pub fn optional_u32(&self, name: &str) -> Result<Option<u32>, DataError> {
        match self.find(name) {
            None => Ok(None),
            Some(Expression::Number(number)) => match number.as_u64().and_then(|value| u32::try_from(value).ok()) {
                Some(value) => Ok(Some(value)),
                None => Err(self.fail(format!("attribute `{name}` must fit in 32 bits"))),
            },
            Some(_) => Err(self.fail(format!("attribute `{name}` must be a non-negative integer"))),
        }
    }

    // Integers may carry a unary minus, which the parser keeps as an operation.
    fn signed(expression: &Expression) -> Option<i64> {
        match expression {
            Expression::Number(number) => number.as_i64(),
            Expression::Operation(operation) => match operation.as_ref() {
                Operation::Unary(unary) if unary.operator == UnaryOperator::Neg => {
                    Self::signed(&unary.expr).and_then(i64::checked_neg)
                }
                _ => None,
            },
            _ => None,
        }
    }
}
