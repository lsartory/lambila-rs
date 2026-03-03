//! Component declaration and instantiation AST nodes.

use super::common::*;
use super::name::Name;

/// EBNF (VHDL-2008): `component_declaration ::= COMPONENT identifier [ IS ]
///     [ local_generic_clause ] [ local_port_clause ] END COMPONENT [ component_simple_name ] ;`
/// EBNF (VHDL-87): `...END COMPONENT ;` (no IS, no closing name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDeclaration {
    pub identifier: Identifier,
    pub generic_clause: Option<super::interface::GenericClause>,
    pub port_clause: Option<super::interface::PortClause>,
    pub end_name: Option<SimpleName>,
}

/// EBNF (VHDL-93+): `component_instantiation_statement ::= instantiation_label :
///     instantiated_unit [ generic_map_aspect ] [ port_map_aspect ] ;`
/// EBNF (VHDL-87): `component_instantiation_statement ::= instantiation_label :
///     component_name [ generic_map_aspect ] [ port_map_aspect ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInstantiationStatement {
    pub label: Label,
    pub unit: InstantiatedUnit,
    pub generic_map_aspect: Option<super::interface::GenericMapAspect>,
    pub port_map_aspect: Option<super::interface::PortMapAspect>,
}

/// EBNF (VHDL-93+): `instantiated_unit ::= [ COMPONENT ] component_name
///     | ENTITY entity_name [ ( architecture_identifier ) ]
///     | CONFIGURATION configuration_name`
/// EBNF (VHDL-87): always a component name (no direct entity/config instantiation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstantiatedUnit {
    Component {
        has_component_keyword: bool,
        name: Box<Name>,
    },
    /// VHDL-93+.
    Entity {
        name: Box<Name>,
        architecture: Option<Identifier>,
    },
    /// VHDL-93+.
    Configuration(Box<Name>),
}
