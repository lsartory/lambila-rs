//! # Lambila — VHDL Lexer Library
//!
//! A Rust library that lexes VHDL source into tokens, supporting
//! three language versions:
//!
//! - **VHDL-1987** (IEEE Std 1076-1987) — 81 keywords
//! - **VHDL-1993** (IEEE Std 1076-1993) — 97 keywords
//! - **VHDL-2008** (IEEE Std 1076-2008) — 116 keywords
//!
//! The lexer is stream-based: it reads from any [`std::io::BufRead`]
//! source, so files can be lexed without loading them entirely into
//! memory.
//!
//! ## Usage
//!
//! ```rust
//! use lambila::{lex, VhdlVersion, TokenKind};
//!
//! let source = "entity my_entity is end entity my_entity;";
//! let result = lex(source, VhdlVersion::Vhdl1993);
//!
//! for token in &result.tokens {
//!     println!("{:?}: {:?}", token.kind, token.text);
//! }
//!
//! assert!(result.errors.is_empty());
//! ```

pub mod analyzer;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod project;
pub mod version;

pub use lexer::token::{KeywordKind, LexError, LexResult, Span, Token, TokenKind};
pub use parser::ParseError;
pub use version::VhdlVersion;

use ast::design_unit::DesignFile;
use ast::node::AstNode;

/// Lex a VHDL source string into tokens.
///
/// This is a convenience wrapper that creates an in-memory reader from the
/// string. For large files, prefer [`lex_reader`] or [`lex_file`] which
/// stream from disk.
pub fn lex(source: &str, version: VhdlVersion) -> LexResult {
    let cursor = std::io::Cursor::new(source.as_bytes().to_vec());
    let reader = std::io::BufReader::new(cursor);
    lex_reader(reader, version)
}

/// Lex tokens from any buffered reader (stream-based).
///
/// The lexer reads bytes on demand through the [`BufRead`](std::io::BufRead)
/// interface, using a small internal lookahead buffer. This keeps memory
/// usage proportional to the token stream, not the input size.
///
/// # Example
///
/// ```rust,no_run
/// use lambila::{lex_reader, VhdlVersion};
/// use std::io::BufReader;
/// use std::fs::File;
///
/// let file = File::open("design.vhd").unwrap();
/// let reader = BufReader::new(file);
/// let result = lex_reader(reader, VhdlVersion::Vhdl2008);
/// ```
pub fn lex_reader<R: std::io::BufRead>(reader: R, version: VhdlVersion) -> LexResult {
    let lexer = lexer::Lexer::new(reader, version);
    lexer.lex()
}

/// Read a VHDL file from disk and lex it using a buffered stream.
///
/// The file is read incrementally through a [`BufReader`](std::io::BufReader),
/// so memory usage stays low even for large files.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the file cannot be opened.
///
/// # Example
///
/// ```rust,no_run
/// use lambila::{lex_file, VhdlVersion};
///
/// let result = lex_file("design.vhd", VhdlVersion::Vhdl2008).unwrap();
/// for token in &result.tokens {
///     println!("{:?}: {:?}", token.kind, token.text);
/// }
/// ```
pub fn lex_file<P: AsRef<std::path::Path>>(
    path: P,
    version: VhdlVersion,
) -> std::io::Result<LexResult> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    Ok(lex_reader(reader, version))
}

/// Parse a token stream into a VHDL design file AST.
pub fn parse(tokens: &[Token]) -> Result<DesignFile, ParseError> {
    let mut parser = parser::Parser::new(tokens);
    DesignFile::parse(&mut parser)
}

/// Lex and parse a VHDL source string into an AST.
pub fn parse_str(source: &str, version: VhdlVersion) -> Result<DesignFile, ParseError> {
    let lex_result = lex(source, version);
    if !lex_result.errors.is_empty() {
        return Err(ParseError {
            message: format!("lexer error: {}", lex_result.errors[0]),
            span: Some(lex_result.errors[0].span),
        });
    }
    parse(&lex_result.tokens)
}
