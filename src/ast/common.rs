//! Common building-block types used throughout the AST.

/// An identifier — either basic or extended (VHDL-93+).
///
/// EBNF: `identifier ::= basic_identifier | extended_identifier`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    /// `basic_identifier ::= letter { [ underline ] letter_or_digit }`
    Basic(String),
    /// `extended_identifier ::= \ graphic_character { graphic_character } \` (VHDL-93+)
    Extended(String),
}

/// A list of identifiers.
///
/// EBNF: `identifier_list ::= identifier { , identifier }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierList {
    pub identifiers: Vec<Identifier>,
}

/// A label used on statements.
///
/// EBNF: `label ::= identifier`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub identifier: Identifier,
}

/// A simple name (just an identifier).
///
/// EBNF: `simple_name ::= identifier`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleName {
    pub identifier: Identifier,
}

/// An operator symbol (always a string literal).
///
/// EBNF: `operator_symbol ::= string_literal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorSymbol {
    pub text: String,
}

/// A designator — either an identifier or an operator symbol.
///
/// EBNF: `designator ::= identifier | operator_symbol`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Designator {
    Identifier(Identifier),
    OperatorSymbol(OperatorSymbol),
}

/// A type or subtype signature used for overload resolution.
///
/// EBNF: `signature ::= [ [ type_mark { , type_mark } ] [ RETURN type_mark ] ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub parameter_types: Vec<super::type_def::TypeMark>,
    pub return_type: Option<super::type_def::TypeMark>,
}

/// Port / signal / parameter mode.
///
/// EBNF: `mode ::= IN | OUT | INOUT | BUFFER | LINKAGE`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

/// Range direction.
///
/// EBNF: `direction ::= TO | DOWNTO`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    To,
    Downto,
}

/// Numeric sign.
///
/// EBNF: `sign ::= + | -`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}
