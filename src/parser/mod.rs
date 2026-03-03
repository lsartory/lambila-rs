//! Parser infrastructure for VHDL token streams.

use crate::{Span, Token, TokenKind};

/// A parse error with optional source location.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Human-readable error message.
    pub message: String,
    /// Location of the error in the source, if available.
    pub span: Option<Span>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}:{}: {}", span.line, span.col, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// A cursor-based parser over a slice of VHDL tokens.
pub struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser over the given token slice.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    /// Peek at the current token without consuming it.
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// Consume the current token and advance the cursor.
    pub fn consume(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.cursor);
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }

    /// Expect the current token to be of the given kind, consume it,
    /// and return a reference to it. Returns a `ParseError` on mismatch.
    pub fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        match self.peek() {
            Some(token) if token.kind == kind => {
                self.cursor += 1;
                Ok(&self.tokens[self.cursor - 1])
            }
            Some(token) => Err(ParseError {
                message: format!("expected {:?}, found {:?}", kind, token.kind),
                span: Some(token.span),
            }),
            None => Err(ParseError {
                message: format!("expected {:?}, found end of input", kind),
                span: None,
            }),
        }
    }

    /// Returns `true` if the parser has reached the end of input.
    pub fn eof(&self) -> bool {
        self.cursor >= self.tokens.len()
            || self
                .tokens
                .get(self.cursor)
                .is_none_or(|t| t.kind == TokenKind::Eof)
    }
}
