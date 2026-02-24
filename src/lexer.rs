use crate::keywords::lookup_keyword;
use crate::token::{LexResult, LexerError, Span, Token, TokenKind};
use crate::version::VhdlVersion;
use std::collections::VecDeque;
use std::io::BufRead;

/// The VHDL lexer. Converts a stream of characters into a sequence of tokens.
///
/// The lexer is generic over any [`BufRead`] source, enabling streaming
/// from files, network sockets, or in-memory buffers without loading the
/// entire input into memory at once.
pub struct Lexer<R: BufRead> {
    reader: R,
    version: VhdlVersion,
    /// Small lookahead buffer (max 3 bytes for compound delimiters like `?/=`).
    lookahead: VecDeque<u8>,
    /// Whether the underlying reader has been exhausted.
    reader_done: bool,
    /// Current byte offset (for span tracking).
    pos: usize,
    line: u32,
    col: u32,
    tokens: Vec<Token>,
    errors: Vec<LexerError>,
    /// Buffer for accumulating the text of the current token.
    current_text: String,
}

impl<R: BufRead> Lexer<R> {
    /// Create a new lexer for the given reader and VHDL version.
    pub fn new(reader: R, version: VhdlVersion) -> Self {
        Lexer {
            reader,
            version,
            lookahead: VecDeque::with_capacity(4),
            reader_done: false,
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            errors: Vec::new(),
            current_text: String::new(),
        }
    }

    /// Run the lexer and return the result.
    pub fn lex(mut self) -> LexResult {
        loop {
            self.skip_whitespace_and_comments();

            if self.at_end() {
                self.emit_simple(TokenKind::Eof, self.pos, self.pos, self.line, self.col);
                break;
            }

            self.next_token();
        }

        LexResult {
            tokens: self.tokens,
            errors: self.errors,
        }
    }

    // ─── Stream helpers ─────────────────────────────────────────────────

    /// Fill the lookahead buffer so it contains at least `n` bytes
    /// (or fewer if the stream is exhausted).
    fn fill_lookahead(&mut self, n: usize) {
        while self.lookahead.len() < n && !self.reader_done {
            let mut buf = [0u8; 1];
            match self.reader.read_exact(&mut buf) {
                Ok(()) => self.lookahead.push_back(buf[0]),
                Err(_) => {
                    self.reader_done = true;
                }
            }
        }
    }

    fn at_end(&mut self) -> bool {
        self.fill_lookahead(1);
        self.lookahead.is_empty()
    }

    fn peek(&mut self) -> u8 {
        self.fill_lookahead(1);
        self.lookahead.front().copied().unwrap_or(0)
    }

    fn peek_at(&mut self, offset: usize) -> u8 {
        self.fill_lookahead(offset + 1);
        self.lookahead.get(offset).copied().unwrap_or(0)
    }

    fn advance(&mut self) -> u8 {
        self.fill_lookahead(1);
        let ch = self.lookahead.pop_front().unwrap_or(0);
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }

    /// Emit a token with pre-built text.
    fn emit(
        &mut self,
        kind: TokenKind,
        text: String,
        start: usize,
        end: usize,
        start_line: u32,
        start_col: u32,
    ) {
        self.tokens.push(Token {
            kind,
            text,
            span: Span {
                start,
                end,
                line: start_line,
                col: start_col,
            },
        });
    }

    /// Emit a token with no text (for Eof, Error, etc.).
    fn emit_simple(
        &mut self,
        kind: TokenKind,
        start: usize,
        end: usize,
        start_line: u32,
        start_col: u32,
    ) {
        self.emit(kind, String::new(), start, end, start_line, start_col);
    }

    /// Take the accumulated `current_text` and emit it as a token.
    fn emit_current(&mut self, kind: TokenKind, start: usize, start_line: u32, start_col: u32) {
        let text = std::mem::take(&mut self.current_text);
        let end = self.pos;
        self.emit(kind, text, start, end, start_line, start_col);
    }

    fn error(&mut self, message: &str, start: usize, end: usize, line: u32, col: u32) {
        self.errors.push(LexerError {
            message: message.to_string(),
            span: Span {
                start,
                end,
                line,
                col,
            },
        });
    }

    /// Advance one byte and append it to `current_text`.
    fn advance_into_text(&mut self) -> u8 {
        let ch = self.advance();
        self.current_text.push(ch as char);
        ch
    }

    /// Returns true if the previous non-whitespace token would make a `'`
    /// an attribute tick rather than the start of a character literal.
    fn prev_token_is_tick_context(&self) -> bool {
        if let Some(prev) = self.tokens.last() {
            matches!(
                prev.kind,
                TokenKind::Identifier
                    | TokenKind::ExtendedIdentifier
                    | TokenKind::RightParen
                    | TokenKind::Kw_All
                    | TokenKind::CharacterLiteral
            ) || prev.kind.is_keyword()
        } else {
            false
        }
    }

    // ─── Whitespace & Comments ──────────────────────────────────────────

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while !self.at_end() && is_whitespace(self.peek()) {
                self.advance();
            }

            if self.at_end() {
                break;
            }

            // Single-line comment: --
            if self.peek() == b'-' && self.peek_at(1) == b'-' {
                self.advance(); // -
                self.advance(); // -
                while !self.at_end() && self.peek() != b'\n' {
                    self.advance();
                }
                continue;
            }

            // Block comment: /* ... */  (VHDL-2008 only)
            if self.version >= VhdlVersion::Vhdl2008
                && self.peek() == b'/'
                && self.peek_at(1) == b'*'
            {
                let start = self.pos;
                let start_line = self.line;
                let start_col = self.col;
                self.advance(); // /
                self.advance(); // *
                let mut terminated = false;
                while !self.at_end() {
                    if self.peek() == b'*' && self.peek_at(1) == b'/' {
                        self.advance(); // *
                        self.advance(); // /
                        terminated = true;
                        break;
                    }
                    self.advance();
                }
                if !terminated {
                    self.error(
                        "unterminated block comment",
                        start,
                        self.pos,
                        start_line,
                        start_col,
                    );
                }
                continue;
            }

            break;
        }
    }

    // ─── Main token dispatch ────────────────────────────────────────────

    fn next_token(&mut self) {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.col;
        let ch = self.peek();

        match ch {
            // ── Letters → identifier or keyword ─────────────────────────
            b'a'..=b'z' | b'A'..=b'Z' => {
                self.lex_identifier_or_keyword(start, start_line, start_col);
            }

            // ── Digits → numeric literal ────────────────────────────────
            b'0'..=b'9' => self.lex_numeric_literal(start, start_line, start_col),

            // ── Extended identifier (VHDL-1993+) ──────────────────────────
            b'\\' => {
                if self.version >= VhdlVersion::Vhdl1993 {
                    self.lex_extended_identifier(start, start_line, start_col);
                } else {
                    self.current_text.clear();
                    self.advance_into_text();
                    self.error(
                        "extended identifiers are not supported in VHDL-1987",
                        start,
                        self.pos,
                        start_line,
                        start_col,
                    );
                    self.emit_current(TokenKind::Error, start, start_line, start_col);
                }
            }

            // ── String literal ──────────────────────────────────────────
            b'"' => self.lex_string_literal(start, start_line, start_col),

            // ── Tick → character literal or attribute tick ───────────────
            b'\'' => {
                if !self.prev_token_is_tick_context()
                    && self.peek_at(1) != 0
                    && self.peek_at(2) == b'\''
                {
                    self.lex_character_literal(start, start_line, start_col);
                } else {
                    self.current_text.clear();
                    self.advance_into_text();
                    self.emit_current(TokenKind::Tick, start, start_line, start_col);
                }
            }

            // ── Simple single-character delimiters ──────────────────────
            b'(' => self.emit_single_char(TokenKind::LeftParen, start, start_line, start_col),
            b')' => self.emit_single_char(TokenKind::RightParen, start, start_line, start_col),
            b'[' => self.emit_single_char(TokenKind::LeftBracket, start, start_line, start_col),
            b']' => self.emit_single_char(TokenKind::RightBracket, start, start_line, start_col),
            b'&' => self.emit_single_char(TokenKind::Ampersand, start, start_line, start_col),
            b'+' => self.emit_single_char(TokenKind::Plus, start, start_line, start_col),
            b',' => self.emit_single_char(TokenKind::Comma, start, start_line, start_col),
            b'-' => self.emit_single_char(TokenKind::Minus, start, start_line, start_col),
            b'.' => self.emit_single_char(TokenKind::Dot, start, start_line, start_col),
            b';' => self.emit_single_char(TokenKind::Semicolon, start, start_line, start_col),
            b'|' => self.emit_single_char(TokenKind::Bar, start, start_line, start_col),
            b'!' => self.emit_single_char(TokenKind::Bar, start, start_line, start_col),

            // ── Star / DoubleStar ───────────────────────────────────────
            b'*' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'*' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::DoubleStar, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::Star, start, start_line, start_col);
                }
            }

            // ── Colon / VarAssign ───────────────────────────────────────
            b':' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::VarAssign, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::Colon, start, start_line, start_col);
                }
            }

            // ── Equals / Arrow ──────────────────────────────────────────
            b'=' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'>' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::Arrow, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::Equals, start, start_line, start_col);
                }
            }

            // ── Less than / LtEquals / Box / DoubleLess ─────────────────
            b'<' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::LtEquals, start, start_line, start_col);
                } else if self.peek() == b'>' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::Box, start, start_line, start_col);
                } else if self.version >= VhdlVersion::Vhdl2008 && self.peek() == b'<' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::DoubleLess, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::LessThan, start, start_line, start_col);
                }
            }

            // ── Greater than / GtEquals / DoubleGreater ─────────────────
            b'>' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::GtEquals, start, start_line, start_col);
                } else if self.version >= VhdlVersion::Vhdl2008 && self.peek() == b'>' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::DoubleGreater, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::GreaterThan, start, start_line, start_col);
                }
            }

            // ── Slash / NotEquals ────────────────────────────────────────
            b'/' => {
                self.current_text.clear();
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::NotEquals, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::Slash, start, start_line, start_col);
                }
            }

            // ── Question mark (VHDL-2008 matching operators) ────────────
            b'?' => {
                if self.version >= VhdlVersion::Vhdl2008 {
                    self.lex_question_mark(start, start_line, start_col);
                } else {
                    self.current_text.clear();
                    self.advance_into_text();
                    self.error(
                        "unexpected character '?'",
                        start,
                        self.pos,
                        start_line,
                        start_col,
                    );
                    self.emit_current(TokenKind::Error, start, start_line, start_col);
                }
            }

            // ── Percent sign (character replacement for ") ──────────────
            b'%' => self.lex_string_literal_percent(start, start_line, start_col),

            // ── Unknown character ───────────────────────────────────────
            _ => {
                self.current_text.clear();
                self.advance_into_text();
                self.error(
                    &format!("unexpected character '{}'", ch as char),
                    start,
                    self.pos,
                    start_line,
                    start_col,
                );
                self.emit_current(TokenKind::Error, start, start_line, start_col);
            }
        }
    }

    /// Shorthand: emit a single-character delimiter token.
    fn emit_single_char(&mut self, kind: TokenKind, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text();
        self.emit_current(kind, start, start_line, start_col);
    }

    // ─── Identifier / keyword ───────────────────────────────────────────

    fn lex_identifier_or_keyword(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // first letter
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.advance_into_text();
            } else {
                break;
            }
        }

        // Check if this might be a bit string literal: base_specifier " bit_value "
        if self.peek() == b'"' || self.peek() == b'%' {
            let lower = self.current_text.to_ascii_lowercase();
            let is_base_spec = match lower.as_str() {
                "b" | "o" | "x" => true,
                "d" | "ub" | "uo" | "ux" | "sb" | "so" | "sx"
                    if self.version >= VhdlVersion::Vhdl2008 =>
                {
                    true
                }
                _ => false,
            };
            if is_base_spec {
                self.lex_bit_string_body(start, start_line, start_col);
                return;
            }
        }

        let kind =
            lookup_keyword(&self.current_text, self.version).unwrap_or(TokenKind::Identifier);
        self.emit_current(kind, start, start_line, start_col);
    }

    // ─── Extended identifier ────────────────────────────────────────────

    fn lex_extended_identifier(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // opening backslash
        while !self.at_end() {
            let ch = self.peek();
            if ch == b'\\' {
                self.advance_into_text();
                // Double backslash is an escaped backslash within the identifier
                if self.peek() == b'\\' {
                    self.advance_into_text();
                    continue;
                }
                // Single backslash closes the extended identifier
                self.emit_current(TokenKind::ExtendedIdentifier, start, start_line, start_col);
                return;
            }
            if ch == b'\n' {
                break;
            }
            self.advance_into_text();
        }
        self.error(
            "unterminated extended identifier",
            start,
            self.pos,
            start_line,
            start_col,
        );
        self.emit_current(TokenKind::Error, start, start_line, start_col);
    }

    // ─── Numeric literals ───────────────────────────────────────────────

    fn lex_numeric_literal(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.consume_digits_into_text();

        // Check for based literal: integer # based_integer [.based_integer] # [exponent]
        if self.peek() == b'#' || self.peek() == b':' {
            self.lex_based_literal(start, start_line, start_col);
            return;
        }

        // Check for VHDL-2008 integer-prefixed bit string literal
        if self.version >= VhdlVersion::Vhdl2008 {
            // We need to check if next chars are a base specifier + quote
            let ch0 = (self.peek() as char).to_ascii_lowercase();
            let ch1 = (self.peek_at(1) as char).to_ascii_lowercase();
            let ch2 = self.peek_at(2) as char;

            let base_spec_len = match (ch0, ch1, ch2) {
                ('u', 'b' | 'o' | 'x', '"' | '%') => 2,
                ('s', 'b' | 'o' | 'x', '"' | '%') => 2,
                ('b' | 'o' | 'x' | 'd', '"' | '%', _) => 1,
                _ => 0,
            };

            if base_spec_len > 0 {
                for _ in 0..base_spec_len {
                    self.advance_into_text();
                }
                self.lex_bit_string_body(start, start_line, start_col);
                return;
            }
        }

        // Check for real literal: integer . integer [exponent]
        if self.peek() == b'.' && self.peek_at(1).is_ascii_digit() {
            self.advance_into_text(); // .
            self.consume_digits_into_text();
            if self.peek() == b'E' || self.peek() == b'e' {
                self.consume_exponent_into_text();
            }
            self.emit_current(TokenKind::RealLiteral, start, start_line, start_col);
            return;
        }

        // Optional exponent
        if self.peek() == b'E' || self.peek() == b'e' {
            self.consume_exponent_into_text();
        }

        self.emit_current(TokenKind::IntegerLiteral, start, start_line, start_col);
    }

    fn lex_based_literal(&mut self, start: usize, start_line: u32, start_col: u32) {
        let delim = self.peek();
        self.advance_into_text(); // # or :

        self.consume_based_digits_into_text();

        if self.peek() == b'.' {
            self.advance_into_text();
            self.consume_based_digits_into_text();
        }

        if self.peek() == delim {
            self.advance_into_text();
        } else {
            self.error(
                "expected closing '#' or ':' in based literal",
                start,
                self.pos,
                start_line,
                start_col,
            );
        }

        if self.peek() == b'E' || self.peek() == b'e' {
            self.consume_exponent_into_text();
        }

        self.emit_current(TokenKind::BasedLiteral, start, start_line, start_col);
    }

    fn consume_digits_into_text(&mut self) {
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_ascii_digit() || ch == b'_' {
                self.advance_into_text();
            } else {
                break;
            }
        }
    }

    fn consume_based_digits_into_text(&mut self) {
        while !self.at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.advance_into_text();
            } else {
                break;
            }
        }
    }

    fn consume_exponent_into_text(&mut self) {
        self.advance_into_text(); // E or e
        if self.peek() == b'+' || self.peek() == b'-' {
            self.advance_into_text();
        }
        self.consume_digits_into_text();
    }

    // ─── Character literal ──────────────────────────────────────────────

    fn lex_character_literal(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // opening '
        self.advance_into_text(); // the character
        self.advance_into_text(); // closing '
        self.emit_current(TokenKind::CharacterLiteral, start, start_line, start_col);
    }

    // ─── String literal ─────────────────────────────────────────────────

    fn lex_string_literal(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // opening "
        while !self.at_end() {
            let ch = self.peek();
            if ch == b'"' {
                self.advance_into_text();
                if self.peek() == b'"' {
                    self.advance_into_text();
                    continue;
                }
                self.emit_current(TokenKind::StringLiteral, start, start_line, start_col);
                return;
            }
            if ch == b'\n' {
                break;
            }
            self.advance_into_text();
        }
        self.error(
            "unterminated string literal",
            start,
            self.pos,
            start_line,
            start_col,
        );
        self.emit_current(TokenKind::Error, start, start_line, start_col);
    }

    fn lex_string_literal_percent(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // opening %
        while !self.at_end() {
            let ch = self.peek();
            if ch == b'%' {
                self.advance_into_text();
                if self.peek() == b'%' {
                    self.advance_into_text();
                    continue;
                }
                self.emit_current(TokenKind::StringLiteral, start, start_line, start_col);
                return;
            }
            if ch == b'\n' {
                break;
            }
            self.advance_into_text();
        }
        self.error(
            "unterminated string literal (percent-delimited)",
            start,
            self.pos,
            start_line,
            start_col,
        );
        self.emit_current(TokenKind::Error, start, start_line, start_col);
    }

    // ─── Bit string literal ─────────────────────────────────────────────

    /// Lex the body of a bit string literal (the `"..."` portion).
    /// `current_text` already contains the base specifier (and optional length).
    fn lex_bit_string_body(&mut self, start: usize, start_line: u32, start_col: u32) {
        let delim = self.peek();
        if delim != b'"' && delim != b'%' {
            self.error(
                "expected '\"' or '%' after base specifier",
                start,
                self.pos,
                start_line,
                start_col,
            );
            self.emit_current(TokenKind::Error, start, start_line, start_col);
            return;
        }
        self.advance_into_text(); // opening delimiter

        while !self.at_end() {
            let ch = self.peek();
            if ch == delim {
                self.advance_into_text();
                if self.peek() == delim {
                    self.advance_into_text();
                    continue;
                }
                self.emit_current(TokenKind::BitStringLiteral, start, start_line, start_col);
                return;
            }
            if ch == b'\n' {
                break;
            }
            self.advance_into_text();
        }
        self.error(
            "unterminated bit string literal",
            start,
            self.pos,
            start_line,
            start_col,
        );
        self.emit_current(TokenKind::Error, start, start_line, start_col);
    }

    // ─── Question mark operators (VHDL-2008) ────────────────────────────

    fn lex_question_mark(&mut self, start: usize, start_line: u32, start_col: u32) {
        self.current_text.clear();
        self.advance_into_text(); // ?
        match self.peek() {
            b'?' => {
                self.advance_into_text();
                self.emit_current(TokenKind::Condition, start, start_line, start_col);
            }
            b'=' => {
                self.advance_into_text();
                self.emit_current(TokenKind::MatchEq, start, start_line, start_col);
            }
            b'/' => {
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::MatchNeq, start, start_line, start_col);
                } else {
                    self.error(
                        "expected '=' after '?/'",
                        start,
                        self.pos,
                        start_line,
                        start_col,
                    );
                    self.emit_current(TokenKind::Error, start, start_line, start_col);
                }
            }
            b'<' => {
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::MatchLte, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::MatchLt, start, start_line, start_col);
                }
            }
            b'>' => {
                self.advance_into_text();
                if self.peek() == b'=' {
                    self.advance_into_text();
                    self.emit_current(TokenKind::MatchGte, start, start_line, start_col);
                } else {
                    self.emit_current(TokenKind::MatchGt, start, start_line, start_col);
                }
            }
            _ => {
                self.emit_current(TokenKind::QuestionMark, start, start_line, start_col);
            }
        }
    }
}

// ─── Utility ────────────────────────────────────────────────────────────

fn is_whitespace(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn lex_tokens(source: &str, version: VhdlVersion) -> Vec<Token> {
        let reader = Cursor::new(source.as_bytes().to_vec());
        let lexer = Lexer::new(std::io::BufReader::new(reader), version);
        lexer.lex().tokens
    }

    fn token_kinds(source: &str, version: VhdlVersion) -> Vec<TokenKind> {
        lex_tokens(source, version)
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_empty_input() {
        let kinds = token_kinds("", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_whitespace_only() {
        let kinds = token_kinds("   \n\t  ", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_single_line_comment() {
        let kinds = token_kinds("-- this is a comment\n", VhdlVersion::Vhdl1987);
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_block_comment_2008() {
        let kinds = token_kinds("/* block\ncomment */", VhdlVersion::Vhdl2008);
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_identifier() {
        let kinds = token_kinds("my_signal", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::Identifier, TokenKind::Eof]);
    }

    #[test]
    fn test_keyword_entity() {
        let kinds = token_kinds("ENTITY", VhdlVersion::Vhdl1987);
        assert_eq!(kinds, vec![TokenKind::Kw_Entity, TokenKind::Eof]);
    }

    #[test]
    fn test_keyword_case_insensitive() {
        let kinds = token_kinds("entity", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::Kw_Entity, TokenKind::Eof]);
    }

    #[test]
    fn test_keyword_version_gating() {
        let kinds_87 = token_kinds("xnor", VhdlVersion::Vhdl1987);
        assert_eq!(kinds_87, vec![TokenKind::Identifier, TokenKind::Eof]);

        let kinds_93 = token_kinds("xnor", VhdlVersion::Vhdl1993);
        assert_eq!(kinds_93, vec![TokenKind::Kw_Xnor, TokenKind::Eof]);
    }

    #[test]
    fn test_integer_literal() {
        let kinds = token_kinds("42", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::IntegerLiteral, TokenKind::Eof]);

        let kinds = token_kinds("1_000_000", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::IntegerLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_real_literal() {
        let kinds = token_kinds("3.14", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::RealLiteral, TokenKind::Eof]);

        let kinds = token_kinds("1.0e-3", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::RealLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_based_literal() {
        let kinds = token_kinds("16#FF#", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::BasedLiteral, TokenKind::Eof]);

        let kinds = token_kinds("2#1010_1100#", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::BasedLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_character_literal() {
        let kinds = token_kinds("'A'", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::CharacterLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_string_literal() {
        let kinds = token_kinds("\"hello world\"", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::StringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_string_literal_with_escaped_quote() {
        let tokens = lex_tokens("\"say \"\"hello\"\"\"", VhdlVersion::Vhdl1993);
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn test_bit_string_literal() {
        let kinds = token_kinds("B\"1010\"", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::BitStringLiteral, TokenKind::Eof]);

        let kinds = token_kinds("X\"FF\"", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::BitStringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_extended_identifier_93() {
        let kinds = token_kinds("\\my signal\\", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::ExtendedIdentifier, TokenKind::Eof]);
    }

    #[test]
    fn test_extended_identifier_rejected_87() {
        let reader = Cursor::new("\\my signal\\".as_bytes().to_vec());
        let result = Lexer::new(std::io::BufReader::new(reader), VhdlVersion::Vhdl1987).lex();
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_delimiters() {
        let kinds = token_kinds("( ) ; : , . => := /= >= <= <>", VhdlVersion::Vhdl1993);
        assert_eq!(
            kinds,
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::Semicolon,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Arrow,
                TokenKind::VarAssign,
                TokenKind::NotEquals,
                TokenKind::GtEquals,
                TokenKind::LtEquals,
                TokenKind::Box,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_matching_operators_2008() {
        let kinds = token_kinds("?= ?/= ?< ?<= ?> ?>=", VhdlVersion::Vhdl2008);
        assert_eq!(
            kinds,
            vec![
                TokenKind::MatchEq,
                TokenKind::MatchNeq,
                TokenKind::MatchLt,
                TokenKind::MatchLte,
                TokenKind::MatchGt,
                TokenKind::MatchGte,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_condition_operator_2008() {
        let kinds = token_kinds("??", VhdlVersion::Vhdl2008);
        assert_eq!(kinds, vec![TokenKind::Condition, TokenKind::Eof]);
    }

    #[test]
    fn test_external_name_delimiters_2008() {
        let kinds = token_kinds("<< >>", VhdlVersion::Vhdl2008);
        assert_eq!(
            kinds,
            vec![
                TokenKind::DoubleLess,
                TokenKind::DoubleGreater,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tick_as_attribute() {
        let kinds = token_kinds("sig'range", VhdlVersion::Vhdl1993);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier,
                TokenKind::Tick,
                TokenKind::Kw_Range,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_tick_as_character_literal() {
        let kinds = token_kinds("= '1'", VhdlVersion::Vhdl1993);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Equals,
                TokenKind::CharacterLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_entity_declaration() {
        let vhdl = "entity my_entity is\nend entity my_entity;";
        let kinds = token_kinds(vhdl, VhdlVersion::Vhdl1993);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Kw_Entity,
                TokenKind::Identifier,
                TokenKind::Kw_Is,
                TokenKind::Kw_End,
                TokenKind::Kw_Entity,
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_span_tracking() {
        let tokens = lex_tokens("entity foo", VhdlVersion::Vhdl1993);
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.col, 1);
        assert_eq!(tokens[1].span.line, 1);
        assert_eq!(tokens[1].span.col, 8);
    }

    #[test]
    fn test_span_multiline() {
        let tokens = lex_tokens("a\nb", VhdlVersion::Vhdl1993);
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.col, 1);
    }

    #[test]
    fn test_exclamation_as_bar() {
        let kinds = token_kinds("!", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::Bar, TokenKind::Eof]);
    }

    #[test]
    fn test_percent_string() {
        let kinds = token_kinds("%hello%", VhdlVersion::Vhdl1993);
        assert_eq!(kinds, vec![TokenKind::StringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_unterminated_string_error() {
        let reader = Cursor::new("\"unterminated".as_bytes().to_vec());
        let result = Lexer::new(std::io::BufReader::new(reader), VhdlVersion::Vhdl1993).lex();
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("unterminated"));
    }

    #[test]
    fn test_unterminated_block_comment_error() {
        let reader = Cursor::new("/* unterminated".as_bytes().to_vec());
        let result = Lexer::new(std::io::BufReader::new(reader), VhdlVersion::Vhdl2008).lex();
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("unterminated"));
    }

    #[test]
    fn test_2008_bit_string_with_length() {
        let kinds = token_kinds("12UB\"0000_1111\"", VhdlVersion::Vhdl2008);
        assert_eq!(kinds, vec![TokenKind::BitStringLiteral, TokenKind::Eof]);
    }
}
