//! Type definition AST nodes.

use super::common::*;
use super::expression::SimpleExpression;
use super::literal::{EnumerationLiteral, PhysicalLiteral};
use super::name::Name;

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
