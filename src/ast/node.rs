//! The `AstNode` trait — a unified interface for parsing and formatting AST nodes.

use crate::parser::{ParseError, Parser};

/// A trait implemented by every AST node, providing a unified parsing
/// and formatting interface.
pub trait AstNode: Sized {
    /// Parse this AST node from the current parser state.
    fn parse(parser: &mut Parser) -> Result<Self, ParseError>;

    /// Format this AST node recursively with proper indentation.
    ///
    /// `indent_level` determines the number of indentation units
    /// (each unit is 4 spaces).
    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result;
}

/// Write `indent_level * 4` spaces to the formatter.
pub fn write_indent(f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
    for _ in 0..level {
        write!(f, "    ")?;
    }
    Ok(())
}

/// Format a comma-separated list of AST nodes on a single line.
pub fn format_comma_separated<T: AstNode>(
    items: &[T],
    f: &mut std::fmt::Formatter<'_>,
    indent_level: usize,
) -> std::fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        item.format(f, indent_level)?;
    }
    Ok(())
}

/// Format a comma-separated list of items, each on its own indented line.
pub fn format_comma_lines<T: AstNode>(
    items: &[T],
    f: &mut std::fmt::Formatter<'_>,
    indent_level: usize,
) -> std::fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            writeln!(f, ",")?;
        }
        write_indent(f, indent_level)?;
        item.format(f, indent_level)?;
    }
    Ok(())
}

/// Format a list of AST nodes, each on its own line at the given indent level.
pub fn format_lines<T: AstNode>(
    items: &[T],
    f: &mut std::fmt::Formatter<'_>,
    indent_level: usize,
) -> std::fmt::Result {
    for item in items {
        item.format(f, indent_level)?;
    }
    Ok(())
}

/// Format a semicolon-separated list of items, each on its own line.
pub fn format_semicolon_lines<T: AstNode>(
    items: &[T],
    f: &mut std::fmt::Formatter<'_>,
    indent_level: usize,
) -> std::fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            writeln!(f, ";")?;
        }
        item.format(f, indent_level)?;
    }
    Ok(())
}
