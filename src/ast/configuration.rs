//! Configuration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use crate::parser::{ParseError, Parser};

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

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ConfigurationDeclaration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "configuration ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " of ")?;
        self.entity_name.format(f, indent_level)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        for vu in &self.verification_units {
            vu.format(f, indent_level + 1)?;
            writeln!(f, ";")?;
        }
        self.block_configuration.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end configuration")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ConfigurationDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ConfigurationDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ConfigurationDeclarativeItem::UseClause(uc) => uc.format(f, indent_level),
            ConfigurationDeclarativeItem::AttributeSpecification(attr) => {
                attr.format(f, indent_level)
            }
            ConfigurationDeclarativeItem::GroupDeclaration(grp) => grp.format(f, indent_level),
        }
    }
}

impl AstNode for ConfigurationItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ConfigurationItem::Block(bc) => bc.format(f, indent_level),
            ConfigurationItem::Component(cc) => cc.format(f, indent_level),
        }
    }
}

impl AstNode for ConfigurationSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ConfigurationSpecification::Simple(s) => s.format(f, indent_level),
            ConfigurationSpecification::Compound(c) => c.format(f, indent_level),
        }
    }
}

impl AstNode for SimpleConfigurationSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "for ")?;
        self.component_spec.format(f, indent_level)?;
        writeln!(f)?;
        self.binding.format(f, indent_level + 1)?;
        writeln!(f, ";")?;
        if self.has_end_for {
            write_indent(f, indent_level)?;
            writeln!(f, "end for;")?;
        }
        Ok(())
    }
}

impl AstNode for CompoundConfigurationSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "for ")?;
        self.component_spec.format(f, indent_level)?;
        writeln!(f)?;
        self.binding.format(f, indent_level + 1)?;
        writeln!(f, ";")?;
        for vu in &self.verification_units {
            vu.format(f, indent_level + 1)?;
            writeln!(f, ";")?;
        }
        write_indent(f, indent_level)?;
        writeln!(f, "end for;")
    }
}

impl AstNode for BlockConfiguration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "for ")?;
        self.block_spec.format(f, indent_level)?;
        writeln!(f)?;
        format_lines(&self.use_clauses, f, indent_level + 1)?;
        format_lines(&self.items, f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "end for;")
    }
}

impl AstNode for BlockSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            BlockSpecification::Architecture(name) => name.format(f, indent_level),
            BlockSpecification::Block(label) => label.format(f, indent_level),
            BlockSpecification::Generate {
                label,
                specification,
            } => {
                label.format(f, indent_level)?;
                if let Some(spec) = specification {
                    write!(f, "(")?;
                    spec.format(f, indent_level)?;
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl AstNode for GenerateOrIndexSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GenerateOrIndexSpecification::DiscreteRange(dr) => dr.format(f, indent_level),
            GenerateOrIndexSpecification::Expression(expr) => expr.format(f, indent_level),
            GenerateOrIndexSpecification::AlternativeLabel(label) => label.format(f, indent_level),
        }
    }
}

impl AstNode for ComponentConfiguration {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "for ")?;
        self.component_spec.format(f, indent_level)?;
        writeln!(f)?;
        if let Some(ref binding) = self.binding {
            binding.format(f, indent_level + 1)?;
            writeln!(f, ";")?;
        }
        for vu in &self.verification_units {
            vu.format(f, indent_level + 1)?;
            writeln!(f, ";")?;
        }
        if let Some(ref bc) = self.block_configuration {
            bc.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        writeln!(f, "end for;")
    }
}

impl AstNode for ComponentSpecification {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.instantiation_list.format(f, indent_level)?;
        write!(f, " : ")?;
        self.component_name.format(f, indent_level)
    }
}

impl AstNode for BindingIndication {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(ref ea) = self.entity_aspect {
            write_indent(f, indent_level)?;
            write!(f, "use ")?;
            ea.format(f, indent_level)?;
            writeln!(f)?;
        }
        if let Some(ref gm) = self.generic_map_aspect {
            gm.format(f, indent_level)?;
        }
        if let Some(ref pm) = self.port_map_aspect {
            pm.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for EntityAspect {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            EntityAspect::Entity {
                entity_name,
                architecture,
            } => {
                write!(f, "entity ")?;
                entity_name.format(f, indent_level)?;
                if let Some(arch) = architecture {
                    write!(f, "(")?;
                    arch.format(f, indent_level)?;
                    write!(f, ")")?;
                }
                Ok(())
            }
            EntityAspect::Configuration(name) => {
                write!(f, "configuration ")?;
                name.format(f, indent_level)
            }
            EntityAspect::Open => write!(f, "open"),
        }
    }
}

impl AstNode for VerificationUnitBindingIndication {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "use vunit ")?;
        self.unit_list.format(f, indent_level)
    }
}

impl AstNode for VerificationUnitList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            name.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for InstantiationList {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            InstantiationList::Labels(labels) => format_comma_separated(labels, f, indent_level),
            InstantiationList::Others => write!(f, "others"),
            InstantiationList::All => write!(f, "all"),
        }
    }
}
