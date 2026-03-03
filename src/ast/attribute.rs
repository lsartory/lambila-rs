//! Attribute declaration and specification AST nodes.

use super::common::*;
use super::expression::Expression;
use super::node::{AstNode, write_indent, format_comma_separated};
use super::type_def::TypeMark;
use crate::parser::{Parser, ParseError};

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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name_list.format(f, indent_level)?;
        write!(f, " : ")?;
        self.entity_class.format(f, indent_level)
    }
}

impl AstNode for EntityClass {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.entries, f, indent_level)
    }
}

impl AstNode for EntityDesignator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EntityTag::SimpleName(name) => name.format(f, indent_level),
            EntityTag::CharacterLiteral(ch) => write!(f, "'{}'", ch),
            EntityTag::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}
