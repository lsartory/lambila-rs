//! Object declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, write_indent};
use super::type_def::SubtypeIndication;
use crate::TokenKind;
use crate::parser::{ParseError, Parser};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ObjectDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Constant)) => Ok(ObjectDeclaration::Constant(
                ConstantDeclaration::parse(parser)?,
            )),
            Some(TokenKind::Keyword(KeywordKind::Signal)) => {
                Ok(ObjectDeclaration::Signal(SignalDeclaration::parse(parser)?))
            }
            Some(TokenKind::Keyword(KeywordKind::Variable)) => Ok(ObjectDeclaration::Variable(
                VariableDeclaration::parse(parser)?,
            )),
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                // SHARED VARIABLE
                Ok(ObjectDeclaration::Variable(VariableDeclaration::parse(
                    parser,
                )?))
            }
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                Ok(ObjectDeclaration::File(FileDeclaration::parse(parser)?))
            }
            _ => {
                Err(parser
                    .error("expected object declaration (constant, signal, variable, or file)"))
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ObjectDeclaration::Constant(inner) => inner.format(f, indent_level),
            ObjectDeclaration::Signal(inner) => inner.format(f, indent_level),
            ObjectDeclaration::Variable(inner) => inner.format(f, indent_level),
            ObjectDeclaration::File(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ConstantDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        parser.expect_keyword(KeywordKind::Constant)?;
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConstantDeclaration {
            identifiers,
            subtype_indication,
            default_expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "constant ")?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SignalDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        parser.expect_keyword(KeywordKind::Signal)?;
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        let signal_kind =
            if parser.at_keyword(KeywordKind::Register) || parser.at_keyword(KeywordKind::Bus) {
                Some(SignalKind::parse(parser)?)
            } else {
                None
            };
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(SignalDeclaration {
            identifiers,
            subtype_indication,
            signal_kind,
            default_expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "signal ")?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        if let Some(kind) = &self.signal_kind {
            write!(f, " ")?;
            kind.format(f, indent_level)?;
        }
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SignalKind {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        if parser.consume_if_keyword(KeywordKind::Register).is_some() {
            Ok(SignalKind::Register)
        } else if parser.consume_if_keyword(KeywordKind::Bus).is_some() {
            Ok(SignalKind::Bus)
        } else {
            Err(parser.error("expected signal kind (register or bus)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            SignalKind::Register => write!(f, "register"),
            SignalKind::Bus => write!(f, "bus"),
        }
    }
}

impl AstNode for VariableDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        let shared = parser.consume_if_keyword(KeywordKind::Shared).is_some();
        parser.expect_keyword(KeywordKind::Variable)?;
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(VariableDeclaration {
            shared,
            identifiers,
            subtype_indication,
            default_expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if self.shared {
            write!(f, "shared ")?;
        }
        write!(f, "variable ")?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for FileDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        parser.expect_keyword(KeywordKind::File)?;
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;

        // Try to parse VHDL-93+ file_open_information: [ OPEN expression ] IS file_logical_name
        // or VHDL-87: IS [ mode ] file_logical_name
        let mut open_information = None;
        let mut mode = None;
        let mut logical_name = None;

        if parser.at_keyword(KeywordKind::Open) || parser.at_keyword(KeywordKind::Is) {
            // Could be VHDL-93+ file_open_information or VHDL-87 form
            // VHDL-93+: [ OPEN expression ] IS file_logical_name
            // VHDL-87:  IS [ mode ] file_logical_name
            // Both share the IS keyword. Distinguish by checking if OPEN comes first.
            if parser.at_keyword(KeywordKind::Open) {
                // VHDL-93+ form with OPEN
                open_information = Some(FileOpenInformation::parse(parser)?);
            } else {
                // Starts with IS — could be either VHDL-87 or VHDL-93+ (without OPEN)
                // VHDL-87: IS [ mode ] file_logical_name
                // VHDL-93+: IS file_logical_name
                // Try to detect VHDL-87 mode after IS
                let save = parser.save();
                parser.expect_keyword(KeywordKind::Is)?;

                // Check if next token is a mode keyword (IN, OUT, INOUT, BUFFER, LINKAGE)
                if parser.at_keyword(KeywordKind::In)
                    || parser.at_keyword(KeywordKind::Out)
                    || parser.at_keyword(KeywordKind::Inout)
                    || parser.at_keyword(KeywordKind::Buffer)
                    || parser.at_keyword(KeywordKind::Linkage)
                {
                    // VHDL-87 form
                    mode = Some(Mode::parse(parser)?);
                    logical_name = Some(FileLogicalName::parse(parser)?);
                } else {
                    // VHDL-93+ form: IS file_logical_name (no OPEN)
                    parser.restore(save);
                    open_information = Some(FileOpenInformation::parse(parser)?);
                }
            }
        }

        parser.expect(TokenKind::Semicolon)?;
        Ok(FileDeclaration {
            identifiers,
            subtype_indication,
            open_information,
            mode,
            logical_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "file ")?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        if let Some(open_info) = &self.open_information {
            write!(f, " ")?;
            open_info.format(f, indent_level)?;
        }
        if let Some(mode) = &self.mode {
            write!(f, " ")?;
            mode.format(f, indent_level)?;
        }
        if let Some(logical_name) = &self.logical_name {
            write!(f, " ")?;
            logical_name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for FileOpenInformation {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        // [ OPEN file_open_kind_expression ] IS file_logical_name
        let open_kind = if parser.consume_if_keyword(KeywordKind::Open).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        parser.expect_keyword(KeywordKind::Is)?;
        let logical_name = FileLogicalName::parse(parser)?;
        Ok(FileOpenInformation {
            open_kind,
            logical_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(open_kind) = &self.open_kind {
            write!(f, "open ")?;
            open_kind.format(f, indent_level)?;
            write!(f, " ")?;
        }
        write!(f, "is ")?;
        self.logical_name.format(f, indent_level)
    }
}

impl AstNode for FileLogicalName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let expression = Expression::parse(parser)?;
        Ok(FileLogicalName { expression })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)
    }
}

impl AstNode for AliasDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        use crate::KeywordKind;
        parser.expect_keyword(KeywordKind::Alias)?;
        let designator = AliasDesignator::parse(parser)?;
        let subtype_indication = if parser.consume_if(TokenKind::Colon).is_some() {
            Some(SubtypeIndication::parse(parser)?)
        } else {
            None
        };
        parser.expect_keyword(KeywordKind::Is)?;
        let name = Name::parse(parser)?;
        // Optional signature (starts with `[`)
        let signature = if parser.at(TokenKind::LeftBracket) {
            Some(Signature::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(AliasDeclaration {
            designator,
            subtype_indication,
            name,
            signature,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "alias ")?;
        self.designator.format(f, indent_level)?;
        if let Some(subtype) = &self.subtype_indication {
            write!(f, " : ")?;
            subtype.format(f, indent_level)?;
        }
        write!(f, " is ")?;
        self.name.format(f, indent_level)?;
        if let Some(sig) = &self.signature {
            write!(f, " ")?;
            sig.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for AliasDesignator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::CharacterLiteral) => {
                let token = parser.consume().unwrap();
                // Strip surrounding quotes from character literal
                let ch = token
                    .text
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                Ok(AliasDesignator::CharacterLiteral(ch))
            }
            Some(TokenKind::StringLiteral) => {
                let op = OperatorSymbol::parse(parser)?;
                Ok(AliasDesignator::OperatorSymbol(op))
            }
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let id = Identifier::parse(parser)?;
                Ok(AliasDesignator::Identifier(id))
            }
            _ => Err(parser.error(
                "expected alias designator (identifier, character literal, or operator symbol)",
            )),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            AliasDesignator::Identifier(id) => id.format(f, indent_level),
            AliasDesignator::CharacterLiteral(ch) => write!(f, "'{}'", ch),
            AliasDesignator::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}
