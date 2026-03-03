//! Name-related AST nodes.

use super::common::*;
use super::expression::Expression;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{DiscreteRange, SubtypeIndication};
use crate::parser::{Parser, ParseError};

/// A VHDL name.
///
/// EBNF (VHDL-2008): `name ::= simple_name | operator_symbol | character_literal
///     | selected_name | indexed_name | slice_name | attribute_name | external_name`
///
/// VHDL-87/93 omit `external_name` and (87 omits) `character_literal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Simple(SimpleName),
    OperatorSymbol(OperatorSymbol),
    CharacterLiteral(String),
    Selected(Box<SelectedName>),
    Indexed(Box<IndexedName>),
    Slice(Box<SliceName>),
    Attribute(Box<AttributeName>),
    /// VHDL-2008 external name.
    External(Box<ExternalName>),
}

/// A prefix for selected / indexed / slice / attribute names.
///
/// EBNF: `prefix ::= name | function_call`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix {
    Name(Box<Name>),
    FunctionCall(Box<FunctionCall>),
}

/// A suffix used in selected names.
///
/// EBNF: `suffix ::= simple_name | character_literal | operator_symbol | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suffix {
    SimpleName(SimpleName),
    CharacterLiteral(String),
    OperatorSymbol(OperatorSymbol),
    All,
}

/// A selected name (dot notation).
///
/// EBNF: `selected_name ::= prefix . suffix`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedName {
    pub prefix: Prefix,
    pub suffix: Suffix,
}

/// An indexed name.
///
/// EBNF: `indexed_name ::= prefix ( expression { , expression } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedName {
    pub prefix: Prefix,
    pub expressions: Vec<Expression>,
}

/// A slice name.
///
/// EBNF: `slice_name ::= prefix ( discrete_range )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceName {
    pub prefix: Prefix,
    pub discrete_range: DiscreteRange,
}

/// An attribute designator.
///
/// EBNF: `attribute_designator ::= attribute_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDesignator {
    pub name: SimpleName,
}

/// An attribute name.
///
/// EBNF (VHDL-2008): `attribute_name ::= prefix [ signature ] ' attribute_designator [ ( expression ) ]`
///
/// VHDL-87: omits optional signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeName {
    pub prefix: Prefix,
    /// VHDL-93+ only.
    pub signature: Option<Signature>,
    pub designator: AttributeDesignator,
    pub expression: Option<Box<Expression>>,
}

/// An external name (VHDL-2008).
///
/// EBNF: `external_name ::= external_constant_name | external_signal_name | external_variable_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalName {
    Constant(ExternalConstantName),
    Signal(ExternalSignalName),
    Variable(ExternalVariableName),
}

/// EBNF: `external_constant_name ::= << CONSTANT external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalConstantName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_signal_name ::= << SIGNAL external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSignalName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_variable_name ::= << VARIABLE external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalVariableName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_pathname ::= package_pathname | absolute_pathname | relative_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalPathname {
    Package(PackagePathname),
    Absolute(AbsolutePathname),
    Relative(RelativePathname),
}

/// EBNF: `package_pathname ::= @ library_logical_name . { package_simple_name . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePathname {
    pub library_name: Identifier,
    pub package_names: Vec<SimpleName>,
    pub object_name: SimpleName,
}

/// EBNF: `absolute_pathname ::= . partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsolutePathname {
    pub partial: PartialPathname,
}

/// EBNF: `relative_pathname ::= { ^ . } partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePathname {
    pub up_count: usize,
    pub partial: PartialPathname,
}

/// EBNF: `partial_pathname ::= { pathname_element . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPathname {
    pub elements: Vec<PathnameElement>,
    pub object_name: SimpleName,
}

/// EBNF: `pathname_element ::= entity_simple_name | component_instantiation_label
///     | block_label | generate_statement_label [ ( static_expression ) ] | package_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathnameElement {
    pub name: Identifier,
    /// Present for generate_statement_label with an index expression.
    pub expression: Option<Box<Expression>>,
}

/// A function call.
///
/// EBNF: `function_call ::= function_name [ ( actual_parameter_part ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub function_name: Box<Name>,
    pub parameters: Option<super::association::ActualParameterPart>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for Name {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Name::Simple(n) => n.format(f, indent_level),
            Name::OperatorSymbol(n) => n.format(f, indent_level),
            Name::CharacterLiteral(c) => write!(f, "'{}'", c),
            Name::Selected(n) => n.format(f, indent_level),
            Name::Indexed(n) => n.format(f, indent_level),
            Name::Slice(n) => n.format(f, indent_level),
            Name::Attribute(n) => n.format(f, indent_level),
            Name::External(n) => n.format(f, indent_level),
        }
    }
}

impl AstNode for Prefix {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Prefix::Name(n) => n.format(f, indent_level),
            Prefix::FunctionCall(fc) => fc.format(f, indent_level),
        }
    }
}

impl AstNode for Suffix {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Suffix::SimpleName(n) => n.format(f, indent_level),
            Suffix::CharacterLiteral(c) => write!(f, "'{}'", c),
            Suffix::OperatorSymbol(n) => n.format(f, indent_level),
            Suffix::All => write!(f, "all"),
        }
    }
}

impl AstNode for SelectedName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, ".")?;
        self.suffix.format(f, indent_level)
    }
}

impl AstNode for IndexedName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, "(")?;
        format_comma_separated(&self.expressions, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for SliceName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, "(")?;
        self.discrete_range.format(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for AttributeDesignator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name.format(f, indent_level)
    }
}

impl AstNode for AttributeName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        if let Some(ref sig) = self.signature {
            sig.format(f, indent_level)?;
        }
        write!(f, "'")?;
        self.designator.format(f, indent_level)?;
        if let Some(ref expr) = self.expression {
            write!(f, "(")?;
            expr.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for ExternalName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ExternalName::Constant(n) => n.format(f, indent_level),
            ExternalName::Signal(n) => n.format(f, indent_level),
            ExternalName::Variable(n) => n.format(f, indent_level),
        }
    }
}

impl AstNode for ExternalConstantName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< constant ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalSignalName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< signal ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalVariableName {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< variable ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalPathname {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ExternalPathname::Package(p) => p.format(f, indent_level),
            ExternalPathname::Absolute(p) => p.format(f, indent_level),
            ExternalPathname::Relative(p) => p.format(f, indent_level),
        }
    }
}

impl AstNode for PackagePathname {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "@")?;
        self.library_name.format(f, indent_level)?;
        for pkg in &self.package_names {
            write!(f, ".")?;
            pkg.format(f, indent_level)?;
        }
        write!(f, ".")?;
        self.object_name.format(f, indent_level)
    }
}

impl AstNode for AbsolutePathname {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, ".")?;
        self.partial.format(f, indent_level)
    }
}

impl AstNode for RelativePathname {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for i in 0..self.up_count {
            write!(f, "^")?;
            if i < self.up_count - 1 {
                write!(f, ".")?;
            }
        }
        if self.up_count > 0 {
            write!(f, ".")?;
        }
        self.partial.format(f, indent_level)
    }
}

impl AstNode for PartialPathname {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for elem in &self.elements {
            elem.format(f, indent_level)?;
            write!(f, ".")?;
        }
        self.object_name.format(f, indent_level)
    }
}

impl AstNode for PathnameElement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name.format(f, indent_level)?;
        if let Some(ref expr) = self.expression {
            write!(f, "(")?;
            expr.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for FunctionCall {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.function_name.format(f, indent_level)?;
        if let Some(ref params) = self.parameters {
            write!(f, "(")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}
