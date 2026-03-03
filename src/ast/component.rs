//! Component declaration and instantiation AST nodes.

use super::common::*;
use super::name::Name;
use super::node::{AstNode, write_indent};
use crate::parser::{Parser, ParseError};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ComponentDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "component ")?;
        self.identifier.format(f, 0)?;
        writeln!(f, " is")?;
        if let Some(ref gc) = self.generic_clause {
            gc.format(f, indent_level + 1)?;
        }
        if let Some(ref pc) = self.port_clause {
            pc.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end component")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ComponentInstantiationStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, 0)?;
        write!(f, " : ")?;
        self.unit.format(f, 0)?;
        writeln!(f)?;
        if let Some(ref gma) = self.generic_map_aspect {
            gma.format(f, indent_level + 1)?;
        }
        if let Some(ref pma) = self.port_map_aspect {
            pma.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for InstantiatedUnit {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            InstantiatedUnit::Component { has_component_keyword, name } => {
                if *has_component_keyword {
                    write!(f, "component ")?;
                }
                name.format(f, indent_level)
            }
            InstantiatedUnit::Entity { name, architecture } => {
                write!(f, "entity ")?;
                name.format(f, indent_level)?;
                if let Some(arch) = architecture {
                    write!(f, "(")?;
                    arch.format(f, 0)?;
                    write!(f, ")")?;
                }
                Ok(())
            }
            InstantiatedUnit::Configuration(name) => {
                write!(f, "configuration ")?;
                name.format(f, indent_level)
            }
        }
    }
}
