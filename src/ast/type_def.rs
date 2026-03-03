//! Type definition AST nodes.

use super::common::*;
use super::expression::SimpleExpression;
use super::literal::{EnumerationLiteral, PhysicalLiteral};
use super::name::Name;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use crate::parser::{ParseError, Parser};

/// EBNF: `type_declaration ::= full_type_declaration | incomplete_type_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDeclaration {
    Full(FullTypeDeclaration),
    Incomplete(IncompleteTypeDeclaration),
}

/// EBNF: `full_type_declaration ::= TYPE identifier IS type_definition ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTypeDeclaration {
    pub identifier: Identifier,
    pub type_definition: TypeDefinition,
}

/// EBNF: `incomplete_type_declaration ::= TYPE identifier ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteTypeDeclaration {
    pub identifier: Identifier,
}

/// EBNF (VHDL-2008): `type_definition ::= scalar_type_definition | composite_type_definition
///     | access_type_definition | file_type_definition | protected_type_definition`
/// EBNF (VHDL-87/93): omits `protected_type_definition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefinition {
    Scalar(ScalarTypeDefinition),
    Composite(CompositeTypeDefinition),
    Access(AccessTypeDefinition),
    File(FileTypeDefinition),
    /// VHDL-2008.
    Protected(ProtectedTypeDefinition),
}

/// EBNF: `type_mark ::= type_name | subtype_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMark {
    TypeName(Box<Name>),
    SubtypeName(Box<Name>),
}

/// EBNF: `scalar_type_definition ::= enumeration_type_definition | integer_type_definition
///     | floating_type_definition | physical_type_definition`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarTypeDefinition {
    Enumeration(EnumerationTypeDefinition),
    Integer(IntegerTypeDefinition),
    Floating(FloatingTypeDefinition),
    Physical(PhysicalTypeDefinition),
}

/// EBNF: `enumeration_type_definition ::= ( enumeration_literal { , enumeration_literal } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumerationTypeDefinition {
    pub literals: Vec<EnumerationLiteral>,
}

/// EBNF: `integer_type_definition ::= range_constraint`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerTypeDefinition {
    pub range_constraint: RangeConstraint,
}

/// EBNF: `floating_type_definition ::= range_constraint`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatingTypeDefinition {
    pub range_constraint: RangeConstraint,
}

/// EBNF (VHDL-2008): `physical_type_definition ::= range_constraint UNITS
///     primary_unit_declaration { secondary_unit_declaration } END UNITS [ physical_type_simple_name ]`
/// EBNF (VHDL-87): omits `[ physical_type_simple_name ]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalTypeDefinition {
    pub range_constraint: RangeConstraint,
    pub primary_unit: PrimaryUnitDeclaration,
    pub secondary_units: Vec<SecondaryUnitDeclaration>,
    /// VHDL-93+.
    pub end_name: Option<SimpleName>,
}

/// EBNF: `primary_unit_declaration ::= identifier ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryUnitDeclaration {
    pub identifier: Identifier,
}

/// EBNF: `secondary_unit_declaration ::= identifier = physical_literal ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryUnitDeclaration {
    pub identifier: Identifier,
    pub literal: PhysicalLiteral,
}

/// EBNF: `composite_type_definition ::= array_type_definition | record_type_definition`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeTypeDefinition {
    Array(ArrayTypeDefinition),
    Record(RecordTypeDefinition),
}

/// EBNF: `array_type_definition ::= unbounded_array_definition | constrained_array_definition`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayTypeDefinition {
    Unbounded(UnboundedArrayDefinition),
    Constrained(ConstrainedArrayDefinition),
}

/// EBNF: `unbounded_array_definition ::= ARRAY ( index_subtype_definition
///     { , index_subtype_definition } ) OF element_subtype_indication`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnboundedArrayDefinition {
    pub index_subtypes: Vec<IndexSubtypeDefinition>,
    pub element_subtype: SubtypeIndication,
}

/// EBNF: `constrained_array_definition ::= ARRAY index_constraint OF element_subtype_indication`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstrainedArrayDefinition {
    pub index_constraint: IndexConstraint,
    pub element_subtype: SubtypeIndication,
}

/// EBNF: `record_type_definition ::= RECORD element_declaration { element_declaration }
///     END RECORD [ record_type_simple_name ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTypeDefinition {
    pub elements: Vec<ElementDeclaration>,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `element_declaration ::= identifier_list : element_subtype_definition ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementDeclaration {
    pub identifiers: IdentifierList,
    pub subtype: ElementSubtypeDefinition,
}

/// EBNF: `element_subtype_definition ::= subtype_indication`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementSubtypeDefinition {
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `access_type_definition ::= ACCESS subtype_indication`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessTypeDefinition {
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `file_type_definition ::= FILE OF type_mark`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTypeDefinition {
    pub type_mark: TypeMark,
}

/// EBNF: `protected_type_definition ::= protected_type_declaration | protected_type_body`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectedTypeDefinition {
    Declaration(ProtectedTypeDeclaration),
    Body(ProtectedTypeBody),
}

/// EBNF: `protected_type_declaration ::= PROTECTED protected_type_declarative_part
///     END PROTECTED [ protected_type_simple_name ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedTypeDeclaration {
    pub declarative_part: ProtectedTypeDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `protected_type_declarative_part ::= { protected_type_declarative_item }` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedTypeDeclarativePart {
    pub items: Vec<ProtectedTypeDeclarativeItem>,
}

/// EBNF: `protected_type_declarative_item ::= subprogram_declaration
///     | subprogram_instantiation_declaration | attribute_specification | use_clause`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectedTypeDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
}

/// EBNF: `protected_type_body ::= PROTECTED BODY protected_type_body_declarative_part
///     END PROTECTED BODY [ protected_type_simple_name ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedTypeBody {
    pub declarative_part: ProtectedTypeBodyDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `protected_type_body_declarative_part ::= { protected_type_body_declarative_item }`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedTypeBodyDeclarativePart {
    pub items: Vec<ProtectedTypeBodyDeclarativeItem>,
}

/// EBNF: `protected_type_body_declarative_item ::= subprogram_declaration | subprogram_body
///     | subprogram_instantiation_declaration | package_declaration | package_body
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | variable_declaration | file_declaration | alias_declaration
///     | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectedTypeBodyDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramBody(Box<super::subprogram::SubprogramBody>),
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    PackageDeclaration(Box<super::package::PackageDeclaration>),
    PackageBody(Box<super::package::PackageBody>),
    PackageInstantiationDeclaration(Box<super::package::PackageInstantiationDeclaration>),
    TypeDeclaration(Box<TypeDeclaration>),
    SubtypeDeclaration(Box<SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `subtype_declaration ::= SUBTYPE identifier IS subtype_indication ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtypeDeclaration {
    pub identifier: Identifier,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF (VHDL-2008): `subtype_indication ::= [ resolution_indication ] type_mark [ constraint ]`
/// EBNF (VHDL-87/93): `subtype_indication ::= [ resolution_function_name ] type_mark [ constraint ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtypeIndication {
    pub resolution: Option<ResolutionIndication>,
    pub type_mark: TypeMark,
    pub constraint: Option<Constraint>,
}

/// EBNF (VHDL-2008): `resolution_indication ::= resolution_function_name | ( element_resolution )`
/// EBNF (VHDL-87/93): the resolution is simply a `resolution_function_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionIndication {
    FunctionName(Box<Name>),
    /// VHDL-2008.
    ElementResolution(Box<ElementResolution>),
}

/// EBNF: `element_resolution ::= array_element_resolution | record_resolution` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementResolution {
    Array(ArrayElementResolution),
    Record(RecordResolution),
}

/// EBNF: `array_element_resolution ::= resolution_indication` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayElementResolution {
    pub resolution: Box<ResolutionIndication>,
}

/// EBNF: `record_resolution ::= record_element_resolution { , record_element_resolution }`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordResolution {
    pub elements: Vec<RecordElementResolution>,
}

/// EBNF: `record_element_resolution ::= record_element_simple_name resolution_indication`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordElementResolution {
    pub element_name: SimpleName,
    pub resolution: ResolutionIndication,
}

/// EBNF (VHDL-2008): `constraint ::= range_constraint | array_constraint | record_constraint`
/// EBNF (VHDL-87/93): `constraint ::= range_constraint | index_constraint`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    Range(RangeConstraint),
    /// In VHDL-87/93, this is `index_constraint`.
    Index(IndexConstraint),
    /// VHDL-2008.
    Array(ArrayConstraint),
    /// VHDL-2008.
    Record(RecordConstraint),
}

/// EBNF: `range_constraint ::= RANGE range`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeConstraint {
    pub range: Range,
}

/// EBNF: `range ::= range_attribute_name | simple_expression direction simple_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    Attribute(Box<Name>),
    Explicit {
        left: SimpleExpression,
        direction: Direction,
        right: SimpleExpression,
    },
}

/// EBNF: `discrete_range ::= discrete_subtype_indication | range`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscreteRange {
    SubtypeIndication(SubtypeIndication),
    Range(Range),
}

/// EBNF: `index_constraint ::= ( discrete_range { , discrete_range } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexConstraint {
    pub ranges: Vec<DiscreteRange>,
}

/// EBNF: `index_subtype_definition ::= type_mark RANGE <>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSubtypeDefinition {
    pub type_mark: TypeMark,
}

/// EBNF: `array_constraint ::= index_constraint [ array_element_constraint ]
///     | ( OPEN ) [ array_element_constraint ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayConstraint {
    IndexConstraint {
        index_constraint: IndexConstraint,
        element_constraint: Option<Box<ElementConstraint>>,
    },
    Open {
        element_constraint: Option<Box<ElementConstraint>>,
    },
}

/// EBNF: `array_element_constraint ::= element_constraint` (VHDL-2008)
pub type ArrayElementConstraint = ElementConstraint;

/// EBNF: `element_constraint ::= array_constraint | record_constraint` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementConstraint {
    Array(ArrayConstraint),
    Record(RecordConstraint),
}

/// EBNF: `record_constraint ::= ( record_element_constraint
///     { , record_element_constraint } )` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordConstraint {
    pub elements: Vec<RecordElementConstraint>,
}

/// EBNF: `record_element_constraint ::= record_element_simple_name element_constraint`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordElementConstraint {
    pub element_name: SimpleName,
    pub constraint: ElementConstraint,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for TypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Full(inner) => inner.format(f, indent_level),
            Self::Incomplete(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for FullTypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "type ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is ")?;
        self.type_definition.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for IncompleteTypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "type ")?;
        self.identifier.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for TypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Scalar(inner) => inner.format(f, indent_level),
            Self::Composite(inner) => inner.format(f, indent_level),
            Self::Access(inner) => inner.format(f, indent_level),
            Self::File(inner) => inner.format(f, indent_level),
            Self::Protected(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for TypeMark {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::TypeName(name) => name.format(f, indent_level),
            Self::SubtypeName(name) => name.format(f, indent_level),
        }
    }
}

impl AstNode for ScalarTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Enumeration(inner) => inner.format(f, indent_level),
            Self::Integer(inner) => inner.format(f, indent_level),
            Self::Floating(inner) => inner.format(f, indent_level),
            Self::Physical(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for EnumerationTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.literals, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for IntegerTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.range_constraint.format(f, indent_level)
    }
}

impl AstNode for FloatingTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.range_constraint.format(f, indent_level)
    }
}

impl AstNode for PhysicalTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.range_constraint.format(f, indent_level)?;
        writeln!(f)?;
        write_indent(f, indent_level + 1)?;
        writeln!(f, "units")?;
        write_indent(f, indent_level + 2)?;
        self.primary_unit.format(f, indent_level + 2)?;
        writeln!(f, ";")?;
        for unit in &self.secondary_units {
            write_indent(f, indent_level + 2)?;
            unit.format(f, indent_level + 2)?;
            writeln!(f, ";")?;
        }
        write_indent(f, indent_level + 1)?;
        write!(f, "end units")?;
        if let Some(name) = &self.end_name {
            write!(f, " ")?;
            name.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for PrimaryUnitDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for SecondaryUnitDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)?;
        write!(f, " = ")?;
        self.literal.format(f, indent_level)
    }
}

impl AstNode for CompositeTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ArrayTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Unbounded(inner) => inner.format(f, indent_level),
            Self::Constrained(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for UnboundedArrayDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "array (")?;
        format_comma_separated(&self.index_subtypes, f, indent_level)?;
        write!(f, ") of ")?;
        self.element_subtype.format(f, indent_level)
    }
}

impl AstNode for ConstrainedArrayDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "array ")?;
        self.index_constraint.format(f, indent_level)?;
        write!(f, " of ")?;
        self.element_subtype.format(f, indent_level)
    }
}

impl AstNode for RecordTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        writeln!(f, "record")?;
        for elem in &self.elements {
            elem.format(f, indent_level + 1)?;
            writeln!(f, ";")?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end record")?;
        if let Some(name) = &self.end_name {
            write!(f, " ")?;
            name.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ElementDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype.format(f, indent_level)
    }
}

impl AstNode for ElementSubtypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.subtype_indication.format(f, indent_level)
    }
}

impl AstNode for AccessTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "access ")?;
        self.subtype_indication.format(f, indent_level)
    }
}

impl AstNode for FileTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "file of ")?;
        self.type_mark.format(f, indent_level)
    }
}

impl AstNode for ProtectedTypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Declaration(inner) => inner.format(f, indent_level),
            Self::Body(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ProtectedTypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        writeln!(f, "protected")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end protected")?;
        if let Some(name) = &self.end_name {
            write!(f, " ")?;
            name.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ProtectedTypeDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProtectedTypeDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubprogramDeclaration(inner) => inner.format(f, indent_level),
            Self::SubprogramInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ProtectedTypeBody {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        writeln!(f, "protected body")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end protected body")?;
        if let Some(name) = &self.end_name {
            write!(f, " ")?;
            name.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ProtectedTypeBodyDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProtectedTypeBodyDeclarativeItem {
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

impl AstNode for SubtypeDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "subtype ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is ")?;
        self.subtype_indication.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for SubtypeIndication {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(resolution) = &self.resolution {
            resolution.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.type_mark.format(f, indent_level)?;
        if let Some(constraint) = &self.constraint {
            write!(f, " ")?;
            constraint.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ResolutionIndication {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::FunctionName(name) => name.format(f, indent_level),
            Self::ElementResolution(res) => {
                write!(f, "(")?;
                res.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}

impl AstNode for ElementResolution {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ArrayElementResolution {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.resolution.format(f, indent_level)
    }
}

impl AstNode for RecordResolution {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.elements, f, indent_level)
    }
}

impl AstNode for RecordElementResolution {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.element_name.format(f, indent_level)?;
        write!(f, " ")?;
        self.resolution.format(f, indent_level)
    }
}

impl AstNode for Constraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Range(inner) => inner.format(f, indent_level),
            Self::Index(inner) => inner.format(f, indent_level),
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for RangeConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "range ")?;
        self.range.format(f, indent_level)
    }
}

impl AstNode for Range {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Attribute(name) => name.format(f, indent_level),
            Self::Explicit {
                left,
                direction,
                right,
            } => {
                left.format(f, indent_level)?;
                write!(f, " ")?;
                direction.format(f, indent_level)?;
                write!(f, " ")?;
                right.format(f, indent_level)
            }
        }
    }
}

impl AstNode for DiscreteRange {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubtypeIndication(inner) => inner.format(f, indent_level),
            Self::Range(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for IndexConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.ranges, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for IndexSubtypeDefinition {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.type_mark.format(f, indent_level)?;
        write!(f, " range <>")
    }
}

impl AstNode for ArrayConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::IndexConstraint {
                index_constraint,
                element_constraint,
            } => {
                index_constraint.format(f, indent_level)?;
                if let Some(ec) = element_constraint {
                    write!(f, " ")?;
                    ec.format(f, indent_level)?;
                }
                Ok(())
            }
            Self::Open {
                element_constraint,
            } => {
                write!(f, "(open)")?;
                if let Some(ec) = element_constraint {
                    write!(f, " ")?;
                    ec.format(f, indent_level)?;
                }
                Ok(())
            }
        }
    }
}

impl AstNode for ElementConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for RecordConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.elements, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for RecordElementConstraint {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.element_name.format(f, indent_level)?;
        write!(f, " ")?;
        self.constraint.format(f, indent_level)
    }
}
