use crate::error::ToolError;

/// Evaluates the C expression in a gettext `Plural-Forms` header.
///
/// The grammar is smaller than C's on purpose: no arithmetic beyond `%`, no unary operators,
/// no identifiers other than `n`. Expressions come from .po files that outside people write,
/// so anything the grammar cannot express is rejected rather than interpreted.
pub struct PluralExpression {
    source: String,
    tokens: Vec<String>,
    position: usize,
    n: u64,
}

impl PluralExpression {
    const TWO_CHAR: [&'static str; 6] = ["||", "&&", "<=", ">=", "==", "!="];
    const ONE_CHAR: [char; 7] = ['<', '>', '%', '?', ':', '(', ')'];

    pub fn evaluate(source: &str, n: u64) -> Result<u64, ToolError> {
        let trimmed = source.trim().trim_end_matches(';');
        let mut state = Self { source: source.to_owned(), tokens: Vec::new(), position: 0, n };
        state.tokens = state.tokenize(trimmed)?;
        let value = state.ternary()?;
        if state.position != state.tokens.len() {
            return Err(state.error("trailing tokens"));
        }
        Ok(value)
    }

    fn tokenize(&self, text: &str) -> Result<Vec<String>, ToolError> {
        let mut tokens = Vec::new();
        let mut rest = text.trim_start();
        while let Some(first) = rest.chars().next() {
            let length = if first.is_ascii_digit() {
                rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len())
            } else if Self::TWO_CHAR.iter().any(|op| rest.starts_with(op)) {
                2
            } else if first == 'n' || Self::ONE_CHAR.contains(&first) {
                1
            } else {
                return Err(self.error(&format!("illegal character {first:?}")));
            };
            tokens.push(rest[..length].to_owned());
            rest = rest[length..].trim_start();
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }

    fn ternary(&mut self) -> Result<u64, ToolError> {
        let condition = self.or()?;
        if self.peek() != Some("?") {
            return Ok(condition);
        }
        self.position += 1;
        let when_true = self.ternary()?;
        self.expect(":")?;
        let when_false = self.ternary()?;
        Ok(if condition != 0 { when_true } else { when_false })
    }

    fn or(&mut self) -> Result<u64, ToolError> {
        let mut left = self.and()?;
        while self.peek() == Some("||") {
            self.position += 1;
            let right = self.and()?;
            left = u64::from(left != 0 || right != 0);
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<u64, ToolError> {
        let mut left = self.equality()?;
        while self.peek() == Some("&&") {
            self.position += 1;
            let right = self.equality()?;
            left = u64::from(left != 0 && right != 0);
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<u64, ToolError> {
        let mut left = self.relational()?;
        while let Some(operator @ ("==" | "!=")) = self.peek() {
            let equal = operator == "==";
            self.position += 1;
            let right = self.relational()?;
            left = u64::from((left == right) == equal);
        }
        Ok(left)
    }

    fn relational(&mut self) -> Result<u64, ToolError> {
        let mut left = self.modulo()?;
        while let Some(operator @ ("<" | ">" | "<=" | ">=")) = self.peek() {
            let operator = operator.to_owned();
            self.position += 1;
            let right = self.modulo()?;
            left = u64::from(match operator.as_str() {
                "<" => left < right,
                ">" => left > right,
                "<=" => left <= right,
                _ => left >= right,
            });
        }
        Ok(left)
    }

    fn modulo(&mut self) -> Result<u64, ToolError> {
        let mut left = self.primary()?;
        while self.peek() == Some("%") {
            self.position += 1;
            let right = self.primary()?;
            left = left.checked_rem(right).ok_or_else(|| self.error("modulo by zero"))?;
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<u64, ToolError> {
        let token = self.peek().map(str::to_owned).ok_or_else(|| self.error("unexpected end"))?;
        self.position += 1;
        if token == "(" {
            let value = self.ternary()?;
            self.expect(")")?;
            return Ok(value);
        }
        if token == "n" {
            return Ok(self.n);
        }
        token.parse::<u64>().map_err(|_| self.error(&format!("unexpected token {token:?}")))
    }

    fn expect(&mut self, token: &str) -> Result<(), ToolError> {
        if self.peek() != Some(token) {
            return Err(self.error(&format!("expected {token:?}")));
        }
        self.position += 1;
        Ok(())
    }

    fn error(&self, message: &str) -> ToolError {
        ToolError::PluralExpression { source_text: self.source.clone(), message: message.to_owned() }
    }
}
