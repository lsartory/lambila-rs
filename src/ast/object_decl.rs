//! Object declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::type_def::SubtypeIndication;

/// EBNF: `object_declaration ::= constant_declaration | signal_declaration
///     | variable_declaration | file_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectDeclaration {
    Constant(ConstantDeclaration),
    Signal(SignalDeclaration),
    Variable(VariableDeclaration),
    File(FileDeclaration),
}

/// EBNF: `constant_declaration ::= CONSTANT identifier_list : subtype_indication
///     [ := expression ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantDeclaration {
    pub identifiers: IdentifierList,
    pub subtype_indication: SubtypeIndication,
    pub default_expression: Option<Expression>,
}

/// EBNF: `signal_declaration ::= SIGNAL identifier_list : subtype_indication
///     [ signal_kind ] [ := expression ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDeclaration {
    pub identifiers: IdentifierList,
    pub subtype_indication: SubtypeIndication,
    pub signal_kind: Option<SignalKind>,
    pub default_expression: Option<Expression>,
}

/// EBNF: `signal_kind ::= REGISTER | BUS`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    Register,
    Bus,
}

/// EBNF (VHDL-93+): `variable_declaration ::= [ SHARED ] VARIABLE identifier_list :
///     subtype_indication [ := expression ] ;`
/// EBNF (VHDL-87): `variable_declaration ::= VARIABLE identifier_list :
///     subtype_indication [ := expression ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDeclaration {
    /// VHDL-93+.
    pub shared: bool,
    pub identifiers: IdentifierList,
    pub subtype_indication: SubtypeIndication,
    pub default_expression: Option<Expression>,
}

/// EBNF (VHDL-93+): `file_declaration ::= FILE identifier_list : subtype_indication
///     [ file_open_information ] ;`
/// EBNF (VHDL-87): `file_declaration ::= FILE identifier_list : subtype_indication
///     IS [ mode ] file_logical_name ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDeclaration {
    pub identifiers: IdentifierList,
    pub subtype_indication: SubtypeIndication,
    /// VHDL-93+ open information.
    pub open_information: Option<FileOpenInformation>,
    /// VHDL-87 mode for file declaration.
    pub mode: Option<Mode>,
    /// VHDL-87 logical name.
    pub logical_name: Option<FileLogicalName>,
}

/// EBNF: `file_open_information ::= [ OPEN file_open_kind_expression ] IS file_logical_name`
/// (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileOpenInformation {
    pub open_kind: Option<Expression>,
    pub logical_name: FileLogicalName,
}

/// EBNF: `file_logical_name ::= string_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileLogicalName {
    pub expression: Expression,
}

/// EBNF (VHDL-93+): `alias_declaration ::= ALIAS alias_designator
///     [ : subtype_indication ] IS name [ signature ] ;`
/// EBNF (VHDL-87): `alias_declaration ::= ALIAS identifier : subtype_indication IS name ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDeclaration {
    pub designator: AliasDesignator,
    pub subtype_indication: Option<SubtypeIndication>,
    pub name: Name,
    /// VHDL-93+.
    pub signature: Option<Signature>,
}

/// EBNF (VHDL-93+): `alias_designator ::= identifier | character_literal | operator_symbol`
/// EBNF (VHDL-87): just `identifier`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasDesignator {
    Identifier(Identifier),
    /// VHDL-93+.
    CharacterLiteral(String),
    /// VHDL-93+.
    OperatorSymbol(OperatorSymbol),
}
