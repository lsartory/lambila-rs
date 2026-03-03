//! Name-related AST nodes.

use super::common::*;
use super::expression::Expression;
use super::type_def::{DiscreteRange, SubtypeIndication};

/// A VHDL name.
///
/// EBNF (VHDL-2008): `name ::= simple_name | operator_symbol | character_literal
///     | selected_name | indexed_name | slice_name | attribute_name | external_name`
///
/// VHDL-87/93 omit `external_name` and (87 omits) `character_literal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Simple(SimpleName),
    OperatorSymbol(OperatorSymbol),
    CharacterLiteral(String),
    Selected(Box<SelectedName>),
    Indexed(Box<IndexedName>),
    Slice(Box<SliceName>),
    Attribute(Box<AttributeName>),
    /// VHDL-2008 external name.
    External(Box<ExternalName>),
}

/// A prefix for selected / indexed / slice / attribute names.
///
/// EBNF: `prefix ::= name | function_call`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix {
    Name(Box<Name>),
    FunctionCall(Box<FunctionCall>),
}

/// A suffix used in selected names.
///
/// EBNF: `suffix ::= simple_name | character_literal | operator_symbol | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suffix {
    SimpleName(SimpleName),
    CharacterLiteral(String),
    OperatorSymbol(OperatorSymbol),
    All,
}

/// A selected name (dot notation).
///
/// EBNF: `selected_name ::= prefix . suffix`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedName {
    pub prefix: Prefix,
    pub suffix: Suffix,
}

/// An indexed name.
///
/// EBNF: `indexed_name ::= prefix ( expression { , expression } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedName {
    pub prefix: Prefix,
    pub expressions: Vec<Expression>,
}

/// A slice name.
///
/// EBNF: `slice_name ::= prefix ( discrete_range )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceName {
    pub prefix: Prefix,
    pub discrete_range: DiscreteRange,
}

/// An attribute designator.
///
/// EBNF: `attribute_designator ::= attribute_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDesignator {
    pub name: SimpleName,
}

/// An attribute name.
///
/// EBNF (VHDL-2008): `attribute_name ::= prefix [ signature ] ' attribute_designator [ ( expression ) ]`
///
/// VHDL-87: omits optional signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeName {
    pub prefix: Prefix,
    /// VHDL-93+ only.
    pub signature: Option<Signature>,
    pub designator: AttributeDesignator,
    pub expression: Option<Box<Expression>>,
}

/// An external name (VHDL-2008).
///
/// EBNF: `external_name ::= external_constant_name | external_signal_name | external_variable_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalName {
    Constant(ExternalConstantName),
    Signal(ExternalSignalName),
    Variable(ExternalVariableName),
}

/// EBNF: `external_constant_name ::= << CONSTANT external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalConstantName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_signal_name ::= << SIGNAL external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSignalName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_variable_name ::= << VARIABLE external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalVariableName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_pathname ::= package_pathname | absolute_pathname | relative_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalPathname {
    Package(PackagePathname),
    Absolute(AbsolutePathname),
    Relative(RelativePathname),
}

/// EBNF: `package_pathname ::= @ library_logical_name . { package_simple_name . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePathname {
    pub library_name: Identifier,
    pub package_names: Vec<SimpleName>,
    pub object_name: SimpleName,
}

/// EBNF: `absolute_pathname ::= . partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsolutePathname {
    pub partial: PartialPathname,
}

/// EBNF: `relative_pathname ::= { ^ . } partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePathname {
    pub up_count: usize,
    pub partial: PartialPathname,
}

/// EBNF: `partial_pathname ::= { pathname_element . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPathname {
    pub elements: Vec<PathnameElement>,
    pub object_name: SimpleName,
}

/// EBNF: `pathname_element ::= entity_simple_name | component_instantiation_label
///     | block_label | generate_statement_label [ ( static_expression ) ] | package_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathnameElement {
    pub name: Identifier,
    /// Present for generate_statement_label with an index expression.
    pub expression: Option<Box<Expression>>,
}

/// A function call.
///
/// EBNF: `function_call ::= function_name [ ( actual_parameter_part ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub function_name: Box<Name>,
    pub parameters: Option<super::association::ActualParameterPart>,
}
