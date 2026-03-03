//! Configuration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;

/// EBNF (VHDL-2008): `configuration_declaration ::= CONFIGURATION identifier OF entity_name IS
///     configuration_declarative_part { verification_unit_binding_indication ; }
///     block_configuration END [ CONFIGURATION ] [ configuration_simple_name ] ;`
/// EBNF (VHDL-87/93): omits verification_unit_binding_indication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationDeclaration {
    pub identifier: Identifier,
    pub entity_name: SimpleName,
    pub declarative_part: ConfigurationDeclarativePart,
    /// VHDL-2008.
    pub verification_units: Vec<VerificationUnitBindingIndication>,
    pub block_configuration: BlockConfiguration,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `configuration_declarative_part ::= { configuration_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationDeclarativePart {
    pub items: Vec<ConfigurationDeclarativeItem>,
}

/// EBNF: `configuration_declarative_item ::= use_clause | attribute_specification
///     | group_declaration`
/// EBNF (VHDL-87): `configuration_declarative_item ::= use_clause | attribute_specification`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationDeclarativeItem {
    UseClause(super::context::UseClause),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `configuration_item ::= block_configuration | component_configuration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationItem {
    Block(BlockConfiguration),
    Component(ComponentConfiguration),
}

/// EBNF (VHDL-2008): `configuration_specification ::= simple_configuration_specification
///     | compound_configuration_specification`
/// EBNF (VHDL-87/93): `configuration_specification ::= FOR component_specification
///     binding_indication ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationSpecification {
    Simple(SimpleConfigurationSpecification),
    /// VHDL-2008.
    Compound(CompoundConfigurationSpecification),
}

/// EBNF (VHDL-2008): `simple_configuration_specification ::= FOR component_specification
///     binding_indication ; [ END FOR ; ]`
/// EBNF (VHDL-87/93): `...FOR component_specification binding_indication ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleConfigurationSpecification {
    pub component_spec: ComponentSpecification,
    pub binding: BindingIndication,
    /// VHDL-2008.
    pub has_end_for: bool,
}

/// EBNF: `compound_configuration_specification ::= FOR component_specification
///     binding_indication ; verification_unit_binding_indication ;
///     { verification_unit_binding_indication ; } END FOR ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundConfigurationSpecification {
    pub component_spec: ComponentSpecification,
    pub binding: BindingIndication,
    pub verification_units: Vec<VerificationUnitBindingIndication>,
}

/// EBNF: `block_configuration ::= FOR block_specification { use_clause }
///     { configuration_item } END FOR ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConfiguration {
    pub block_spec: BlockSpecification,
    pub use_clauses: Vec<super::context::UseClause>,
    pub items: Vec<ConfigurationItem>,
}

/// EBNF (VHDL-2008): `block_specification ::= architecture_name | block_statement_label
///     | generate_statement_label [ ( generate_specification ) ]`
/// EBNF (VHDL-87/93): `...[ ( index_specification ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockSpecification {
    Architecture(SimpleName),
    Block(Label),
    Generate {
        label: Label,
        specification: Option<GenerateOrIndexSpecification>,
    },
}

/// Combined generate/index specification for block specifications.
///
/// EBNF (VHDL-2008): `generate_specification ::= static_discrete_range
///     | static_expression | alternative_label`
/// EBNF (VHDL-87/93): `index_specification ::= discrete_range | static_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateOrIndexSpecification {
    DiscreteRange(super::type_def::DiscreteRange),
    Expression(Expression),
    /// VHDL-2008.
    AlternativeLabel(Label),
}

/// EBNF: `component_configuration ::= FOR component_specification
///     [ binding_indication ; ] { verification_unit_binding_indication ; }
///     [ block_configuration ] END FOR ;`
/// VHDL-87/93 omit verification_unit_binding_indication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentConfiguration {
    pub component_spec: ComponentSpecification,
    pub binding: Option<BindingIndication>,
    /// VHDL-2008.
    pub verification_units: Vec<VerificationUnitBindingIndication>,
    pub block_configuration: Option<BlockConfiguration>,
}

/// EBNF: `component_specification ::= instantiation_list : component_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSpecification {
    pub instantiation_list: InstantiationList,
    pub component_name: Box<Name>,
}

/// EBNF: `binding_indication ::= [ USE entity_aspect ] [ generic_map_aspect ]
///     [ port_map_aspect ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingIndication {
    pub entity_aspect: Option<EntityAspect>,
    pub generic_map_aspect: Option<super::interface::GenericMapAspect>,
    pub port_map_aspect: Option<super::interface::PortMapAspect>,
}

/// EBNF: `entity_aspect ::= ENTITY entity_name [ ( architecture_identifier ) ]
///     | CONFIGURATION configuration_name | OPEN`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityAspect {
    Entity {
        entity_name: Box<Name>,
        architecture: Option<Identifier>,
    },
    Configuration(Box<Name>),
    Open,
}

/// EBNF: `verification_unit_binding_indication ::= USE VUNIT verification_unit_list`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationUnitBindingIndication {
    pub unit_list: VerificationUnitList,
}

/// EBNF: `verification_unit_list ::= verification_unit_name { , verification_unit_name }`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationUnitList {
    pub names: Vec<Box<Name>>,
}

/// EBNF: `instantiation_list ::= instantiation_label { , instantiation_label }
///     | OTHERS | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstantiationList {
    Labels(Vec<Label>),
    Others,
    All,
}
