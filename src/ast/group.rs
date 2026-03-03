//! Group declaration AST nodes (VHDL-93+).

use super::attribute::EntityClassEntryList;
use super::common::*;
use super::name::Name;
use super::node::{AstNode, write_indent, format_comma_separated};
use crate::parser::{Parser, ParseError};

/// EBNF: `group_template_declaration ::= GROUP identifier IS ( entity_class_entry_list ) ;`
/// (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupTemplateDeclaration {
    pub identifier: Identifier,
    pub entity_class_entry_list: EntityClassEntryList,
}

/// EBNF: `group_declaration ::= GROUP identifier : group_template_name
///     ( group_constituent_list ) ;` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDeclaration {
    pub identifier: Identifier,
    pub template_name: Box<Name>,
    pub constituent_list: GroupConstituentList,
}

/// EBNF: `group_constituent ::= name | character_literal` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupConstituent {
    Name(Name),
    CharacterLiteral(String),
}

/// EBNF: `group_constituent_list ::= group_constituent { , group_constituent }` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupConstituentList {
    pub constituents: Vec<GroupConstituent>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for GroupTemplateDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "group ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is (")?;
        self.entity_class_entry_list.format(f, indent_level)?;
        writeln!(f, ");")
    }
}

impl AstNode for GroupDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "group ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " : ")?;
        self.template_name.format(f, indent_level)?;
        write!(f, " (")?;
        self.constituent_list.format(f, indent_level)?;
        writeln!(f, ");")
    }
}

impl AstNode for GroupConstituent {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GroupConstituent::Name(name) => name.format(f, indent_level),
            GroupConstituent::CharacterLiteral(ch) => write!(f, "'{}'", ch),
        }
    }
}

impl AstNode for GroupConstituentList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.constituents, f, indent_level)
    }
}
