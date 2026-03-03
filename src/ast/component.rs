//! Component declaration and instantiation AST nodes.

use super::common::*;
use super::name::Name;
use super::node::{AstNode, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // COMPONENT identifier [ IS ]
        parser.expect_keyword(KeywordKind::Component)?;
        let identifier = Identifier::parse(parser)?;
        // optional IS
        parser.consume_if_keyword(KeywordKind::Is);
        // [ generic_clause ]
        let generic_clause = if parser.at_keyword(KeywordKind::Generic) {
            Some(super::interface::GenericClause::parse(parser)?)
        } else {
            None
        };
        // [ port_clause ]
        let port_clause = if parser.at_keyword(KeywordKind::Port) {
            Some(super::interface::PortClause::parse(parser)?)
        } else {
            None
        };
        // END COMPONENT [ component_simple_name ] ;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Component)?;
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ComponentDeclaration {
            identifier,
            generic_clause,
            port_clause,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // instantiation_label : instantiated_unit [ generic_map_aspect ] [ port_map_aspect ] ;
        // The label has already been parsed by ConcurrentStatement dispatcher,
        // but this standalone parse expects the full form.
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let unit = InstantiatedUnit::parse(parser)?;
        // [ generic_map_aspect ]
        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            Some(super::interface::GenericMapAspect::parse(parser)?)
        } else {
            None
        };
        // [ port_map_aspect ]
        let port_map_aspect = if parser.at_keyword(KeywordKind::Port) {
            Some(super::interface::PortMapAspect::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ComponentInstantiationStatement {
            label,
            unit,
            generic_map_aspect,
            port_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, 0)?;
        write!(f, " : ")?;
        self.unit.format(f, 0)?;
        let has_maps = self.generic_map_aspect.is_some() || self.port_map_aspect.is_some();
        if has_maps {
            writeln!(f)?;
            if let Some(ref gma) = self.generic_map_aspect {
                gma.format(f, indent_level + 1)?;
                if self.port_map_aspect.is_some() {
                    writeln!(f)?;
                }
            }
            if let Some(ref pma) = self.port_map_aspect {
                pma.format(f, indent_level + 1)?;
            }
        }
        writeln!(f, ";")
    }
}

impl AstNode for InstantiatedUnit {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::Entity) {
            // ENTITY entity_name [ ( architecture_identifier ) ]
            parser.consume();
            let name = Name::parse(parser)?;
            let architecture = if parser.consume_if(TokenKind::LeftParen).is_some() {
                let arch = Identifier::parse(parser)?;
                parser.expect(TokenKind::RightParen)?;
                Some(arch)
            } else {
                None
            };
            Ok(InstantiatedUnit::Entity {
                name: Box::new(name),
                architecture,
            })
        } else if parser.at_keyword(KeywordKind::Configuration) {
            // CONFIGURATION configuration_name
            parser.consume();
            let name = Name::parse(parser)?;
            Ok(InstantiatedUnit::Configuration(Box::new(name)))
        } else {
            // [ COMPONENT ] component_name
            let has_component_keyword = parser.consume_if_keyword(KeywordKind::Component).is_some();
            let name = Name::parse(parser)?;
            Ok(InstantiatedUnit::Component {
                has_component_keyword,
                name: Box::new(name),
            })
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            InstantiatedUnit::Component {
                has_component_keyword,
                name,
            } => {
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
