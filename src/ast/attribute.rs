//! Attribute declaration and specification AST nodes.

use super::common::*;
use super::expression::Expression;
use super::type_def::TypeMark;

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
