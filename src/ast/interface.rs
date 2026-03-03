//! Interface declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::type_def::{SubtypeIndication, TypeMark};

/// EBNF (VHDL-2008): `interface_declaration ::= interface_object_declaration
///     | interface_type_declaration | interface_subprogram_declaration
///     | interface_package_declaration`
/// EBNF (VHDL-87/93): `interface_declaration ::= interface_constant_declaration
///     | interface_signal_declaration | interface_variable_declaration
///     | interface_file_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceDeclaration {
    Object(InterfaceObjectDeclaration),
    /// VHDL-2008.
    Type(InterfaceTypeDeclaration),
    /// VHDL-2008.
    Subprogram(InterfaceSubprogramDeclaration),
    /// VHDL-2008.
    Package(InterfacePackageDeclaration),
}

/// An interface element.
///
/// EBNF: `interface_element ::= interface_declaration`
pub type InterfaceElement = InterfaceDeclaration;

/// An interface list.
///
/// EBNF: `interface_list ::= interface_element { ; interface_element }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceList {
    pub elements: Vec<InterfaceElement>,
}

/// EBNF (VHDL-2008): `interface_object_declaration ::= interface_constant_declaration
///     | interface_signal_declaration | interface_variable_declaration
///     | interface_file_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceObjectDeclaration {
    Constant(InterfaceConstantDeclaration),
    Signal(InterfaceSignalDeclaration),
    Variable(InterfaceVariableDeclaration),
    File(InterfaceFileDeclaration),
}

/// EBNF: `interface_constant_declaration ::= [ CONSTANT ] identifier_list : [ IN ]
///     subtype_indication [ := static_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceConstantDeclaration {
    pub has_constant_keyword: bool,
    pub identifiers: IdentifierList,
    pub has_in_keyword: bool,
    pub subtype_indication: SubtypeIndication,
    pub default_expression: Option<Expression>,
}

/// EBNF: `interface_signal_declaration ::= [SIGNAL] identifier_list : [ mode ]
///     subtype_indication [ BUS ] [ := static_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceSignalDeclaration {
    pub has_signal_keyword: bool,
    pub identifiers: IdentifierList,
    pub mode: Option<Mode>,
    pub subtype_indication: SubtypeIndication,
    pub has_bus: bool,
    pub default_expression: Option<Expression>,
}

/// EBNF: `interface_variable_declaration ::= [VARIABLE] identifier_list : [ mode ]
///     subtype_indication [ := static_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceVariableDeclaration {
    pub has_variable_keyword: bool,
    pub identifiers: IdentifierList,
    pub mode: Option<Mode>,
    pub subtype_indication: SubtypeIndication,
    pub default_expression: Option<Expression>,
}

/// EBNF: `interface_file_declaration ::= FILE identifier_list : subtype_indication`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceFileDeclaration {
    pub identifiers: IdentifierList,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `interface_type_declaration ::= interface_incomplete_type_declaration` (VHDL-2008)
pub type InterfaceTypeDeclaration = InterfaceIncompleteTypeDeclaration;

/// EBNF: `interface_incomplete_type_declaration ::= TYPE identifier` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceIncompleteTypeDeclaration {
    pub identifier: Identifier,
}

/// EBNF: `interface_subprogram_declaration ::= interface_subprogram_specification
///     [ IS interface_subprogram_default ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceSubprogramDeclaration {
    pub specification: InterfaceSubprogramSpecification,
    pub default: Option<InterfaceSubprogramDefault>,
}

/// EBNF: `interface_subprogram_specification ::= interface_procedure_specification
///     | interface_function_specification` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceSubprogramSpecification {
    Procedure(InterfaceProcedureSpecification),
    Function(InterfaceFunctionSpecification),
}

/// EBNF: `interface_procedure_specification ::= PROCEDURE designator
///     [ [ PARAMETER ] ( formal_parameter_list ) ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceProcedureSpecification {
    pub designator: Designator,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
}

/// EBNF: `interface_function_specification ::= [ PURE | IMPURE ] FUNCTION designator
///     [ [ PARAMETER ] ( formal_parameter_list ) ] RETURN type_mark` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceFunctionSpecification {
    pub purity: Option<Purity>,
    pub designator: Designator,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
    pub return_type: TypeMark,
}

/// Function purity (VHDL-93+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purity {
    Pure,
    Impure,
}

/// EBNF: `interface_subprogram_default ::= subprogram_name | <>` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceSubprogramDefault {
    Name(Box<Name>),
    Box,
}

/// EBNF: `interface_package_declaration ::= PACKAGE identifier IS NEW
///     uninstantiated_package_name interface_package_generic_map_aspect` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfacePackageDeclaration {
    pub identifier: Identifier,
    pub package_name: Box<Name>,
    pub generic_map_aspect: InterfacePackageGenericMapAspect,
}

/// EBNF: `interface_package_generic_map_aspect ::= generic_map_aspect
///     | GENERIC MAP ( <> ) | GENERIC MAP ( DEFAULT )` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfacePackageGenericMapAspect {
    GenericMapAspect(GenericMapAspect),
    Box,
    Default,
}

/// EBNF: `generic_clause ::= GENERIC ( generic_list ) ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericClause {
    pub generic_list: GenericList,
}

/// EBNF: `generic_list ::= generic_interface_list`
pub type GenericList = InterfaceList;

/// EBNF: `generic_map_aspect ::= GENERIC MAP ( generic_association_list )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericMapAspect {
    pub association_list: super::association::AssociationList,
}

/// EBNF: `port_clause ::= PORT ( port_list ) ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortClause {
    pub port_list: PortList,
}

/// EBNF: `port_list ::= port_interface_list`
pub type PortList = InterfaceList;

/// EBNF: `port_map_aspect ::= PORT MAP ( port_association_list )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMapAspect {
    pub association_list: super::association::AssociationList,
}
