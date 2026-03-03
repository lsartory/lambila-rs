//! Group declaration AST nodes (VHDL-93+).

use super::attribute::EntityClassEntryList;
use super::common::*;
use super::name::Name;
use super::node::{AstNode, format_comma_separated, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Group)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        parser.expect(TokenKind::LeftParen)?;
        let entity_class_entry_list = EntityClassEntryList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(GroupTemplateDeclaration {
            identifier,
            entity_class_entry_list,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Group)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let template_name = Box::new(Name::parse(parser)?);
        parser.expect(TokenKind::LeftParen)?;
        let constituent_list = GroupConstituentList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(GroupDeclaration {
            identifier,
            template_name,
            constituent_list,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::CharacterLiteral) => {
                let token = parser.consume().unwrap();
                let ch = token
                    .text
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                Ok(GroupConstituent::CharacterLiteral(ch))
            }
            _ => {
                let name = Name::parse(parser)?;
                Ok(GroupConstituent::Name(name))
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GroupConstituent::Name(name) => name.format(f, indent_level),
            GroupConstituent::CharacterLiteral(ch) => write!(f, "'{}'", ch),
        }
    }
}

impl AstNode for GroupConstituentList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut constituents = vec![GroupConstituent::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            constituents.push(GroupConstituent::parse(parser)?);
        }
        Ok(GroupConstituentList { constituents })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.constituents, f, indent_level)
    }
}
