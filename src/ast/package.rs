//! Package declaration and body AST nodes.

use super::common::*;
use super::interface::GenericMapAspect;
use super::name::Name;

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
