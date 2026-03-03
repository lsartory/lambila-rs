//! Package declaration and body AST nodes.

use super::common::*;
use super::interface::GenericMapAspect;
use super::name::Name;
use super::node::{AstNode, write_indent, format_lines};
use crate::parser::{Parser, ParseError};

/// EBNF (VHDL-2008): `package_declaration ::= PACKAGE identifier IS package_header
///     package_declarative_part END [ PACKAGE ] [ package_simple_name ] ;`
/// EBNF (VHDL-87/93): `package_declaration ::= PACKAGE identifier IS
///     package_declarative_part END [ PACKAGE ] [ package_simple_name ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDeclaration {
    pub identifier: Identifier,
    /// VHDL-2008.
    pub header: Option<PackageHeader>,
    pub declarative_part: PackageDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `package_header ::= [ generic_clause [ generic_map_aspect ; ] ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageHeader {
    pub generic_clause: Option<super::interface::GenericClause>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

/// EBNF: `package_declarative_part ::= { package_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDeclarativePart {
    pub items: Vec<PackageDeclarativeItem>,
}

/// EBNF (VHDL-2008): `package_declarative_item ::= subprogram_declaration
///     | subprogram_instantiation_declaration | package_declaration
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | signal_declaration | variable_declaration | file_declaration
///     | alias_declaration | component_declaration | attribute_declaration
///     | attribute_specification | disconnection_specification | use_clause
///     | group_template_declaration | group_declaration
///     | PSL_Property_Declaration | PSL_Sequence_Declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<PackageDeclaration>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    SignalDeclaration(Box<super::object_decl::SignalDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    ComponentDeclaration(Box<super::component::ComponentDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    DisconnectionSpecification(Box<super::signal::DisconnectionSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF (VHDL-2008): `package_body ::= PACKAGE BODY package_simple_name IS
///     package_body_declarative_part END [ PACKAGE BODY ] [ package_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ package_simple_name ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageBody {
    pub name: SimpleName,
    pub declarative_part: PackageBodyDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `package_body_declarative_part ::= { package_body_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageBodyDeclarativePart {
    pub items: Vec<PackageBodyDeclarativeItem>,
}

/// EBNF (VHDL-2008): `package_body_declarative_item ::= subprogram_declaration
///     | subprogram_body | subprogram_instantiation_declaration | package_declaration
///     | package_body | package_instantiation_declaration | type_declaration
///     | subtype_declaration | constant_declaration | variable_declaration | file_declaration
///     | alias_declaration | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageBodyDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramBody(Box<super::subprogram::SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    /// VHDL-2008.
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    /// VHDL-2008.
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `package_instantiation_declaration ::= PACKAGE identifier IS NEW
///     uninstantiated_package_name [ generic_map_aspect ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInstantiationDeclaration {
    pub identifier: Identifier,
    pub package_name: Box<Name>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for PackageDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package ")?;
        self.identifier.format(f, indent_level)?;
        writeln!(f, " is")?;
        if let Some(header) = &self.header {
            header.format(f, indent_level + 1)?;
        }
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end package")?;
        if let Some(end_name) = &self.end_name {
            write!(f, " ")?;
            end_name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for PackageHeader {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(generic_clause) = &self.generic_clause {
            generic_clause.format(f, indent_level)?;
        }
        if let Some(generic_map_aspect) = &self.generic_map_aspect {
            generic_map_aspect.format(f, indent_level)?;
            writeln!(f, ";")?;
        }
        Ok(())
    }
}

impl AstNode for PackageDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for PackageDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubprogramDeclaration(inner) => inner.format(f, indent_level),
            Self::SubprogramInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::TypeDeclaration(inner) => inner.format(f, indent_level),
            Self::SubtypeDeclaration(inner) => inner.format(f, indent_level),
            Self::ConstantDeclaration(inner) => inner.format(f, indent_level),
            Self::SignalDeclaration(inner) => inner.format(f, indent_level),
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
            Self::VariableDeclaration(inner) => inner.format(f, indent_level),
            Self::FileDeclaration(inner) => inner.format(f, indent_level),
            Self::AliasDeclaration(inner) => inner.format(f, indent_level),
            Self::ComponentDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::DisconnectionSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for PackageBody {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package body ")?;
        self.name.format(f, indent_level)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end package body")?;
        if let Some(end_name) = &self.end_name {
            write!(f, " ")?;
            end_name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for PackageBodyDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for PackageBodyDeclarativeItem {
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
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
            Self::VariableDeclaration(inner) => inner.format(f, indent_level),
            Self::FileDeclaration(inner) => inner.format(f, indent_level),
            Self::AliasDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for PackageInstantiationDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is new ")?;
        self.package_name.format(f, indent_level)?;
        if let Some(generic_map) = &self.generic_map_aspect {
            writeln!(f)?;
            generic_map.format(f, indent_level + 1)?;
        }
        writeln!(f, ";")
    }
}
