//! Subprogram declaration and body AST nodes.

use super::common::*;
use super::interface::GenericMapAspect;
use super::name::Name;
use super::node::{AstNode, write_indent, format_lines};
use super::type_def::TypeMark;
use crate::parser::{Parser, ParseError};

/// EBNF: `subprogram_declaration ::= subprogram_specification ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramDeclaration {
    pub specification: SubprogramSpecification,
}

/// EBNF (VHDL-2008): `subprogram_body ::= subprogram_specification IS
///     subprogram_declarative_part BEGIN subprogram_statement_part
///     END [ subprogram_kind ] [ designator ] ;`
/// EBNF (VHDL-87): `...END [ designator ] ;` (no optional subprogram_kind).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramBody {
    pub specification: SubprogramSpecification,
    pub declarative_part: SubprogramDeclarativePart,
    pub statement_part: SubprogramStatementPart,
    pub end_kind: Option<SubprogramKind>,
    pub end_designator: Option<Designator>,
}

/// EBNF (VHDL-2008): `subprogram_specification ::= procedure_specification
///     | function_specification`
/// EBNF (VHDL-87/93): Combined inline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubprogramSpecification {
    Procedure(ProcedureSpecification),
    Function(FunctionSpecification),
}

/// EBNF (VHDL-2008): `procedure_specification ::= PROCEDURE designator subprogram_header
///     [ [ PARAMETER ] ( formal_parameter_list ) ]`
/// EBNF (VHDL-87/93): `PROCEDURE designator [ ( formal_parameter_list ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureSpecification {
    pub designator: Designator,
    /// VHDL-2008.
    pub header: Option<SubprogramHeader>,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
}

/// EBNF (VHDL-2008): `function_specification ::= [ PURE | IMPURE ] FUNCTION designator
///     subprogram_header [ [ PARAMETER ] ( formal_parameter_list ) ] RETURN type_mark`
/// EBNF (VHDL-87): `FUNCTION designator [ ( formal_parameter_list ) ] RETURN type_mark`
/// EBNF (VHDL-93): `[ PURE | IMPURE ] FUNCTION designator
///     [ ( formal_parameter_list ) ] RETURN type_mark`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpecification {
    /// VHDL-93+.
    pub purity: Option<super::interface::Purity>,
    pub designator: Designator,
    /// VHDL-2008.
    pub header: Option<SubprogramHeader>,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
    pub return_type: TypeMark,
}

/// EBNF: `subprogram_header ::= [ GENERIC ( generic_list ) [ generic_map_aspect ] ]`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramHeader {
    pub generic_list: super::interface::GenericList,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

/// EBNF: `subprogram_kind ::= PROCEDURE | FUNCTION`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubprogramKind {
    Procedure,
    Function,
}

/// EBNF: `subprogram_declarative_part ::= { subprogram_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramDeclarativePart {
    pub items: Vec<SubprogramDeclarativeItem>,
}

/// EBNF (VHDL-2008): `subprogram_declarative_item ::= subprogram_declaration
///     | subprogram_body | subprogram_instantiation_declaration | package_declaration
///     | package_body | package_instantiation_declaration | type_declaration
///     | subtype_declaration | constant_declaration | variable_declaration | file_declaration
///     | alias_declaration | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubprogramDeclarativeItem {
    SubprogramDeclaration(Box<SubprogramDeclaration>),
    SubprogramBody(Box<SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<super::package::PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<super::package::PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `subprogram_statement_part ::= { sequential_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramStatementPart {
    pub statements: Vec<super::sequential::SequentialStatement>,
}

/// EBNF: `subprogram_instantiation_declaration ::= subprogram_kind identifier IS NEW
///     uninstantiated_subprogram_name [ signature ] [ generic_map_aspect ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramInstantiationDeclaration {
    pub kind: SubprogramKind,
    pub identifier: Identifier,
    pub subprogram_name: Box<Name>,
    pub signature: Option<Signature>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for SubprogramDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.specification.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for SubprogramBody {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.specification.format(f, indent_level)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "begin")?;
        self.statement_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end")?;
        if let Some(kind) = &self.end_kind {
            write!(f, " ")?;
            kind.format(f, indent_level)?;
        }
        if let Some(desig) = &self.end_designator {
            write!(f, " ")?;
            desig.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SubprogramSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            SubprogramSpecification::Procedure(spec) => spec.format(f, indent_level),
            SubprogramSpecification::Function(spec) => spec.format(f, indent_level),
        }
    }
}

impl AstNode for ProcedureSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "procedure ")?;
        self.designator.format(f, indent_level)?;
        if let Some(header) = &self.header {
            write!(f, " ")?;
            header.format(f, indent_level)?;
        }
        if self.has_parameter_keyword {
            write!(f, " parameter")?;
        }
        if let Some(params) = &self.parameter_list {
            write!(f, " (")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for FunctionSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(purity) = &self.purity {
            purity.format(f, indent_level)?;
            write!(f, " ")?;
        }
        write!(f, "function ")?;
        self.designator.format(f, indent_level)?;
        if let Some(header) = &self.header {
            write!(f, " ")?;
            header.format(f, indent_level)?;
        }
        if self.has_parameter_keyword {
            write!(f, " parameter")?;
        }
        if let Some(params) = &self.parameter_list {
            write!(f, " (")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        write!(f, " return ")?;
        self.return_type.format(f, indent_level)
    }
}

impl AstNode for SubprogramHeader {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "generic (")?;
        self.generic_list.format(f, indent_level)?;
        write!(f, ")")?;
        if let Some(generic_map) = &self.generic_map_aspect {
            write!(f, " ")?;
            generic_map.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for SubprogramKind {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            SubprogramKind::Procedure => write!(f, "procedure"),
            SubprogramKind::Function => write!(f, "function"),
        }
    }
}

impl AstNode for SubprogramDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for SubprogramDeclarativeItem {
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

impl AstNode for SubprogramStatementPart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for SubprogramInstantiationDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.kind.format(f, indent_level)?;
        write!(f, " ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is new ")?;
        self.subprogram_name.format(f, indent_level)?;
        if let Some(sig) = &self.signature {
            write!(f, " ")?;
            sig.format(f, indent_level)?;
        }
        if let Some(generic_map) = &self.generic_map_aspect {
            write!(f, " ")?;
            generic_map.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}
