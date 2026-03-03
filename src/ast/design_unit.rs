//! Top-level design file AST nodes.

use super::common::*;
use super::node::{AstNode, write_indent, format_lines};
use crate::parser::{Parser, ParseError};

/// EBNF: `design_file ::= design_unit { design_unit }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignFile {
    pub design_units: Vec<DesignUnit>,
}

/// EBNF: `design_unit ::= context_clause library_unit`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignUnit {
    pub context_clause: super::context::ContextClause,
    pub library_unit: LibraryUnit,
}

/// EBNF: `library_unit ::= primary_unit | secondary_unit`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryUnit {
    Primary(PrimaryUnit),
    Secondary(SecondaryUnit),
}

/// EBNF (VHDL-2008): `primary_unit ::= entity_declaration | configuration_declaration
///     | package_declaration | package_instantiation_declaration | context_declaration
///     | PSL_Verification_Unit`
/// EBNF (VHDL-87/93): `primary_unit ::= entity_declaration | configuration_declaration
///     | package_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryUnit {
    Entity(Box<super::entity::EntityDeclaration>),
    Configuration(Box<super::configuration::ConfigurationDeclaration>),
    Package(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageInstantiation(Box<super::package::PackageInstantiationDeclaration>),
    /// VHDL-2008.
    Context(Box<super::context::ContextDeclaration>),
}

/// EBNF: `secondary_unit ::= architecture_body | package_body`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecondaryUnit {
    Architecture(Box<super::architecture::ArchitectureBody>),
    PackageBody(Box<super::package::PackageBody>),
}

/// EBNF (VHDL-2008): `declaration ::= type_declaration | subtype_declaration
///     | object_declaration | interface_declaration | alias_declaration
///     | attribute_declaration | component_declaration | group_template_declaration
///     | group_declaration | entity_declaration | configuration_declaration
///     | subprogram_declaration | package_declaration`
/// EBNF (VHDL-87): omits group_template_declaration and group_declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Type(Box<super::type_def::TypeDeclaration>),
    Subtype(Box<super::type_def::SubtypeDeclaration>),
    Object(Box<super::object_decl::ObjectDeclaration>),
    Interface(Box<super::interface::InterfaceDeclaration>),
    Alias(Box<super::object_decl::AliasDeclaration>),
    Attribute(Box<super::attribute::AttributeDeclaration>),
    Component(Box<super::component::ComponentDeclaration>),
    /// VHDL-93+.
    GroupTemplate(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    Group(Box<super::group::GroupDeclaration>),
    Entity(Box<super::entity::EntityDeclaration>),
    Configuration(Box<super::configuration::ConfigurationDeclaration>),
    Subprogram(Box<super::subprogram::SubprogramDeclaration>),
    Package(Box<super::package::PackageDeclaration>),
}

/// EBNF: `tool_directive ::= ` identifier { graphic_character }` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDirective {
    pub identifier: Identifier,
    pub content: String,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for DesignFile {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, unit) in self.design_units.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            unit.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for DesignUnit {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.context_clause.items, f, indent_level)?;
        self.library_unit.format(f, indent_level)
    }
}

impl AstNode for LibraryUnit {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            LibraryUnit::Primary(inner) => inner.format(f, indent_level),
            LibraryUnit::Secondary(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for PrimaryUnit {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            PrimaryUnit::Entity(inner) => inner.format(f, indent_level),
            PrimaryUnit::Configuration(inner) => inner.format(f, indent_level),
            PrimaryUnit::Package(inner) => inner.format(f, indent_level),
            PrimaryUnit::PackageInstantiation(inner) => inner.format(f, indent_level),
            PrimaryUnit::Context(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for SecondaryUnit {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            SecondaryUnit::Architecture(inner) => inner.format(f, indent_level),
            SecondaryUnit::PackageBody(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for Declaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Declaration::Type(inner) => inner.format(f, indent_level),
            Declaration::Subtype(inner) => inner.format(f, indent_level),
            Declaration::Object(inner) => inner.format(f, indent_level),
            Declaration::Interface(inner) => inner.format(f, indent_level),
            Declaration::Alias(inner) => inner.format(f, indent_level),
            Declaration::Attribute(inner) => inner.format(f, indent_level),
            Declaration::Component(inner) => inner.format(f, indent_level),
            Declaration::GroupTemplate(inner) => inner.format(f, indent_level),
            Declaration::Group(inner) => inner.format(f, indent_level),
            Declaration::Entity(inner) => inner.format(f, indent_level),
            Declaration::Configuration(inner) => inner.format(f, indent_level),
            Declaration::Subprogram(inner) => inner.format(f, indent_level),
            Declaration::Package(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ToolDirective {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "`")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " {}", self.content)
    }
}
