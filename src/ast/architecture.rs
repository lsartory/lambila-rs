//! Architecture body AST nodes.

use super::common::*;
use super::node::{AstNode, write_indent, format_lines};
use crate::parser::{Parser, ParseError};

/// EBNF (VHDL-2008): `architecture_body ::= ARCHITECTURE identifier OF entity_name IS
///     architecture_declarative_part BEGIN architecture_statement_part
///     END [ ARCHITECTURE ] [ architecture_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ architecture_simple_name ] ;` (no optional ARCHITECTURE keyword).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureBody {
    pub identifier: Identifier,
    pub entity_name: SimpleName,
    pub declarative_part: ArchitectureDeclarativePart,
    pub statement_part: ArchitectureStatementPart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `architecture_declarative_part ::= { block_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureDeclarativePart {
    pub items: Vec<super::concurrent::BlockDeclarativeItem>,
}

/// EBNF: `architecture_statement_part ::= { concurrent_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureStatementPart {
    pub statements: Vec<super::concurrent::ConcurrentStatement>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ArchitectureBody {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "architecture ")?;
        self.identifier.format(f, 0)?;
        write!(f, " of ")?;
        self.entity_name.format(f, 0)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "begin")?;
        self.statement_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end architecture")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ArchitectureDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ArchitectureStatementPart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}
