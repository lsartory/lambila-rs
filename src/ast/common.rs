//! Common building-block types used throughout the AST.

use super::node::AstNode;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Identifier) => {
                let token = parser.consume().unwrap();
                Ok(Identifier::Basic(token.text.clone()))
            }
            Some(TokenKind::ExtendedIdentifier) => {
                let token = parser.consume().unwrap();
                // Strip the surrounding backslashes from the extended identifier text
                let inner = token
                    .text
                    .trim_start_matches('\\')
                    .trim_end_matches('\\')
                    .to_string();
                Ok(Identifier::Extended(inner))
            }
            _ => Err(parser.error("expected identifier")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Identifier::Basic(s) => write!(f, "{}", s),
            Identifier::Extended(s) => write!(f, "\\{}\\", s),
        }
    }
}

impl AstNode for IdentifierList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut identifiers = vec![Identifier::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            identifiers.push(Identifier::parse(parser)?);
        }
        Ok(IdentifierList { identifiers })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifier = Identifier::parse(parser)?;
        Ok(Label { identifier })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for SimpleName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifier = Identifier::parse(parser)?;
        Ok(SimpleName { identifier })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for OperatorSymbol {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let token = parser.expect(TokenKind::StringLiteral)?;
        // Strip the surrounding double quotes from the string literal text
        let text = token.text.clone();
        let inner = text
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
        Ok(OperatorSymbol { text: inner })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

impl AstNode for Designator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::StringLiteral) => {
                let op = OperatorSymbol::parse(parser)?;
                Ok(Designator::OperatorSymbol(op))
            }
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let id = Identifier::parse(parser)?;
                Ok(Designator::Identifier(id))
            }
            _ => Err(parser.error("expected designator (identifier or operator symbol)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Designator::Identifier(id) => id.format(f, indent_level),
            Designator::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}

impl AstNode for Signature {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use super::type_def::TypeMark;

        parser.expect(TokenKind::LeftBracket)?;

        let mut parameter_types = Vec::new();
        let mut return_type = None;

        // Check if we immediately see RETURN or `]`
        if !parser.at(TokenKind::RightBracket) && !parser.at_keyword(KeywordKind::Return) {
            // Parse the first type_mark
            parameter_types.push(TypeMark::parse(parser)?);
            // Parse additional comma-separated type_marks
            while parser.consume_if(TokenKind::Comma).is_some() {
                parameter_types.push(TypeMark::parse(parser)?);
            }
        }

        // Optional RETURN type_mark
        if parser.consume_if_keyword(KeywordKind::Return).is_some() {
            return_type = Some(TypeMark::parse(parser)?);
        }

        parser.expect(TokenKind::RightBracket)?;

        Ok(Signature {
            parameter_types,
            return_type,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::In).is_some() {
            Ok(Mode::In)
        } else if parser.consume_if_keyword(KeywordKind::Out).is_some() {
            Ok(Mode::Out)
        } else if parser.consume_if_keyword(KeywordKind::Inout).is_some() {
            Ok(Mode::InOut)
        } else if parser.consume_if_keyword(KeywordKind::Buffer).is_some() {
            Ok(Mode::Buffer)
        } else if parser.consume_if_keyword(KeywordKind::Linkage).is_some() {
            Ok(Mode::Linkage)
        } else {
            Err(parser.error("expected mode (in, out, inout, buffer, or linkage)"))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::To).is_some() {
            Ok(Direction::To)
        } else if parser.consume_if_keyword(KeywordKind::Downto).is_some() {
            Ok(Direction::Downto)
        } else {
            Err(parser.error("expected direction (to or downto)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Direction::To => write!(f, "to"),
            Direction::Downto => write!(f, "downto"),
        }
    }
}

impl AstNode for Sign {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if(TokenKind::Plus).is_some() {
            Ok(Sign::Plus)
        } else if parser.consume_if(TokenKind::Minus).is_some() {
            Ok(Sign::Minus)
        } else {
            Err(parser.error("expected sign (+ or -)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Sign::Plus => write!(f, "+"),
            Sign::Minus => write!(f, "-"),
        }
    }
}
