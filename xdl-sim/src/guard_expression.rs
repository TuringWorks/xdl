// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Small, deterministic expression language for state-machine guards.

use std::collections::{BTreeMap, BTreeSet};

const MAX_GUARD_EXPRESSION_BYTES: usize = 16_384;
const MAX_GUARD_EXPRESSION_TOKENS: usize = 1_024;

#[derive(Debug, Clone)]
pub(crate) struct GuardExpression {
    source: String,
    root: Expression,
    variables: BTreeSet<String>,
}

#[derive(Debug, Clone)]
enum Expression {
    Number(f64),
    Variable(String),
    UnaryNot(Box<Self>),
    UnaryNegate(Box<Self>),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
    Less(Box<Self>, Box<Self>),
    LessEqual(Box<Self>, Box<Self>),
    Greater(Box<Self>, Box<Self>),
    GreaterEqual(Box<Self>, Box<Self>),
    Equal(Box<Self>, Box<Self>),
    NotEqual(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl GuardExpression {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        if source.len() > MAX_GUARD_EXPRESSION_BYTES {
            return Err(format!(
                "guard expression exceeds the {MAX_GUARD_EXPRESSION_BYTES}-byte limit"
            ));
        }
        let tokens = tokenize(source)?;
        if tokens.len() > MAX_GUARD_EXPRESSION_TOKENS {
            return Err(format!(
                "guard expression exceeds the {MAX_GUARD_EXPRESSION_TOKENS}-token limit"
            ));
        }
        let mut parser = Parser {
            tokens,
            position: 0,
            variables: BTreeSet::new(),
        };
        let root = parser.parse_or()?;
        if !matches!(parser.peek(), Token::End) {
            return Err(format!(
                "unexpected token {} in guard expression",
                parser.peek()
            ));
        }
        if source.trim().is_empty() {
            return Err("guard expression must not be empty".to_string());
        }
        Ok(Self {
            source: source.to_string(),
            root,
            variables: parser.variables,
        })
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn variables(&self) -> &BTreeSet<String> {
        &self.variables
    }

    pub(crate) fn evaluate(&self, values: &BTreeMap<String, f64>) -> Result<bool, String> {
        self.root.evaluate(values).map(|value| value != 0.0)
    }
}

impl Expression {
    fn evaluate(&self, values: &BTreeMap<String, f64>) -> Result<f64, String> {
        let boolean = |value: bool| f64::from(u8::from(value));
        let value = match self {
            Self::Number(value) => *value,
            Self::Variable(name) => {
                let value = values
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("guard input {name} is not configured"))?;
                if !value.is_finite() {
                    return Err(format!("guard input {name} must be finite"));
                }
                value
            }
            Self::UnaryNot(expression) => boolean(expression.evaluate(values)? == 0.0),
            Self::UnaryNegate(expression) => -expression.evaluate(values)?,
            Self::Add(left, right) => left.evaluate(values)? + right.evaluate(values)?,
            Self::Subtract(left, right) => left.evaluate(values)? - right.evaluate(values)?,
            Self::Multiply(left, right) => left.evaluate(values)? * right.evaluate(values)?,
            Self::Divide(left, right) => {
                let denominator = right.evaluate(values)?;
                if denominator == 0.0 {
                    return Err("division by zero in guard expression".to_string());
                }
                left.evaluate(values)? / denominator
            }
            Self::Less(left, right) => boolean(left.evaluate(values)? < right.evaluate(values)?),
            Self::LessEqual(left, right) => {
                boolean(left.evaluate(values)? <= right.evaluate(values)?)
            }
            Self::Greater(left, right) => boolean(left.evaluate(values)? > right.evaluate(values)?),
            Self::GreaterEqual(left, right) => {
                boolean(left.evaluate(values)? >= right.evaluate(values)?)
            }
            Self::Equal(left, right) => boolean(left.evaluate(values)? == right.evaluate(values)?),
            Self::NotEqual(left, right) => {
                boolean(left.evaluate(values)? != right.evaluate(values)?)
            }
            Self::And(left, right) => {
                if left.evaluate(values)? == 0.0 {
                    0.0
                } else {
                    boolean(right.evaluate(values)? != 0.0)
                }
            }
            Self::Or(left, right) => {
                if left.evaluate(values)? != 0.0 {
                    1.0
                } else {
                    boolean(right.evaluate(values)? != 0.0)
                }
            }
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err("guard expression produced a non-finite value".to_string())
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Identifier(String),
    Symbol(&'static str),
    End,
}

impl std::fmt::Display for Token {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(number) => write!(formatter, "{number}"),
            Self::Identifier(identifier) => formatter.write_str(identifier),
            Self::Symbol(symbol) => formatter.write_str(symbol),
            Self::End => formatter.write_str("<end>"),
        }
    }
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
        } else if bytes[index].is_ascii_digit()
            || (bytes[index] == b'.' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
        {
            let start = index;
            index += 1;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            if bytes.get(index) == Some(&b'.') {
                index += 1;
                while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                    index += 1;
                }
            }
            if matches!(bytes.get(index), Some(b'e' | b'E')) {
                index += 1;
                if matches!(bytes.get(index), Some(b'+' | b'-')) {
                    index += 1;
                }
                let exponent_start = index;
                while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                    index += 1;
                }
                if exponent_start == index {
                    return Err("invalid exponent in guard expression".to_string());
                }
            }
            let number = source[start..index]
                .parse::<f64>()
                .map_err(|error| format!("invalid number in guard expression: {error}"))?;
            if !number.is_finite() {
                return Err("guard expression numeric literals must be finite".to_string());
            }
            tokens.push(Token::Number(number));
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                index += 1;
            }
            tokens.push(Token::Identifier(source[start..index].to_string()));
        } else {
            let remaining = &source[index..];
            let (symbol, width) = [
                ("&&", 2),
                ("||", 2),
                ("<=", 2),
                (">=", 2),
                ("==", 2),
                ("!=", 2),
            ]
            .into_iter()
            .find(|(symbol, _)| remaining.starts_with(symbol))
            .unwrap_or_else(|| match bytes[index] {
                b'(' => ("(", 1),
                b')' => (")", 1),
                b'!' => ("!", 1),
                b'<' => ("<", 1),
                b'>' => (">", 1),
                b'+' => ("+", 1),
                b'-' => ("-", 1),
                b'*' => ("*", 1),
                b'/' => ("/", 1),
                _ => ("", 0),
            });
            if width == 0 {
                let ch = source[index..].chars().next().unwrap_or('\0');
                return Err(format!("unsupported character {ch:?} in guard expression"));
            }
            tokens.push(Token::Symbol(symbol));
            index += width;
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{GuardExpression, MAX_GUARD_EXPRESSION_BYTES, MAX_GUARD_EXPRESSION_TOKENS};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn parses_and_evaluates_compound_guards_with_numeric_precedence() {
        let guard = GuardExpression::parse("temperature >= 10 && (soc > 0.3 || launch)").unwrap();
        assert_eq!(
            guard.variables(),
            &BTreeSet::from([
                "launch".to_string(),
                "soc".to_string(),
                "temperature".to_string()
            ])
        );
        assert!(guard
            .evaluate(&BTreeMap::from([
                ("temperature".to_string(), 12.0),
                ("soc".to_string(), 0.45),
                ("launch".to_string(), 0.0)
            ]))
            .unwrap());
        assert!(!guard
            .evaluate(&BTreeMap::from([
                ("temperature".to_string(), 9.0),
                ("soc".to_string(), 0.45),
                ("launch".to_string(), 1.0)
            ]))
            .unwrap());
    }

    #[test]
    fn scientific_notation_is_numeric_and_not_a_guard_variable() {
        let guard = GuardExpression::parse("temperature > 1e-3 && offset <= .25").unwrap();
        assert_eq!(
            guard.variables(),
            &BTreeSet::from(["offset".to_string(), "temperature".to_string()])
        );
        assert!(guard
            .evaluate(&BTreeMap::from([
                ("temperature".to_string(), 0.1),
                ("offset".to_string(), 0.25)
            ]))
            .unwrap());
    }

    #[test]
    fn malformed_and_nonfinite_guards_fail_explicitly() {
        assert!(GuardExpression::parse("temperature >").is_err());
        assert!(GuardExpression::parse("temperature / 0 > 1")
            .unwrap()
            .evaluate(&BTreeMap::from([("temperature".to_string(), 2.0)]))
            .is_err());
        assert!(GuardExpression::parse("temperature > 0")
            .unwrap()
            .evaluate(&BTreeMap::from([(
                "temperature".to_string(),
                f64::INFINITY
            )]))
            .is_err());
    }

    #[test]
    fn expression_size_and_token_count_are_bounded() {
        assert!(GuardExpression::parse(&"x".repeat(MAX_GUARD_EXPRESSION_BYTES + 1)).is_err());
        assert!(
            GuardExpression::parse(&"x + ".repeat(MAX_GUARD_EXPRESSION_TOKENS / 2 + 2)).is_err()
        );
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    variables: BTreeSet<String>,
}

impl Parser {
    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::End)
    }

    fn take_symbol(&mut self, symbol: &'static str) -> bool {
        if matches!(self.peek(), Token::Symbol(found) if *found == symbol) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_and()?;
        while self.take_symbol("||") {
            expression = Expression::Or(Box::new(expression), Box::new(self.parse_and()?));
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_equality()?;
        while self.take_symbol("&&") {
            expression = Expression::And(Box::new(expression), Box::new(self.parse_equality()?));
        }
        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_comparison()?;
        loop {
            expression = if self.take_symbol("==") {
                Expression::Equal(Box::new(expression), Box::new(self.parse_comparison()?))
            } else if self.take_symbol("!=") {
                Expression::NotEqual(Box::new(expression), Box::new(self.parse_comparison()?))
            } else {
                return Ok(expression);
            };
        }
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_additive()?;
        loop {
            expression = if self.take_symbol("<") {
                Expression::Less(Box::new(expression), Box::new(self.parse_additive()?))
            } else if self.take_symbol("<=") {
                Expression::LessEqual(Box::new(expression), Box::new(self.parse_additive()?))
            } else if self.take_symbol(">") {
                Expression::Greater(Box::new(expression), Box::new(self.parse_additive()?))
            } else if self.take_symbol(">=") {
                Expression::GreaterEqual(Box::new(expression), Box::new(self.parse_additive()?))
            } else {
                return Ok(expression);
            };
        }
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_multiplicative()?;
        loop {
            expression = if self.take_symbol("+") {
                Expression::Add(Box::new(expression), Box::new(self.parse_multiplicative()?))
            } else if self.take_symbol("-") {
                Expression::Subtract(Box::new(expression), Box::new(self.parse_multiplicative()?))
            } else {
                return Ok(expression);
            };
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_unary()?;
        loop {
            expression = if self.take_symbol("*") {
                Expression::Multiply(Box::new(expression), Box::new(self.parse_unary()?))
            } else if self.take_symbol("/") {
                Expression::Divide(Box::new(expression), Box::new(self.parse_unary()?))
            } else {
                return Ok(expression);
            };
        }
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.take_symbol("!") {
            Ok(Expression::UnaryNot(Box::new(self.parse_unary()?)))
        } else if self.take_symbol("-") {
            Ok(Expression::UnaryNegate(Box::new(self.parse_unary()?)))
        } else if self.take_symbol("+") {
            self.parse_unary()
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.peek().clone() {
            Token::Number(number) => {
                self.position += 1;
                Ok(Expression::Number(number))
            }
            Token::Identifier(identifier) => {
                self.position += 1;
                self.variables.insert(identifier.clone());
                Ok(Expression::Variable(identifier))
            }
            Token::Symbol("(") => {
                self.position += 1;
                let expression = self.parse_or()?;
                if !self.take_symbol(")") {
                    return Err("missing ')' in guard expression".to_string());
                }
                Ok(expression)
            }
            token => Err(format!(
                "expected a value in guard expression, found {token}"
            )),
        }
    }
}
