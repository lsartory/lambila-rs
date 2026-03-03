//! Context and library AST nodes.

use super::common::*;
use super::name::SelectedName;
use super::node::{AstNode, write_indent, format_comma_separated, format_lines};
use crate::parser::{Parser, ParseError};

/// EBNF: `context_clause ::= { context_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextClause {
    pub items: Vec<ContextItem>,
}

/// EBNF (VHDL-2008): `context_item ::= library_clause | use_clause | context_reference`
/// EBNF (VHDL-87/93): `context_item ::= library_clause | use_clause`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextItem {
    Library(LibraryClause),
    Use(UseClause),
    /// VHDL-2008.
    ContextReference(ContextReference),
}

/// EBNF: `context_declaration ::= CONTEXT identifier IS context_clause
///     END [ CONTEXT ] [ context_simple_name ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDeclaration {
    pub identifier: Identifier,
    pub context_clause: ContextClause,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `context_reference ::= CONTEXT selected_name { , selected_name } ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextReference {
    pub names: Vec<SelectedName>,
}

/// EBNF: `library_clause ::= LIBRARY logical_name_list ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryClause {
    pub logical_names: LogicalNameList,
}

/// EBNF: `logical_name ::= identifier`
pub type LogicalName = Identifier;

/// EBNF: `logical_name_list ::= logical_name { , logical_name }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalNameList {
    pub names: Vec<LogicalName>,
}

/// EBNF: `use_clause ::= USE selected_name { , selected_name } ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseClause {
    pub names: Vec<SelectedName>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ContextClause {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ContextItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ContextItem::Library(clause) => clause.format(f, indent_level),
            ContextItem::Use(clause) => clause.format(f, indent_level),
            ContextItem::ContextReference(reference) => reference.format(f, indent_level),
        }
    }
}

impl AstNode for ContextDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "context ")?;
        self.identifier.format(f, 0)?;
        writeln!(f, " is")?;
        self.context_clause.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end context")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ContextReference {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "context ")?;
        format_comma_separated(&self.names, f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for LibraryClause {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "library ")?;
        self.logical_names.format(f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for LogicalNameList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.names, f, indent_level)
    }
}

impl AstNode for UseClause {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "use ")?;
        format_comma_separated(&self.names, f, 0)?;
        writeln!(f, ";")
    }
}
