//! Interface declaration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{
    AstNode, format_comma_lines, format_comma_separated, format_semicolon_lines, write_indent,
};
use super::type_def::{SubtypeIndication, TypeMark};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Discriminate by leading keyword
        if parser.at_keyword(KeywordKind::Constant) {
            let decl = InterfaceConstantDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration::Constant(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Signal) {
            let decl = InterfaceSignalDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration::Signal(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Variable) {
            let decl = InterfaceVariableDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration::Variable(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::File) {
            let decl = InterfaceFileDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration::File(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Type) {
            let decl = InterfaceIncompleteTypeDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Type(decl));
        }

        if parser.at_keyword(KeywordKind::Procedure)
            || parser.at_keyword(KeywordKind::Function)
            || parser.at_keyword(KeywordKind::Pure)
            || parser.at_keyword(KeywordKind::Impure)
        {
            let decl = InterfaceSubprogramDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Subprogram(decl));
        }

        if parser.at_keyword(KeywordKind::Package) {
            let decl = InterfacePackageDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Package(decl));
        }

        // No keyword prefix -- default to signal declaration (common in port lists)
        // The identifier_list : [mode] subtype_indication pattern
        if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
            let decl = InterfaceSignalDeclaration::parse(parser)?;
            return Ok(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration::Signal(decl),
            ));
        }

        Err(parser.error("expected interface declaration"))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut elements = vec![InterfaceElement::parse(parser)?];
        while parser.consume_if(TokenKind::Semicolon).is_some() {
            // Check if there's actually another element or if we hit closing paren / end
            if parser.at(TokenKind::RightParen) || parser.eof() {
                break;
            }
            elements.push(InterfaceElement::parse(parser)?);
        }
        Ok(InterfaceList { elements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_semicolon_lines(&self.elements, f, indent_level)
    }
}

impl AstNode for InterfaceObjectDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Constant) {
            Ok(InterfaceObjectDeclaration::Constant(
                InterfaceConstantDeclaration::parse(parser)?,
            ))
        } else if parser.at_keyword(KeywordKind::Signal) {
            Ok(InterfaceObjectDeclaration::Signal(
                InterfaceSignalDeclaration::parse(parser)?,
            ))
        } else if parser.at_keyword(KeywordKind::Variable) {
            Ok(InterfaceObjectDeclaration::Variable(
                InterfaceVariableDeclaration::parse(parser)?,
            ))
        } else if parser.at_keyword(KeywordKind::File) {
            Ok(InterfaceObjectDeclaration::File(
                InterfaceFileDeclaration::parse(parser)?,
            ))
        } else {
            // Default to signal (for port list context)
            Ok(InterfaceObjectDeclaration::Signal(
                InterfaceSignalDeclaration::parse(parser)?,
            ))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let has_constant_keyword = parser.consume_if_keyword(KeywordKind::Constant).is_some();
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let has_in_keyword = parser.consume_if_keyword(KeywordKind::In).is_some();
        let subtype_indication = SubtypeIndication::parse(parser)?;
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(InterfaceConstantDeclaration {
            has_constant_keyword,
            identifiers,
            has_in_keyword,
            subtype_indication,
            default_expression,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let has_signal_keyword = parser.consume_if_keyword(KeywordKind::Signal).is_some();
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;

        // Optional mode
        let mode = if parser.at_keyword(KeywordKind::In)
            || parser.at_keyword(KeywordKind::Out)
            || parser.at_keyword(KeywordKind::Inout)
            || parser.at_keyword(KeywordKind::Buffer)
            || parser.at_keyword(KeywordKind::Linkage)
        {
            Some(Mode::parse(parser)?)
        } else {
            None
        };

        let subtype_indication = SubtypeIndication::parse(parser)?;
        let has_bus = parser.consume_if_keyword(KeywordKind::Bus).is_some();
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(InterfaceSignalDeclaration {
            has_signal_keyword,
            identifiers,
            mode,
            subtype_indication,
            has_bus,
            default_expression,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let has_variable_keyword = parser.consume_if_keyword(KeywordKind::Variable).is_some();
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;

        // Optional mode
        let mode = if parser.at_keyword(KeywordKind::In)
            || parser.at_keyword(KeywordKind::Out)
            || parser.at_keyword(KeywordKind::Inout)
            || parser.at_keyword(KeywordKind::Buffer)
            || parser.at_keyword(KeywordKind::Linkage)
        {
            Some(Mode::parse(parser)?)
        } else {
            None
        };

        let subtype_indication = SubtypeIndication::parse(parser)?;
        let default_expression = if parser.consume_if(TokenKind::VarAssign).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(InterfaceVariableDeclaration {
            has_variable_keyword,
            identifiers,
            mode,
            subtype_indication,
            default_expression,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::File)?;
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        Ok(InterfaceFileDeclaration {
            identifiers,
            subtype_indication,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Type)?;
        let identifier = Identifier::parse(parser)?;
        Ok(InterfaceIncompleteTypeDeclaration { identifier })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "type ")?;
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for InterfaceSubprogramDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let specification = InterfaceSubprogramSpecification::parse(parser)?;
        let default = if parser.consume_if_keyword(KeywordKind::Is).is_some() {
            Some(InterfaceSubprogramDefault::parse(parser)?)
        } else {
            None
        };
        Ok(InterfaceSubprogramDeclaration {
            specification,
            default,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Procedure) {
            let spec = InterfaceProcedureSpecification::parse(parser)?;
            Ok(InterfaceSubprogramSpecification::Procedure(spec))
        } else if parser.at_keyword(KeywordKind::Function)
            || parser.at_keyword(KeywordKind::Pure)
            || parser.at_keyword(KeywordKind::Impure)
        {
            let spec = InterfaceFunctionSpecification::parse(parser)?;
            Ok(InterfaceSubprogramSpecification::Function(spec))
        } else {
            Err(parser.error("expected procedure or function specification"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Procedure(inner) => inner.format(f, indent_level),
            Self::Function(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for InterfaceProcedureSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Procedure)?;
        let designator = Designator::parse(parser)?;

        let mut has_parameter_keyword = false;
        let parameter_list =
            if parser.at_keyword(KeywordKind::Parameter) || parser.at(TokenKind::LeftParen) {
                if parser.consume_if_keyword(KeywordKind::Parameter).is_some() {
                    has_parameter_keyword = true;
                }
                parser.expect(TokenKind::LeftParen)?;
                let list = InterfaceList::parse(parser)?;
                parser.expect(TokenKind::RightParen)?;
                Some(list)
            } else {
                None
            };

        Ok(InterfaceProcedureSpecification {
            designator,
            has_parameter_keyword,
            parameter_list,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "procedure ")?;
        self.designator.format(f, indent_level)?;
        if let Some(params) = &self.parameter_list {
            if self.has_parameter_keyword {
                write!(f, " parameter")?;
            }
            if params.elements.len() <= 1 {
                write!(f, " (")?;
                params.format(f, 0)?;
                write!(f, ")")?;
            } else {
                writeln!(f, " (")?;
                params.format(f, indent_level + 1)?;
                writeln!(f)?;
                write_indent(f, indent_level)?;
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl AstNode for InterfaceFunctionSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Optional PURE | IMPURE
        let purity =
            if parser.at_keyword(KeywordKind::Pure) || parser.at_keyword(KeywordKind::Impure) {
                Some(Purity::parse(parser)?)
            } else {
                None
            };

        parser.expect_keyword(KeywordKind::Function)?;
        let designator = Designator::parse(parser)?;

        let mut has_parameter_keyword = false;
        let parameter_list =
            if parser.at_keyword(KeywordKind::Parameter) || parser.at(TokenKind::LeftParen) {
                if parser.consume_if_keyword(KeywordKind::Parameter).is_some() {
                    has_parameter_keyword = true;
                }
                parser.expect(TokenKind::LeftParen)?;
                let list = InterfaceList::parse(parser)?;
                parser.expect(TokenKind::RightParen)?;
                Some(list)
            } else {
                None
            };

        parser.expect_keyword(KeywordKind::Return)?;
        let return_type = TypeMark::parse(parser)?;

        Ok(InterfaceFunctionSpecification {
            purity,
            designator,
            has_parameter_keyword,
            parameter_list,
            return_type,
        })
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
            if params.elements.len() <= 1 {
                write!(f, " (")?;
                params.format(f, 0)?;
                write!(f, ")")?;
            } else {
                writeln!(f, " (")?;
                params.format(f, indent_level + 1)?;
                writeln!(f)?;
                write_indent(f, indent_level)?;
                write!(f, ")")?;
            }
        }
        write!(f, " return ")?;
        self.return_type.format(f, indent_level)
    }
}

impl AstNode for Purity {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Pure).is_some() {
            Ok(Purity::Pure)
        } else if parser.consume_if_keyword(KeywordKind::Impure).is_some() {
            Ok(Purity::Impure)
        } else {
            Err(parser.error("expected 'pure' or 'impure'"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Pure => write!(f, "pure"),
            Self::Impure => write!(f, "impure"),
        }
    }
}

impl AstNode for InterfaceSubprogramDefault {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at(TokenKind::Box) {
            parser.consume();
            Ok(InterfaceSubprogramDefault::Box)
        } else {
            let name = Name::parse(parser)?;
            Ok(InterfaceSubprogramDefault::Name(Box::new(name)))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Name(name) => name.format(f, indent_level),
            Self::Box => write!(f, "<>"),
        }
    }
}

impl AstNode for InterfacePackageDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Package)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        parser.expect_keyword(KeywordKind::New)?;
        let package_name = Box::new(Name::parse(parser)?);
        let generic_map_aspect = InterfacePackageGenericMapAspect::parse(parser)?;
        Ok(InterfacePackageDeclaration {
            identifier,
            package_name,
            generic_map_aspect,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // GENERIC MAP ( <> ) | GENERIC MAP ( DEFAULT ) | generic_map_aspect
        parser.expect_keyword(KeywordKind::Generic)?;
        parser.expect_keyword(KeywordKind::Map)?;
        parser.expect(TokenKind::LeftParen)?;

        if parser.at(TokenKind::Box) {
            parser.consume();
            parser.expect(TokenKind::RightParen)?;
            return Ok(InterfacePackageGenericMapAspect::Box);
        }

        if parser.at_keyword(KeywordKind::Default) {
            parser.consume();
            parser.expect(TokenKind::RightParen)?;
            return Ok(InterfacePackageGenericMapAspect::Default);
        }

        // Regular association list
        let association_list = super::association::AssociationList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Ok(InterfacePackageGenericMapAspect::GenericMapAspect(
            GenericMapAspect { association_list },
        ))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Generic)?;
        parser.expect(TokenKind::LeftParen)?;
        let generic_list = InterfaceList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(GenericClause { generic_list })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Generic)?;
        parser.expect_keyword(KeywordKind::Map)?;
        parser.expect(TokenKind::LeftParen)?;
        let association_list = super::association::AssociationList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Ok(GenericMapAspect { association_list })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "generic map (")?;
        format_comma_lines(&self.association_list.elements, f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for PortClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Port)?;
        parser.expect(TokenKind::LeftParen)?;
        let port_list = InterfaceList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(PortClause { port_list })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Port)?;
        parser.expect_keyword(KeywordKind::Map)?;
        parser.expect(TokenKind::LeftParen)?;
        let association_list = super::association::AssociationList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Ok(PortMapAspect { association_list })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        writeln!(f, "port map (")?;
        format_comma_lines(&self.association_list.elements, f, indent_level + 1)?;
        writeln!(f)?;
        write_indent(f, indent_level)?;
        write!(f, ")")
    }
}
