//! Object declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, write_indent};
use super::type_def::SubtypeIndication;
use crate::parser::{Parser, ParseError};

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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            SignalKind::Register => write!(f, "register"),
            SignalKind::Bus => write!(f, "bus"),
        }
    }
}

impl AstNode for VariableDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)
    }
}

impl AstNode for AliasDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            AliasDesignator::Identifier(id) => id.format(f, indent_level),
            AliasDesignator::CharacterLiteral(ch) => write!(f, "'{}'", ch),
            AliasDesignator::OperatorSymbol(op) => op.format(f, indent_level),
        }
    }
}
