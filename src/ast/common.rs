//! Common building-block types used throughout the AST.

use super::node::AstNode;
use crate::parser::{Parser, ParseError};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for Identifier {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Identifier::Basic(s) => write!(f, "{}", s),
            Identifier::Extended(s) => write!(f, "\\{}\\", s),
        }
    }
}

impl AstNode for IdentifierList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, id) in self.identifiers.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            id.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for Label {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for SimpleName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for OperatorSymbol {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

impl AstNode for Designator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Designator::Identifier(id) => id.format(f, indent_level),
            Designator::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}

impl AstNode for Signature {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, param) in self.parameter_types.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            param.format(f, indent_level)?;
        }
        if let Some(ref ret) = self.return_type {
            if !self.parameter_types.is_empty() {
                write!(f, " ")?;
            }
            write!(f, "return ")?;
            ret.format(f, indent_level)?;
        }
        write!(f, "]")
    }
}

impl AstNode for Mode {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Mode::In => write!(f, "in"),
            Mode::Out => write!(f, "out"),
            Mode::InOut => write!(f, "inout"),
            Mode::Buffer => write!(f, "buffer"),
            Mode::Linkage => write!(f, "linkage"),
        }
    }
}

impl AstNode for Direction {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Direction::To => write!(f, "to"),
            Direction::Downto => write!(f, "downto"),
        }
    }
}

impl AstNode for Sign {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Sign::Plus => write!(f, "+"),
            Sign::Minus => write!(f, "-"),
        }
    }
}
