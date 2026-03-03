//! Literal-related AST nodes.

use super::common::*;
use super::name::Name;
use super::node::AstNode;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            // NULL literal
            Some(TokenKind::Keyword(KeywordKind::Null)) => {
                parser.consume();
                Ok(Literal::Null)
            }
            // Bit string literal
            Some(TokenKind::BitStringLiteral) => {
                let bsl = BitStringLiteral::parse(parser)?;
                Ok(Literal::BitString(bsl))
            }
            // String literal
            Some(TokenKind::StringLiteral) => {
                let sl = StringLiteral::parse(parser)?;
                Ok(Literal::String(sl))
            }
            // Numeric literals (decimal, real, or based) -- may also be physical
            Some(TokenKind::IntegerLiteral)
            | Some(TokenKind::RealLiteral)
            | Some(TokenKind::BasedLiteral) => {
                let num = NumericLiteral::parse(parser)?;
                Ok(Literal::Numeric(num))
            }
            // Character literal (enumeration literal)
            Some(TokenKind::CharacterLiteral) => {
                let el = EnumerationLiteral::parse(parser)?;
                Ok(Literal::Enumeration(el))
            }
            // Identifier (enumeration literal)
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let el = EnumerationLiteral::parse(parser)?;
                Ok(Literal::Enumeration(el))
            }
            _ => Err(parser.error("expected literal")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Parse the abstract literal first
        let abstract_lit = AbstractLiteral::parse(parser)?;

        // If the next token is an identifier, this is a physical literal (abstract + unit name)
        match parser.peek_kind() {
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let identifier = Identifier::parse(parser)?;
                let unit_name = Name::Simple(SimpleName { identifier });
                Ok(NumericLiteral::Physical(PhysicalLiteral {
                    value: Some(abstract_lit),
                    unit_name,
                }))
            }
            _ => Ok(NumericLiteral::Abstract(abstract_lit)),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            NumericLiteral::Abstract(a) => a.format(f, indent_level),
            NumericLiteral::Physical(p) => p.format(f, indent_level),
        }
    }
}

impl AstNode for AbstractLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::BasedLiteral) => {
                let based = BasedLiteral::parse(parser)?;
                Ok(AbstractLiteral::Based(based))
            }
            Some(TokenKind::IntegerLiteral) | Some(TokenKind::RealLiteral) => {
                let decimal = DecimalLiteral::parse(parser)?;
                Ok(AbstractLiteral::Decimal(decimal))
            }
            _ => Err(parser.error("expected abstract literal (decimal or based)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            AbstractLiteral::Decimal(d) => d.format(f, indent_level),
            AbstractLiteral::Based(b) => b.format(f, indent_level),
        }
    }
}

impl AstNode for DecimalLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::IntegerLiteral) | Some(TokenKind::RealLiteral) => {
                let token = parser.consume().unwrap();
                Ok(DecimalLiteral {
                    text: token.text.clone(),
                })
            }
            _ => Err(parser.error("expected decimal literal (integer or real)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for BasedLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let token = parser.expect(TokenKind::BasedLiteral)?;
        Ok(BasedLiteral {
            text: token.text.clone(),
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for PhysicalLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Optional abstract literal prefix
        let value = match parser.peek_kind() {
            Some(TokenKind::IntegerLiteral)
            | Some(TokenKind::RealLiteral)
            | Some(TokenKind::BasedLiteral) => Some(AbstractLiteral::parse(parser)?),
            _ => None,
        };

        // Unit name (an identifier, represented as a Name)
        let identifier = Identifier::parse(parser)?;
        let unit_name = Name::Simple(SimpleName { identifier });

        Ok(PhysicalLiteral { value, unit_name })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::CharacterLiteral) => {
                let token = parser.consume().unwrap();
                // Strip surrounding single quotes from the character literal text
                let inner = token
                    .text
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                Ok(EnumerationLiteral::CharacterLiteral(inner))
            }
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let id = Identifier::parse(parser)?;
                Ok(EnumerationLiteral::Identifier(id))
            }
            _ => {
                Err(parser.error("expected enumeration literal (identifier or character literal)"))
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EnumerationLiteral::Identifier(id) => id.format(f, indent_level),
            EnumerationLiteral::CharacterLiteral(c) => write!(f, "'{}'", c),
        }
    }
}

impl AstNode for StringLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let token = parser.expect(TokenKind::StringLiteral)?;
        // Strip surrounding double quotes from the string literal text
        let inner = token
            .text
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
        Ok(StringLiteral { text: inner })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

impl AstNode for BitStringLiteral {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let token = parser.expect(TokenKind::BitStringLiteral)?;
        Ok(BitStringLiteral {
            text: token.text.clone(),
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl AstNode for BaseSpecifier {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // BaseSpecifier is parsed from the text of a BitStringLiteral token.
        // The format is: [integer] base_specifier " bit_value "
        // We extract the base specifier letters that appear before the first '"'.
        let token = parser.expect(TokenKind::BitStringLiteral)?;
        let text = token.text.clone();

        // Find the position of the first '"' to isolate the prefix
        let quote_pos = text
            .find('"')
            .ok_or_else(|| parser.error("invalid bit string literal: missing '\"'"))?;
        let prefix = &text[..quote_pos];

        // Strip any leading digits (optional integer length in VHDL-2008)
        let spec_str = prefix.trim_start_matches(|c: char| c.is_ascii_digit());

        match spec_str.to_lowercase().as_str() {
            "b" => Ok(BaseSpecifier::B),
            "o" => Ok(BaseSpecifier::O),
            "x" => Ok(BaseSpecifier::X),
            "ub" => Ok(BaseSpecifier::UB),
            "uo" => Ok(BaseSpecifier::UO),
            "ux" => Ok(BaseSpecifier::UX),
            "sb" => Ok(BaseSpecifier::SB),
            "so" => Ok(BaseSpecifier::SO),
            "sx" => Ok(BaseSpecifier::SX),
            "d" => Ok(BaseSpecifier::D),
            _ => Err(parser.error(format!("unknown base specifier: '{}'", spec_str))),
        }
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
