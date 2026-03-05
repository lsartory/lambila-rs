//! Type definition AST nodes.

use super::common::*;
use super::expression::SimpleExpression;
use super::literal::{EnumerationLiteral, PhysicalLiteral};
use super::name::Name;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `type_declaration ::= full_type_declaration | incomplete_type_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDeclaration {
    Full(Box<FullTypeDeclaration>),
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Type)?;
        let identifier = Identifier::parse(parser)?;

        // Incomplete type declaration: TYPE identifier ;
        if parser.consume_if(TokenKind::Semicolon).is_some() {
            return Ok(TypeDeclaration::Incomplete(IncompleteTypeDeclaration {
                identifier,
            }));
        }

        // Full type declaration: TYPE identifier IS type_definition ;
        parser.expect_keyword(KeywordKind::Is)?;
        let type_definition = TypeDefinition::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;

        Ok(TypeDeclaration::Full(Box::new(FullTypeDeclaration {
            identifier,
            type_definition,
        })))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Full(inner) => inner.format(f, indent_level),
            Self::Incomplete(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for FullTypeDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Type)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let type_definition = TypeDefinition::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(FullTypeDeclaration {
            identifier,
            type_definition,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Type)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(IncompleteTypeDeclaration { identifier })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "type ")?;
        self.identifier.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for TypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at(TokenKind::LeftParen) {
            // Enumeration type definition: ( enumeration_literal { , enumeration_literal } )
            let enum_def = EnumerationTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::Scalar(ScalarTypeDefinition::Enumeration(
                enum_def,
            )));
        }

        if parser.at_keyword(KeywordKind::Range) {
            // Could be integer, floating, or physical type definition
            // All start with range_constraint. Physical has UNITS after.
            let range_constraint = RangeConstraint::parse(parser)?;
            if parser.at_keyword(KeywordKind::Units) {
                // Physical type definition
                parser.expect_keyword(KeywordKind::Units)?;
                let primary_unit = PrimaryUnitDeclaration::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                let mut secondary_units = Vec::new();
                while !parser.at_keyword(KeywordKind::End) {
                    secondary_units.push(SecondaryUnitDeclaration::parse(parser)?);
                    parser.expect(TokenKind::Semicolon)?;
                }
                parser.expect_keyword(KeywordKind::End)?;
                parser.expect_keyword(KeywordKind::Units)?;
                let end_name = if parser.at(TokenKind::Identifier)
                    || parser.at(TokenKind::ExtendedIdentifier)
                {
                    Some(SimpleName::parse(parser)?)
                } else {
                    None
                };
                return Ok(TypeDefinition::Scalar(ScalarTypeDefinition::Physical(
                    PhysicalTypeDefinition {
                        range_constraint,
                        primary_unit,
                        secondary_units,
                        end_name,
                    },
                )));
            }
            // Integer or floating -- we can't easily distinguish at the AST level,
            // so default to integer. (Semantic analysis would distinguish them.)
            return Ok(TypeDefinition::Scalar(ScalarTypeDefinition::Integer(
                IntegerTypeDefinition { range_constraint },
            )));
        }

        if parser.at_keyword(KeywordKind::Array) {
            let array_def = ArrayTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::Composite(CompositeTypeDefinition::Array(
                array_def,
            )));
        }

        if parser.at_keyword(KeywordKind::Record) {
            let record_def = RecordTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::Composite(CompositeTypeDefinition::Record(
                record_def,
            )));
        }

        if parser.at_keyword(KeywordKind::Access) {
            let access_def = AccessTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::Access(access_def));
        }

        if parser.at_keyword(KeywordKind::File) {
            let file_def = FileTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::File(file_def));
        }

        if parser.at_keyword(KeywordKind::Protected) {
            let protected_def = ProtectedTypeDefinition::parse(parser)?;
            return Ok(TypeDefinition::Protected(protected_def));
        }

        Err(parser.error("expected type definition"))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // type_mark ::= type_name | subtype_name
        // At the syntactic level we cannot distinguish between a type name and a subtype name.
        // We parse a Name and wrap it as TypeName.
        let name = Name::parse(parser)?;
        Ok(TypeMark::TypeName(Box::new(name)))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::TypeName(name) => name.format(f, indent_level),
            Self::SubtypeName(name) => name.format(f, indent_level),
        }
    }
}

impl AstNode for ScalarTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at(TokenKind::LeftParen) {
            let enum_def = EnumerationTypeDefinition::parse(parser)?;
            return Ok(ScalarTypeDefinition::Enumeration(enum_def));
        }

        if parser.at_keyword(KeywordKind::Range) {
            let range_constraint = RangeConstraint::parse(parser)?;
            if parser.at_keyword(KeywordKind::Units) {
                parser.expect_keyword(KeywordKind::Units)?;
                let primary_unit = PrimaryUnitDeclaration::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                let mut secondary_units = Vec::new();
                while !parser.at_keyword(KeywordKind::End) {
                    secondary_units.push(SecondaryUnitDeclaration::parse(parser)?);
                    parser.expect(TokenKind::Semicolon)?;
                }
                parser.expect_keyword(KeywordKind::End)?;
                parser.expect_keyword(KeywordKind::Units)?;
                let end_name = if parser.at(TokenKind::Identifier)
                    || parser.at(TokenKind::ExtendedIdentifier)
                {
                    Some(SimpleName::parse(parser)?)
                } else {
                    None
                };
                return Ok(ScalarTypeDefinition::Physical(PhysicalTypeDefinition {
                    range_constraint,
                    primary_unit,
                    secondary_units,
                    end_name,
                }));
            }
            return Ok(ScalarTypeDefinition::Integer(IntegerTypeDefinition {
                range_constraint,
            }));
        }

        Err(parser.error("expected scalar type definition"))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::LeftParen)?;
        let mut literals = vec![EnumerationLiteral::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            literals.push(EnumerationLiteral::parse(parser)?);
        }
        parser.expect(TokenKind::RightParen)?;
        Ok(EnumerationTypeDefinition { literals })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.literals, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for IntegerTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let range_constraint = RangeConstraint::parse(parser)?;
        Ok(IntegerTypeDefinition { range_constraint })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.range_constraint.format(f, indent_level)
    }
}

impl AstNode for FloatingTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let range_constraint = RangeConstraint::parse(parser)?;
        Ok(FloatingTypeDefinition { range_constraint })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.range_constraint.format(f, indent_level)
    }
}

impl AstNode for PhysicalTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let range_constraint = RangeConstraint::parse(parser)?;
        parser.expect_keyword(KeywordKind::Units)?;
        let primary_unit = PrimaryUnitDeclaration::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        let mut secondary_units = Vec::new();
        while !parser.at_keyword(KeywordKind::End) {
            secondary_units.push(SecondaryUnitDeclaration::parse(parser)?);
            parser.expect(TokenKind::Semicolon)?;
        }
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Units)?;
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        Ok(PhysicalTypeDefinition {
            range_constraint,
            primary_unit,
            secondary_units,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifier = Identifier::parse(parser)?;
        Ok(PrimaryUnitDeclaration { identifier })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)
    }
}

impl AstNode for SecondaryUnitDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifier = Identifier::parse(parser)?;
        parser.expect(TokenKind::Equals)?;
        let literal = PhysicalLiteral::parse(parser)?;
        Ok(SecondaryUnitDeclaration {
            identifier,
            literal,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)?;
        write!(f, " = ")?;
        self.literal.format(f, indent_level)
    }
}

impl AstNode for CompositeTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Array) {
            let array_def = ArrayTypeDefinition::parse(parser)?;
            Ok(CompositeTypeDefinition::Array(array_def))
        } else if parser.at_keyword(KeywordKind::Record) {
            let record_def = RecordTypeDefinition::parse(parser)?;
            Ok(CompositeTypeDefinition::Record(record_def))
        } else {
            Err(parser.error("expected composite type definition (array or record)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ArrayTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Array)?;
        parser.expect(TokenKind::LeftParen)?;

        // Try to determine if this is unbounded or constrained.
        // Unbounded: type_mark RANGE <> { , type_mark RANGE <> }
        // Constrained: discrete_range { , discrete_range }
        // Use backtracking: try parsing as index_subtype_definition first.
        let save = parser.save();
        let is_unbounded = {
            // Try to parse: type_mark RANGE <>
            let result = (|| -> Result<(), ParseError> {
                let _tm = TypeMark::parse(parser)?;
                parser.expect_keyword(KeywordKind::Range)?;
                parser.expect(TokenKind::Box)?;
                Ok(())
            })();
            result.is_ok()
        };
        parser.restore(save);

        if is_unbounded {
            let mut index_subtypes = vec![IndexSubtypeDefinition::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                index_subtypes.push(IndexSubtypeDefinition::parse(parser)?);
            }
            parser.expect(TokenKind::RightParen)?;
            parser.expect_keyword(KeywordKind::Of)?;
            let element_subtype = SubtypeIndication::parse(parser)?;
            Ok(ArrayTypeDefinition::Unbounded(UnboundedArrayDefinition {
                index_subtypes,
                element_subtype,
            }))
        } else {
            let mut ranges = vec![DiscreteRange::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                ranges.push(DiscreteRange::parse(parser)?);
            }
            parser.expect(TokenKind::RightParen)?;
            parser.expect_keyword(KeywordKind::Of)?;
            let element_subtype = SubtypeIndication::parse(parser)?;
            Ok(ArrayTypeDefinition::Constrained(
                ConstrainedArrayDefinition {
                    index_constraint: IndexConstraint { ranges },
                    element_subtype,
                },
            ))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Unbounded(inner) => inner.format(f, indent_level),
            Self::Constrained(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for UnboundedArrayDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Array)?;
        parser.expect(TokenKind::LeftParen)?;
        let mut index_subtypes = vec![IndexSubtypeDefinition::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            index_subtypes.push(IndexSubtypeDefinition::parse(parser)?);
        }
        parser.expect(TokenKind::RightParen)?;
        parser.expect_keyword(KeywordKind::Of)?;
        let element_subtype = SubtypeIndication::parse(parser)?;
        Ok(UnboundedArrayDefinition {
            index_subtypes,
            element_subtype,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "array (")?;
        format_comma_separated(&self.index_subtypes, f, indent_level)?;
        write!(f, ") of ")?;
        self.element_subtype.format(f, indent_level)
    }
}

impl AstNode for ConstrainedArrayDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Array)?;
        let index_constraint = IndexConstraint::parse(parser)?;
        parser.expect_keyword(KeywordKind::Of)?;
        let element_subtype = SubtypeIndication::parse(parser)?;
        Ok(ConstrainedArrayDefinition {
            index_constraint,
            element_subtype,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "array ")?;
        self.index_constraint.format(f, indent_level)?;
        write!(f, " of ")?;
        self.element_subtype.format(f, indent_level)
    }
}

impl AstNode for RecordTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Record)?;
        let mut elements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) {
            elements.push(ElementDeclaration::parse(parser)?);
            parser.expect(TokenKind::Semicolon)?;
        }
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Record)?;
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        Ok(RecordTypeDefinition { elements, end_name })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifiers = IdentifierList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype = ElementSubtypeDefinition::parse(parser)?;
        Ok(ElementDeclaration {
            identifiers,
            subtype,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.identifiers.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype.format(f, indent_level)
    }
}

impl AstNode for ElementSubtypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let subtype_indication = SubtypeIndication::parse(parser)?;
        Ok(ElementSubtypeDefinition { subtype_indication })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.subtype_indication.format(f, indent_level)
    }
}

impl AstNode for AccessTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Access)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        Ok(AccessTypeDefinition { subtype_indication })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "access ")?;
        self.subtype_indication.format(f, indent_level)
    }
}

impl AstNode for FileTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::File)?;
        parser.expect_keyword(KeywordKind::Of)?;
        let type_mark = TypeMark::parse(parser)?;
        Ok(FileTypeDefinition { type_mark })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "file of ")?;
        self.type_mark.format(f, indent_level)
    }
}

impl AstNode for ProtectedTypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // PROTECTED is consumed, check if BODY follows
        // Peek ahead: if next two tokens are PROTECTED BODY → ProtectedTypeBody
        if parser.at_keyword(KeywordKind::Protected) {
            // Check if BODY follows PROTECTED
            if let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Body)
            {
                let body = ProtectedTypeBody::parse(parser)?;
                return Ok(ProtectedTypeDefinition::Body(body));
            }
            let decl = ProtectedTypeDeclaration::parse(parser)?;
            return Ok(ProtectedTypeDefinition::Declaration(decl));
        }
        Err(parser.error("expected protected type definition"))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Declaration(inner) => inner.format(f, indent_level),
            Self::Body(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ProtectedTypeDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Protected)?;
        let declarative_part = ProtectedTypeDeclarativePart::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Protected)?;
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        Ok(ProtectedTypeDeclaration {
            declarative_part,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            items.push(ProtectedTypeDeclarativeItem::parse(parser)?);
        }
        Ok(ProtectedTypeDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProtectedTypeDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Discriminate by leading keyword
        if parser.at_keyword(KeywordKind::Procedure)
            || parser.at_keyword(KeywordKind::Function)
            || parser.at_keyword(KeywordKind::Pure)
            || parser.at_keyword(KeywordKind::Impure)
        {
            let decl = super::subprogram::SubprogramDeclaration::parse(parser)?;
            return Ok(ProtectedTypeDeclarativeItem::SubprogramDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Attribute) {
            let spec = super::attribute::AttributeSpecification::parse(parser)?;
            return Ok(ProtectedTypeDeclarativeItem::AttributeSpecification(
                Box::new(spec),
            ));
        }

        if parser.at_keyword(KeywordKind::Use) {
            let use_clause = super::context::UseClause::parse(parser)?;
            return Ok(ProtectedTypeDeclarativeItem::UseClause(use_clause));
        }

        Err(parser.error("expected protected type declarative item"))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Protected)?;
        parser.expect_keyword(KeywordKind::Body)?;
        let declarative_part = ProtectedTypeBodyDeclarativePart::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Protected)?;
        parser.expect_keyword(KeywordKind::Body)?;
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        Ok(ProtectedTypeBody {
            declarative_part,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            items.push(ProtectedTypeBodyDeclarativeItem::parse(parser)?);
        }
        Ok(ProtectedTypeBodyDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProtectedTypeBodyDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Discriminate by leading keyword
        if parser.at_keyword(KeywordKind::Procedure)
            || parser.at_keyword(KeywordKind::Function)
            || parser.at_keyword(KeywordKind::Pure)
            || parser.at_keyword(KeywordKind::Impure)
        {
            // Could be subprogram declaration or subprogram body.
            // Try parsing as subprogram body first (it includes IS ... BEGIN ... END).
            // Use backtracking: subprogram_body has IS after specification,
            // subprogram_declaration has just ;
            let save = parser.save();
            if let Ok(body) = super::subprogram::SubprogramBody::parse(parser) {
                return Ok(ProtectedTypeBodyDeclarativeItem::SubprogramBody(Box::new(
                    body,
                )));
            }
            parser.restore(save);
            let decl = super::subprogram::SubprogramDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::SubprogramDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Package) {
            // Could be package declaration, body, or instantiation
            let save = parser.save();
            if let Ok(body) = super::package::PackageBody::parse(parser) {
                return Ok(ProtectedTypeBodyDeclarativeItem::PackageBody(Box::new(
                    body,
                )));
            }
            parser.restore(save);
            if let Ok(inst) = super::package::PackageInstantiationDeclaration::parse(parser) {
                return Ok(
                    ProtectedTypeBodyDeclarativeItem::PackageInstantiationDeclaration(Box::new(
                        inst,
                    )),
                );
            }
            parser.restore(save);
            let decl = super::package::PackageDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::PackageDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Type) {
            let decl = TypeDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::TypeDeclaration(Box::new(
                decl,
            )));
        }

        if parser.at_keyword(KeywordKind::Subtype) {
            let decl = SubtypeDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::SubtypeDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Constant) {
            let decl = super::object_decl::ConstantDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::ConstantDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Variable) || parser.at_keyword(KeywordKind::Shared) {
            let decl = super::object_decl::VariableDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::VariableDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::File) {
            let decl = super::object_decl::FileDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::FileDeclaration(Box::new(
                decl,
            )));
        }

        if parser.at_keyword(KeywordKind::Alias) {
            let decl = super::object_decl::AliasDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::AliasDeclaration(
                Box::new(decl),
            ));
        }

        if parser.at_keyword(KeywordKind::Attribute) {
            // Could be attribute declaration or attribute specification
            // Attribute declaration: ATTRIBUTE identifier : type_mark ;
            // Attribute specification: ATTRIBUTE designator OF entity_specification IS expression ;
            let save = parser.save();
            if let Ok(decl) = super::attribute::AttributeDeclaration::parse(parser) {
                return Ok(ProtectedTypeBodyDeclarativeItem::AttributeDeclaration(
                    Box::new(decl),
                ));
            }
            parser.restore(save);
            let spec = super::attribute::AttributeSpecification::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::AttributeSpecification(
                Box::new(spec),
            ));
        }

        if parser.at_keyword(KeywordKind::Use) {
            let use_clause = super::context::UseClause::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::UseClause(use_clause));
        }

        if parser.at_keyword(KeywordKind::Group) {
            let save = parser.save();
            if let Ok(template) = super::group::GroupTemplateDeclaration::parse(parser) {
                return Ok(ProtectedTypeBodyDeclarativeItem::GroupTemplateDeclaration(
                    Box::new(template),
                ));
            }
            parser.restore(save);
            let group = super::group::GroupDeclaration::parse(parser)?;
            return Ok(ProtectedTypeBodyDeclarativeItem::GroupDeclaration(
                Box::new(group),
            ));
        }

        Err(parser.error("expected protected type body declarative item"))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Subtype)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(SubtypeDeclaration {
            identifier,
            subtype_indication,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // subtype_indication ::= [ resolution_indication ] type_mark [ constraint ]
        //
        // resolution_indication can be a function name (a name) or ( element_resolution ).
        // type_mark is also a name.
        // Strategy:
        //   1. If we see '(' it could be a parenthesized resolution_indication
        //      (e.g. `(resolved) STD_ULOGIC_VECTOR`). Try with backtracking.
        //   2. Otherwise, parse a name. Then check: if the next token starts another name
        //      (identifier/extended identifier), the first name was the resolution function
        //      and the second is the type_mark. Otherwise, the first name IS the type_mark.
        //   3. After type_mark, optionally parse constraint.

        let mut resolution = None;

        // Check for parenthesized resolution_indication: ( element_resolution ) type_mark
        // Use backtracking: if we see '(' and it's followed by a closing ')' and then a name,
        // it's a resolution indication. Otherwise restore and treat as normal.
        if parser.at(TokenKind::LeftParen) {
            let save = parser.save();
            if let Ok(res) = ResolutionIndication::parse(parser) {
                // After resolution, the next token must start a type_mark (a name)
                if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                    resolution = Some(res);
                } else {
                    // Not a resolution indication — restore and fall through
                    parser.restore(save);
                }
            } else {
                parser.restore(save);
            }
        }

        // Parse first name
        let first_name = Name::parse(parser)?;

        // Check if next token could start another name (i.e., first_name was a resolution function)
        if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
            // First name is resolution_indication, second is type_mark
            resolution = Some(ResolutionIndication::FunctionName(Box::new(first_name)));
            let type_name = Name::parse(parser)?;
            let type_mark = TypeMark::TypeName(Box::new(type_name));

            // Optional constraint
            let constraint = if parser.at_keyword(KeywordKind::Range) {
                Some(Constraint::Range(RangeConstraint::parse(parser)?))
            } else if parser.at(TokenKind::LeftParen) {
                Some(Constraint::Index(IndexConstraint::parse(parser)?))
            } else {
                None
            };

            Ok(SubtypeIndication {
                resolution,
                type_mark,
                constraint,
            })
        } else {
            let type_mark = TypeMark::TypeName(Box::new(first_name));

            // Optional constraint
            let constraint = if parser.at_keyword(KeywordKind::Range) {
                Some(Constraint::Range(RangeConstraint::parse(parser)?))
            } else if parser.at(TokenKind::LeftParen) {
                Some(Constraint::Index(IndexConstraint::parse(parser)?))
            } else {
                None
            };

            Ok(SubtypeIndication {
                resolution,
                type_mark,
                constraint,
            })
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at(TokenKind::LeftParen) {
            // ( element_resolution )
            parser.expect(TokenKind::LeftParen)?;
            let element = ElementResolution::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Ok(ResolutionIndication::ElementResolution(Box::new(element)))
        } else {
            // resolution_function_name
            let name = Name::parse(parser)?;
            Ok(ResolutionIndication::FunctionName(Box::new(name)))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // element_resolution ::= array_element_resolution | record_resolution
        // record_resolution contains record_element_resolution items: simple_name resolution_indication
        // array_element_resolution is just resolution_indication
        //
        // Disambiguation: if we see an identifier followed by another name or '(',
        // it could be record. Use backtracking.
        let save = parser.save();
        if let Ok(record) = RecordResolution::parse(parser) {
            // Verify it parsed at least one element with a name + resolution
            if !record.elements.is_empty() {
                return Ok(ElementResolution::Record(record));
            }
        }
        parser.restore(save);

        // Array element resolution
        let resolution = ResolutionIndication::parse(parser)?;
        Ok(ElementResolution::Array(ArrayElementResolution {
            resolution: Box::new(resolution),
        }))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ArrayElementResolution {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let resolution = ResolutionIndication::parse(parser)?;
        Ok(ArrayElementResolution {
            resolution: Box::new(resolution),
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.resolution.format(f, indent_level)
    }
}

impl AstNode for RecordResolution {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut elements = vec![RecordElementResolution::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            elements.push(RecordElementResolution::parse(parser)?);
        }
        Ok(RecordResolution { elements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.elements, f, indent_level)
    }
}

impl AstNode for RecordElementResolution {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let element_name = SimpleName::parse(parser)?;
        let resolution = ResolutionIndication::parse(parser)?;
        Ok(RecordElementResolution {
            element_name,
            resolution,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.element_name.format(f, indent_level)?;
        write!(f, " ")?;
        self.resolution.format(f, indent_level)
    }
}

impl AstNode for Constraint {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Range) {
            let rc = RangeConstraint::parse(parser)?;
            Ok(Constraint::Range(rc))
        } else if parser.at(TokenKind::LeftParen) {
            // Could be index_constraint, array_constraint, or record_constraint.
            // For simplicity, parse as index_constraint (which covers the common case).
            let ic = IndexConstraint::parse(parser)?;
            Ok(Constraint::Index(ic))
        } else {
            Err(parser.error("expected constraint (range or '(')"))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Range)?;
        let range = Range::parse(parser)?;
        Ok(RangeConstraint { range })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "range ")?;
        self.range.format(f, indent_level)
    }
}

impl AstNode for Range {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // range ::= range_attribute_name | simple_expression direction simple_expression
        // Try: parse simple_expression, check if TO/DOWNTO follows.
        // If so, it's an explicit range.
        // If not, it should be a range_attribute_name (name'RANGE).
        let save = parser.save();
        if let Ok(left) = SimpleExpression::parse(parser)
            && (parser.at_keyword(KeywordKind::To) || parser.at_keyword(KeywordKind::Downto))
        {
            let direction = Direction::parse(parser)?;
            let right = SimpleExpression::parse(parser)?;
            return Ok(Range::Explicit {
                left,
                direction,
                right,
            });
        }
        // Fall back to attribute name
        parser.restore(save);
        let name = Name::parse(parser)?;
        Ok(Range::Attribute(Box::new(name)))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // discrete_range ::= discrete_subtype_indication | range
        // Use backtracking: try to parse as range first (explicit range with direction),
        // then fall back to subtype_indication.
        let save = parser.save();

        // Try explicit range: simple_expression direction simple_expression
        if let Ok(left) = SimpleExpression::parse(parser)
            && (parser.at_keyword(KeywordKind::To) || parser.at_keyword(KeywordKind::Downto))
        {
            let direction = Direction::parse(parser)?;
            let right = SimpleExpression::parse(parser)?;
            return Ok(DiscreteRange::Range(Range::Explicit {
                left,
                direction,
                right,
            }));
        }
        parser.restore(save);

        // Try as range_attribute_name (name'RANGE)
        // We need to check if it looks like a name with 'RANGE attribute
        let save2 = parser.save();
        if let Ok(name) = Name::parse(parser) {
            // Check if this was a range attribute name (ends with 'range)
            if matches!(&name, Name::Attribute(attr) if {
                let attr_name = &attr.designator.name.identifier;
                matches!(attr_name, Identifier::Basic(s) if s.to_lowercase() == "range")
            }) {
                return Ok(DiscreteRange::Range(Range::Attribute(Box::new(name))));
            }
            // Not a range attribute -- could be a subtype_indication that starts with a name
            // Restore and try subtype_indication
        }
        parser.restore(save2);

        // Parse as subtype_indication
        let subtype = SubtypeIndication::parse(parser)?;
        Ok(DiscreteRange::SubtypeIndication(subtype))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubtypeIndication(inner) => inner.format(f, indent_level),
            Self::Range(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for IndexConstraint {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::LeftParen)?;
        let mut ranges = vec![DiscreteRange::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            ranges.push(DiscreteRange::parse(parser)?);
        }
        parser.expect(TokenKind::RightParen)?;
        Ok(IndexConstraint { ranges })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.ranges, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for IndexSubtypeDefinition {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let type_mark = TypeMark::parse(parser)?;
        parser.expect_keyword(KeywordKind::Range)?;
        parser.expect(TokenKind::Box)?;
        Ok(IndexSubtypeDefinition { type_mark })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.type_mark.format(f, indent_level)?;
        write!(f, " range <>")
    }
}

impl AstNode for ArrayConstraint {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::LeftParen)?;
        if parser.at_keyword(KeywordKind::Open) {
            parser.consume();
            parser.expect(TokenKind::RightParen)?;
            let element_constraint = if parser.at(TokenKind::LeftParen) {
                Some(Box::new(ElementConstraint::parse(parser)?))
            } else {
                None
            };
            Ok(ArrayConstraint::Open { element_constraint })
        } else {
            // Parse discrete ranges for index constraint
            let mut ranges = vec![DiscreteRange::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                ranges.push(DiscreteRange::parse(parser)?);
            }
            parser.expect(TokenKind::RightParen)?;
            let index_constraint = IndexConstraint { ranges };
            let element_constraint = if parser.at(TokenKind::LeftParen) {
                Some(Box::new(ElementConstraint::parse(parser)?))
            } else {
                None
            };
            Ok(ArrayConstraint::IndexConstraint {
                index_constraint,
                element_constraint,
            })
        }
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
            Self::Open { element_constraint } => {
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // element_constraint ::= array_constraint | record_constraint
        // Both start with '('. Distinguish by content.
        // Record constraint: ( simple_name element_constraint { , ... } )
        // Array constraint: ( OPEN ) or ( discrete_range { , ... } )
        //
        // Use backtracking: try record_constraint first, fall back to array_constraint.
        let save = parser.save();
        if let Ok(record) = RecordConstraint::parse(parser) {
            return Ok(ElementConstraint::Record(record));
        }
        parser.restore(save);
        let array = ArrayConstraint::parse(parser)?;
        Ok(ElementConstraint::Array(array))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Array(inner) => inner.format(f, indent_level),
            Self::Record(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for RecordConstraint {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::LeftParen)?;
        let mut elements = vec![RecordElementConstraint::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            elements.push(RecordElementConstraint::parse(parser)?);
        }
        parser.expect(TokenKind::RightParen)?;
        Ok(RecordConstraint { elements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.elements, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for RecordElementConstraint {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let element_name = SimpleName::parse(parser)?;
        let constraint = ElementConstraint::parse(parser)?;
        Ok(RecordElementConstraint {
            element_name,
            constraint,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.element_name.format(f, indent_level)?;
        write!(f, " ")?;
        self.constraint.format(f, indent_level)
    }
}
