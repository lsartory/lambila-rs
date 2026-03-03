//! Context and library AST nodes.

use super::common::*;
use super::name::SelectedName;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut items = Vec::new();
        while parser.at_keyword(KeywordKind::Library)
            || parser.at_keyword(KeywordKind::Use)
            || parser.at_keyword(KeywordKind::Context)
        {
            items.push(ContextItem::parse(parser)?);
        }
        Ok(ContextClause { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ContextItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Library) {
            Ok(ContextItem::Library(LibraryClause::parse(parser)?))
        } else if parser.at_keyword(KeywordKind::Use) {
            Ok(ContextItem::Use(UseClause::parse(parser)?))
        } else if parser.at_keyword(KeywordKind::Context) {
            // This must be a context_reference (CONTEXT selected_name { , selected_name } ;)
            // Context declarations are handled at the design_unit level, not here.
            Ok(ContextItem::ContextReference(ContextReference::parse(
                parser,
            )?))
        } else {
            Err(parser.error("expected context item (library, use, or context reference)"))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // CONTEXT identifier IS context_clause END [ CONTEXT ] [ context_simple_name ] ;
        parser.expect_keyword(KeywordKind::Context)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let context_clause = ContextClause::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        // Optional CONTEXT keyword
        parser.consume_if_keyword(KeywordKind::Context);
        // Optional end name
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ContextDeclaration {
            identifier,
            context_clause,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // CONTEXT selected_name { , selected_name } ;
        parser.expect_keyword(KeywordKind::Context)?;
        let mut names = vec![SelectedName::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            names.push(SelectedName::parse(parser)?);
        }
        parser.expect(TokenKind::Semicolon)?;
        Ok(ContextReference { names })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "context ")?;
        format_comma_separated(&self.names, f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for LibraryClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // LIBRARY logical_name_list ;
        parser.expect_keyword(KeywordKind::Library)?;
        let logical_names = LogicalNameList::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(LibraryClause { logical_names })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "library ")?;
        self.logical_names.format(f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for LogicalNameList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // logical_name { , logical_name }
        let mut names = vec![Identifier::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            names.push(Identifier::parse(parser)?);
        }
        Ok(LogicalNameList { names })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.names, f, indent_level)
    }
}

impl AstNode for UseClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // USE selected_name { , selected_name } ;
        parser.expect_keyword(KeywordKind::Use)?;
        let mut names = vec![SelectedName::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            names.push(SelectedName::parse(parser)?);
        }
        parser.expect(TokenKind::Semicolon)?;
        Ok(UseClause { names })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "use ")?;
        format_comma_separated(&self.names, f, 0)?;
        writeln!(f, ";")
    }
}
