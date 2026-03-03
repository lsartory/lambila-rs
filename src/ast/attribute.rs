//! Attribute declaration and specification AST nodes.

use super::common::*;
use super::expression::Expression;
use super::node::{AstNode, format_comma_separated, write_indent};
use super::type_def::TypeMark;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `attribute_declaration ::= ATTRIBUTE identifier : type_mark ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDeclaration {
    pub identifier: Identifier,
    pub type_mark: TypeMark,
}

/// EBNF: `attribute_specification ::= ATTRIBUTE attribute_designator OF
///     entity_specification IS expression ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSpecification {
    pub designator: SimpleName,
    pub entity_specification: EntitySpecification,
    pub expression: Expression,
}

/// EBNF: `entity_specification ::= entity_name_list : entity_class`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySpecification {
    pub name_list: EntityNameList,
    pub entity_class: EntityClass,
}

/// EBNF (VHDL-2008): `entity_class ::= ENTITY | ARCHITECTURE | CONFIGURATION | PROCEDURE
///     | FUNCTION | PACKAGE | TYPE | SUBTYPE | CONSTANT | SIGNAL | VARIABLE | COMPONENT
///     | LABEL | LITERAL | UNITS | GROUP | FILE | PROPERTY | SEQUENCE`
/// EBNF (VHDL-87): omits GROUP, FILE, PROPERTY, SEQUENCE, LITERAL, UNITS.
/// EBNF (VHDL-93): adds GROUP, FILE, LITERAL, UNITS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityClass {
    Entity,
    Architecture,
    Configuration,
    Procedure,
    Function,
    Package,
    Type,
    Subtype,
    Constant,
    Signal,
    Variable,
    Component,
    Label,
    /// VHDL-93+.
    Literal,
    /// VHDL-93+.
    Units,
    /// VHDL-93+.
    Group,
    /// VHDL-93+.
    File,
    /// VHDL-2008.
    Property,
    /// VHDL-2008.
    Sequence,
}

/// EBNF (VHDL-93+): `entity_class_entry ::= entity_class [ <> ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityClassEntry {
    pub entity_class: EntityClass,
    pub has_box: bool,
}

/// EBNF (VHDL-93+): `entity_class_entry_list ::= entity_class_entry
///     { , entity_class_entry }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityClassEntryList {
    pub entries: Vec<EntityClassEntry>,
}

/// EBNF (VHDL-93+): `entity_designator ::= entity_tag [ signature ]`
/// EBNF (VHDL-87): `entity_designator ::= simple_name | character_literal | operator_symbol`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDesignator {
    pub tag: EntityTag,
    /// VHDL-93+.
    pub signature: Option<Signature>,
}

/// EBNF (VHDL-93+): `entity_name_list ::= entity_designator { , entity_designator }
///     | OTHERS | ALL`
/// EBNF (VHDL-87): same structure but uses slightly different designator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityNameList {
    Designators(Vec<EntityDesignator>),
    Others,
    All,
}

/// EBNF (VHDL-93+): `entity_tag ::= simple_name | character_literal | operator_symbol`
/// EBNF (VHDL-87): same set without signature context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityTag {
    SimpleName(SimpleName),
    CharacterLiteral(String),
    OperatorSymbol(OperatorSymbol),
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for AttributeDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Attribute)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let type_mark = TypeMark::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(AttributeDeclaration {
            identifier,
            type_mark,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "attribute ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " : ")?;
        self.type_mark.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for AttributeSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Attribute)?;
        let designator = SimpleName::parse(parser)?;
        parser.expect_keyword(KeywordKind::Of)?;
        let entity_specification = EntitySpecification::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let expression = Expression::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(AttributeSpecification {
            designator,
            entity_specification,
            expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "attribute ")?;
        self.designator.format(f, indent_level)?;
        write!(f, " of ")?;
        self.entity_specification.format(f, indent_level)?;
        write!(f, " is ")?;
        self.expression.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for EntitySpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let name_list = EntityNameList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let entity_class = EntityClass::parse(parser)?;
        Ok(EntitySpecification {
            name_list,
            entity_class,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name_list.format(f, indent_level)?;
        write!(f, " : ")?;
        self.entity_class.format(f, indent_level)
    }
}

impl AstNode for EntityClass {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Entity).is_some() {
            Ok(EntityClass::Entity)
        } else if parser
            .consume_if_keyword(KeywordKind::Architecture)
            .is_some()
        {
            Ok(EntityClass::Architecture)
        } else if parser
            .consume_if_keyword(KeywordKind::Configuration)
            .is_some()
        {
            Ok(EntityClass::Configuration)
        } else if parser.consume_if_keyword(KeywordKind::Procedure).is_some() {
            Ok(EntityClass::Procedure)
        } else if parser.consume_if_keyword(KeywordKind::Function).is_some() {
            Ok(EntityClass::Function)
        } else if parser.consume_if_keyword(KeywordKind::Package).is_some() {
            Ok(EntityClass::Package)
        } else if parser.consume_if_keyword(KeywordKind::Type).is_some() {
            Ok(EntityClass::Type)
        } else if parser.consume_if_keyword(KeywordKind::Subtype).is_some() {
            Ok(EntityClass::Subtype)
        } else if parser.consume_if_keyword(KeywordKind::Constant).is_some() {
            Ok(EntityClass::Constant)
        } else if parser.consume_if_keyword(KeywordKind::Signal).is_some() {
            Ok(EntityClass::Signal)
        } else if parser.consume_if_keyword(KeywordKind::Variable).is_some() {
            Ok(EntityClass::Variable)
        } else if parser.consume_if_keyword(KeywordKind::Component).is_some() {
            Ok(EntityClass::Component)
        } else if parser.consume_if_keyword(KeywordKind::Label).is_some() {
            Ok(EntityClass::Label)
        } else if parser.consume_if_keyword(KeywordKind::Literal).is_some() {
            Ok(EntityClass::Literal)
        } else if parser.consume_if_keyword(KeywordKind::Units).is_some() {
            Ok(EntityClass::Units)
        } else if parser.consume_if_keyword(KeywordKind::Group).is_some() {
            Ok(EntityClass::Group)
        } else if parser.consume_if_keyword(KeywordKind::File).is_some() {
            Ok(EntityClass::File)
        } else if parser.consume_if_keyword(KeywordKind::Property).is_some() {
            Ok(EntityClass::Property)
        } else if parser.consume_if_keyword(KeywordKind::Sequence).is_some() {
            Ok(EntityClass::Sequence)
        } else {
            Err(parser.error("expected entity class"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            EntityClass::Entity => write!(f, "entity"),
            EntityClass::Architecture => write!(f, "architecture"),
            EntityClass::Configuration => write!(f, "configuration"),
            EntityClass::Procedure => write!(f, "procedure"),
            EntityClass::Function => write!(f, "function"),
            EntityClass::Package => write!(f, "package"),
            EntityClass::Type => write!(f, "type"),
            EntityClass::Subtype => write!(f, "subtype"),
            EntityClass::Constant => write!(f, "constant"),
            EntityClass::Signal => write!(f, "signal"),
            EntityClass::Variable => write!(f, "variable"),
            EntityClass::Component => write!(f, "component"),
            EntityClass::Label => write!(f, "label"),
            EntityClass::Literal => write!(f, "literal"),
            EntityClass::Units => write!(f, "units"),
            EntityClass::Group => write!(f, "group"),
            EntityClass::File => write!(f, "file"),
            EntityClass::Property => write!(f, "property"),
            EntityClass::Sequence => write!(f, "sequence"),
        }
    }
}

impl AstNode for EntityClassEntry {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let entity_class = EntityClass::parse(parser)?;
        let has_box = parser.consume_if(TokenKind::Box).is_some();
        Ok(EntityClassEntry {
            entity_class,
            has_box,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.entity_class.format(f, indent_level)?;
        if self.has_box {
            write!(f, " <>")?;
        }
        Ok(())
    }
}

impl AstNode for EntityClassEntryList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut entries = vec![EntityClassEntry::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            entries.push(EntityClassEntry::parse(parser)?);
        }
        Ok(EntityClassEntryList { entries })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.entries, f, indent_level)
    }
}

impl AstNode for EntityDesignator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let tag = EntityTag::parse(parser)?;
        // Optional signature (starts with `[`)
        let signature = if parser.at(TokenKind::LeftBracket) {
            Some(Signature::parse(parser)?)
        } else {
            None
        };
        Ok(EntityDesignator { tag, signature })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.tag.format(f, indent_level)?;
        if let Some(sig) = &self.signature {
            write!(f, " ")?;
            sig.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for EntityNameList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Others).is_some() {
            Ok(EntityNameList::Others)
        } else if parser.consume_if_keyword(KeywordKind::All).is_some() {
            Ok(EntityNameList::All)
        } else {
            let mut designators = vec![EntityDesignator::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                designators.push(EntityDesignator::parse(parser)?);
            }
            Ok(EntityNameList::Designators(designators))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EntityNameList::Designators(desigs) => format_comma_separated(desigs, f, indent_level),
            EntityNameList::Others => write!(f, "others"),
            EntityNameList::All => write!(f, "all"),
        }
    }
}

impl AstNode for EntityTag {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::CharacterLiteral) => {
                let token = parser.consume().unwrap();
                let ch = token
                    .text
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                Ok(EntityTag::CharacterLiteral(ch))
            }
            Some(TokenKind::StringLiteral) => {
                let op = OperatorSymbol::parse(parser)?;
                Ok(EntityTag::OperatorSymbol(op))
            }
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let name = SimpleName::parse(parser)?;
                Ok(EntityTag::SimpleName(name))
            }
            _ => Err(parser
                .error("expected entity tag (simple name, character literal, or operator symbol)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EntityTag::SimpleName(name) => name.format(f, indent_level),
            EntityTag::CharacterLiteral(ch) => write!(f, "'{}'", ch),
            EntityTag::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}
