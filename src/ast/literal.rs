//! Literal-related AST nodes.

use super::common::*;
use super::name::Name;
use super::node::AstNode;
use crate::parser::{Parser, ParseError};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for Literal {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Literal::Numeric(n) => n.format(f, indent_level),
            Literal::Enumeration(e) => e.format(f, indent_level),
            Literal::String(s) => s.format(f, indent_level),
            Literal::BitString(b) => b.format(f, indent_level),
            Literal::Null => write!(f, "null"),
        }
    }
}

impl AstNode for NumericLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            NumericLiteral::Abstract(a) => a.format(f, indent_level),
            NumericLiteral::Physical(p) => p.format(f, indent_level),
        }
    }
}

impl AstNode for AbstractLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            AbstractLiteral::Decimal(d) => d.format(f, indent_level),
            AbstractLiteral::Based(b) => b.format(f, indent_level),
        }
    }
}

impl AstNode for DecimalLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for BasedLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for PhysicalLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(ref value) = self.value {
            value.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.unit_name.format(f, indent_level)
    }
}

impl AstNode for EnumerationLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EnumerationLiteral::Identifier(id) => id.format(f, indent_level),
            EnumerationLiteral::CharacterLiteral(c) => write!(f, "'{}'", c),
        }
    }
}

impl AstNode for StringLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

impl AstNode for BitStringLiteral {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for BaseSpecifier {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        let s = match self {
            BaseSpecifier::B => "b",
            BaseSpecifier::O => "o",
            BaseSpecifier::X => "x",
            BaseSpecifier::UB => "ub",
            BaseSpecifier::UO => "uo",
            BaseSpecifier::UX => "ux",
            BaseSpecifier::SB => "sb",
            BaseSpecifier::SO => "so",
            BaseSpecifier::SX => "sx",
            BaseSpecifier::D => "d",
        };
        write!(f, "{}", s)
    }
}
