//! Interface declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, write_indent, format_comma_separated, format_semicolon_lines};
use super::type_def::{SubtypeIndication, TypeMark};
use crate::parser::{Parser, ParseError};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for InterfaceDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Object(inner) => inner.format(f, indent_level),
            Self::Type(inner) => inner.format(f, indent_level),
            Self::Subprogram(inner) => inner.format(f, indent_level),
            Self::Package(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for InterfaceList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_semicolon_lines(&self.elements, f, indent_level)
    }
}

impl AstNode for InterfaceObjectDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Constant(inner) => inner.format(f, indent_level),
            Self::Signal(inner) => inner.format(f, indent_level),
            Self::Variable(inner) => inner.format(f, indent_level),
            Self::File(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for InterfaceConstantDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if self.has_constant_keyword {
            write!(f, "constant ")?;
        }
        format_comma_separated(&self.identifiers.identifiers, f, indent_level)?;
        write!(f, " : ")?;
        if self.has_in_keyword {
            write!(f, "in ")?;
        }
        self.subtype_indication.format(f, indent_level)?;
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for InterfaceSignalDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if self.has_signal_keyword {
            write!(f, "signal ")?;
        }
        format_comma_separated(&self.identifiers.identifiers, f, indent_level)?;
        write!(f, " : ")?;
        if let Some(mode) = &self.mode {
            mode.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.subtype_indication.format(f, indent_level)?;
        if self.has_bus {
            write!(f, " bus")?;
        }
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for InterfaceVariableDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if self.has_variable_keyword {
            write!(f, "variable ")?;
        }
        format_comma_separated(&self.identifiers.identifiers, f, indent_level)?;
        write!(f, " : ")?;
        if let Some(mode) = &self.mode {
            mode.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.subtype_indication.format(f, indent_level)?;
        if let Some(expr) = &self.default_expression {
            write!(f, " := ")?;
            expr.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for InterfaceFileDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "file ")?;
        format_comma_separated(&self.identifiers.identifiers, f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)
    }
}

impl AstNode for InterfaceIncompleteTypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "type ")?;
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for InterfaceSubprogramDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.specification.format(f, indent_level)?;
        if let Some(default) = &self.default {
            write!(f, " is ")?;
            default.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for InterfaceSubprogramSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Procedure(inner) => inner.format(f, indent_level),
            Self::Function(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for InterfaceProcedureSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "procedure ")?;
        self.designator.format(f, indent_level)?;
        if let Some(params) = &self.parameter_list {
            if self.has_parameter_keyword {
                write!(f, " parameter")?;
            }
            write!(f, " (")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for InterfaceFunctionSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(purity) = &self.purity {
            purity.format(f, indent_level)?;
            write!(f, " ")?;
        }
        write!(f, "function ")?;
        self.designator.format(f, indent_level)?;
        if let Some(params) = &self.parameter_list {
            if self.has_parameter_keyword {
                write!(f, " parameter")?;
            }
            write!(f, " (")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        write!(f, " return ")?;
        self.return_type.format(f, indent_level)
    }
}

impl AstNode for Purity {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Pure => write!(f, "pure"),
            Self::Impure => write!(f, "impure"),
        }
    }
}

impl AstNode for InterfaceSubprogramDefault {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Name(name) => name.format(f, indent_level),
            Self::Box => write!(f, "<>"),
        }
    }
}

impl AstNode for InterfacePackageDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is new ")?;
        self.package_name.format(f, indent_level)?;
        write!(f, " ")?;
        self.generic_map_aspect.format(f, indent_level)
    }
}

impl AstNode for InterfacePackageGenericMapAspect {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::GenericMapAspect(inner) => inner.format(f, indent_level),
            Self::Box => write!(f, "generic map (<>)"),
            Self::Default => write!(f, "generic map (default)"),
        }
    }
}

impl AstNode for GenericClause {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "generic (")?;
        self.generic_list.format(f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        writeln!(f, ");")
    }
}

impl AstNode for GenericMapAspect {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "generic map (")?;
        format_comma_separated(&self.association_list.elements, f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for PortClause {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "port (")?;
        self.port_list.format(f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        writeln!(f, ");")
    }
}

impl AstNode for PortMapAspect {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "port map (")?;
        format_comma_separated(&self.association_list.elements, f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        write!(f, ")")
    }
}
