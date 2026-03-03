//! Literal-related AST nodes.

use super::common::*;
use super::name::Name;

/// A VHDL literal.
///
/// EBNF: `literal ::= numeric_literal | enumeration_literal | string_literal
///     | bit_string_literal | NULL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Numeric(NumericLiteral),
    Enumeration(EnumerationLiteral),
    String(StringLiteral),
    BitString(BitStringLiteral),
    Null,
}

/// EBNF: `numeric_literal ::= abstract_literal | physical_literal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericLiteral {
    Abstract(AbstractLiteral),
    Physical(PhysicalLiteral),
}

/// EBNF: `abstract_literal ::= decimal_literal | based_literal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbstractLiteral {
    Decimal(DecimalLiteral),
    Based(BasedLiteral),
}

/// EBNF: `decimal_literal ::= integer [ . integer ] [ exponent ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecimalLiteral {
    pub text: String,
}

/// EBNF: `based_literal ::= base # based_integer [ . based_integer ] # [ exponent ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasedLiteral {
    pub text: String,
}

/// EBNF: `physical_literal ::= [ abstract_literal ] unit_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalLiteral {
    pub value: Option<AbstractLiteral>,
    pub unit_name: Name,
}

/// EBNF: `enumeration_literal ::= identifier | character_literal`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumerationLiteral {
    Identifier(Identifier),
    CharacterLiteral(String),
}

/// A string literal.
///
/// EBNF: `string_literal ::= " { graphic_character } "`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteral {
    pub text: String,
}

/// A bit string literal.
///
/// EBNF (VHDL-2008): `bit_string_literal ::= [ integer ] base_specifier " [ bit_value ] "`
/// EBNF (VHDL-87/93): `bit_string_literal ::= base_specifier " bit_value "`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitStringLiteral {
    pub text: String,
}

/// Base specifier for based and bit-string literals.
///
/// EBNF (VHDL-2008): `base_specifier ::= B | O | X | UB | UO | UX | SB | SO | SX | D`
/// EBNF (VHDL-87/93): `base_specifier ::= B | O | X`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseSpecifier {
    B,
    O,
    X,
    /// Unsigned binary (VHDL-2008).
    UB,
    /// Unsigned octal (VHDL-2008).
    UO,
    /// Unsigned hex (VHDL-2008).
    UX,
    /// Signed binary (VHDL-2008).
    SB,
    /// Signed octal (VHDL-2008).
    SO,
    /// Signed hex (VHDL-2008).
    SX,
    /// Decimal (VHDL-2008).
    D,
}
