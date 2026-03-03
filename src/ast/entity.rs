//! Entity declaration AST nodes.

use super::common::*;
use super::node::{AstNode, write_indent, format_lines};
use crate::parser::{Parser, ParseError};

/// EBNF (VHDL-2008): `entity_declaration ::= ENTITY identifier IS entity_header
///     entity_declarative_part [ BEGIN entity_statement_part ] END [ ENTITY ]
///     [ entity_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ entity_simple_name ] ;` (no optional ENTITY keyword).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDeclaration {
    pub identifier: Identifier,
    pub header: EntityHeader,
    pub declarative_part: EntityDeclarativePart,
    pub statement_part: Option<EntityStatementPart>,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `entity_header ::= [ formal_generic_clause ] [ formal_port_clause ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityHeader {
    pub generic_clause: Option<super::interface::GenericClause>,
    pub port_clause: Option<super::interface::PortClause>,
}

/// EBNF: `entity_declarative_part ::= { entity_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDeclarativePart {
    pub items: Vec<EntityDeclarativeItem>,
}

/// EBNF (VHDL-2008): `entity_declarative_item ::= subprogram_declaration | subprogram_body
///     | subprogram_instantiation_declaration | package_declaration | package_body
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | signal_declaration | shared_variable_declaration
///     | file_declaration | alias_declaration | attribute_declaration
///     | attribute_specification | disconnection_specification | use_clause
///     | group_template_declaration | group_declaration
///     | PSL_Property_Declaration | PSL_Sequence_Declaration | PSL_Clock_Declaration`
/// Earlier versions have fewer alternatives (no package_body, no subprogram_instantiation, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramBody(Box<super::subprogram::SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<super::package::PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<super::package::PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    SignalDeclaration(Box<super::object_decl::SignalDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    DisconnectionSpecification(Box<super::signal::DisconnectionSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `entity_statement_part ::= { entity_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityStatementPart {
    pub statements: Vec<EntityStatement>,
}

/// EBNF (VHDL-2008): `entity_statement ::= concurrent_assertion_statement
///     | passive_concurrent_procedure_call_statement | passive_process_statement
///     | PSL_PSL_Directive`
/// EBNF (VHDL-87/93): omits PSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityStatement {
    ConcurrentAssertion(Box<super::concurrent::ConcurrentAssertionStatement>),
    PassiveProcedureCall(Box<super::concurrent::ConcurrentProcedureCallStatement>),
    PassiveProcess(Box<super::concurrent::ProcessStatement>),
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for EntityDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "entity ")?;
        self.identifier.format(f, 0)?;
        writeln!(f, " is")?;
        self.header.format(f, indent_level + 1)?;
        self.declarative_part.format(f, indent_level + 1)?;
        if let Some(ref stmt_part) = self.statement_part {
            write_indent(f, indent_level)?;
            writeln!(f, "begin")?;
            stmt_part.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end entity")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for EntityHeader {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(ref gc) = self.generic_clause {
            gc.format(f, indent_level)?;
        }
        if let Some(ref pc) = self.port_clause {
            pc.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for EntityDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for EntityDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubprogramDeclaration(inner) => inner.format(f, indent_level),
            Self::SubprogramBody(inner) => inner.format(f, indent_level),
            Self::SubprogramInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageBody(inner) => inner.format(f, indent_level),
            Self::PackageInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::TypeDeclaration(inner) => inner.format(f, indent_level),
            Self::SubtypeDeclaration(inner) => inner.format(f, indent_level),
            Self::ConstantDeclaration(inner) => inner.format(f, indent_level),
            Self::SignalDeclaration(inner) => inner.format(f, indent_level),
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
            Self::FileDeclaration(inner) => inner.format(f, indent_level),
            Self::AliasDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::DisconnectionSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for EntityStatementPart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for EntityStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::ConcurrentAssertion(inner) => inner.format(f, indent_level),
            Self::PassiveProcedureCall(inner) => inner.format(f, indent_level),
            Self::PassiveProcess(inner) => inner.format(f, indent_level),
        }
    }
}
