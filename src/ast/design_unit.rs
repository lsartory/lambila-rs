//! Top-level design file AST nodes.

use super::common::*;

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
