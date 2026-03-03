//! Configuration AST nodes.

use super::common::*;
use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // CONFIGURATION identifier OF entity_name IS
        //     configuration_declarative_part
        //     { verification_unit_binding_indication ; }
        //     block_configuration
        // END [ CONFIGURATION ] [ configuration_simple_name ] ;
        parser.expect_keyword(KeywordKind::Configuration)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Of)?;
        let entity_name = SimpleName::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let declarative_part = ConfigurationDeclarativePart::parse(parser)?;
        // Parse optional verification_unit_binding_indications (USE VUNIT ...)
        let mut verification_units = Vec::new();
        while parser.at_keyword(KeywordKind::Use) {
            // Check if this is USE VUNIT (verification unit) vs USE selected_name (use clause)
            if let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                let vu = VerificationUnitBindingIndication::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                verification_units.push(vu);
                continue;
            }
            break;
        }
        let block_configuration = BlockConfiguration::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.consume_if_keyword(KeywordKind::Configuration);
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConfigurationDeclaration {
            identifier,
            entity_name,
            declarative_part,
            verification_units,
            block_configuration,
            end_name,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { configuration_declarative_item }
        // Parse until FOR (start of block_configuration) or END or EOF
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::For)
            && !parser.at_keyword(KeywordKind::End)
            && !parser.eof()
        {
            // If USE VUNIT, stop — those are verification_unit_binding_indications
            if parser.at_keyword(KeywordKind::Use)
                && let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                break;
            }
            items.push(ConfigurationDeclarativeItem::parse(parser)?);
        }
        Ok(ConfigurationDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ConfigurationDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Use)) => Ok(
                ConfigurationDeclarativeItem::UseClause(super::context::UseClause::parse(parser)?),
            ),
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                Ok(ConfigurationDeclarativeItem::AttributeSpecification(
                    Box::new(super::attribute::AttributeSpecification::parse(parser)?),
                ))
            }
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                Ok(ConfigurationDeclarativeItem::GroupDeclaration(Box::new(
                    super::group::GroupDeclaration::parse(parser)?,
                )))
            }
            _ => {
                Err(parser
                    .error("expected configuration declarative item (use, attribute, or group)"))
            }
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Both block_configuration and component_configuration start with FOR.
        // Disambiguate: After FOR, if we see (identifier/ALL/OTHERS) followed by `:`,
        // it's a component_configuration. Otherwise it's a block_configuration.
        let save = parser.save();
        parser.consume(); // FOR
        // Check for component_specification pattern: instantiation_list `:` ...
        let is_component = match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::All)) => {
                parser.consume(); // ALL
                parser.at(TokenKind::Colon)
            }
            Some(TokenKind::Keyword(KeywordKind::Others)) => {
                parser.consume(); // OTHERS
                parser.at(TokenKind::Colon)
            }
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                // Could be label, label list, or block specification name.
                // In component_spec: label {, label} : component_name
                // In block_spec: name [( ... )]
                // If we see identifier then `,` or `:`, it's component spec.
                parser.consume(); // identifier
                parser.at(TokenKind::Colon) || parser.at(TokenKind::Comma)
            }
            _ => false,
        };
        parser.restore(save);
        if is_component {
            Ok(ConfigurationItem::Component(ComponentConfiguration::parse(
                parser,
            )?))
        } else {
            Ok(ConfigurationItem::Block(BlockConfiguration::parse(parser)?))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ConfigurationItem::Block(bc) => bc.format(f, indent_level),
            ConfigurationItem::Component(cc) => cc.format(f, indent_level),
        }
    }
}

impl AstNode for ConfigurationSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // FOR component_specification binding_indication ;
        // Then check for verification_unit_binding_indication(s) and END FOR
        parser.expect_keyword(KeywordKind::For)?;
        let component_spec = ComponentSpecification::parse(parser)?;
        let binding = BindingIndication::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;

        // Check if there are verification_unit_binding_indications (USE VUNIT ...)
        let mut verification_units = Vec::new();
        while parser.at_keyword(KeywordKind::Use) {
            if let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                let vu = VerificationUnitBindingIndication::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                verification_units.push(vu);
                continue;
            }
            break;
        }

        if !verification_units.is_empty() {
            // Compound: must have END FOR ;
            parser.expect_keyword(KeywordKind::End)?;
            parser.expect_keyword(KeywordKind::For)?;
            parser.expect(TokenKind::Semicolon)?;
            Ok(ConfigurationSpecification::Compound(
                CompoundConfigurationSpecification {
                    component_spec,
                    binding,
                    verification_units,
                },
            ))
        } else {
            // Simple: optional END FOR ;
            let has_end_for = if parser.at_keyword(KeywordKind::End) {
                let save = parser.save();
                parser.consume(); // END
                if parser.at_keyword(KeywordKind::For) {
                    parser.consume(); // FOR
                    parser.expect(TokenKind::Semicolon)?;
                    true
                } else {
                    parser.restore(save);
                    false
                }
            } else {
                false
            };
            Ok(ConfigurationSpecification::Simple(
                SimpleConfigurationSpecification {
                    component_spec,
                    binding,
                    has_end_for,
                },
            ))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ConfigurationSpecification::Simple(s) => s.format(f, indent_level),
            ConfigurationSpecification::Compound(c) => c.format(f, indent_level),
        }
    }
}

impl AstNode for SimpleConfigurationSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // FOR component_specification binding_indication ; [ END FOR ; ]
        parser.expect_keyword(KeywordKind::For)?;
        let component_spec = ComponentSpecification::parse(parser)?;
        let binding = BindingIndication::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        let has_end_for = if parser.at_keyword(KeywordKind::End) {
            let save = parser.save();
            parser.consume(); // END
            if parser.at_keyword(KeywordKind::For) {
                parser.consume(); // FOR
                parser.expect(TokenKind::Semicolon)?;
                true
            } else {
                parser.restore(save);
                false
            }
        } else {
            false
        };
        Ok(SimpleConfigurationSpecification {
            component_spec,
            binding,
            has_end_for,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // FOR component_specification binding_indication ;
        //     verification_unit_binding_indication ; { verification_unit_binding_indication ; }
        // END FOR ;
        parser.expect_keyword(KeywordKind::For)?;
        let component_spec = ComponentSpecification::parse(parser)?;
        let binding = BindingIndication::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        let mut verification_units = Vec::new();
        // At least one verification_unit_binding_indication required
        loop {
            if parser.at_keyword(KeywordKind::Use)
                && let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                let vu = VerificationUnitBindingIndication::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                verification_units.push(vu);
                continue;
            }
            break;
        }
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::For)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(CompoundConfigurationSpecification {
            component_spec,
            binding,
            verification_units,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // FOR block_specification { use_clause } { configuration_item } END FOR ;
        parser.expect_keyword(KeywordKind::For)?;
        let block_spec = BlockSpecification::parse(parser)?;
        // Parse use clauses
        let mut use_clauses = Vec::new();
        while parser.at_keyword(KeywordKind::Use) {
            // Check it's not USE VUNIT (verification unit)
            if let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                break;
            }
            use_clauses.push(super::context::UseClause::parse(parser)?);
        }
        // Parse configuration items (each starts with FOR)
        let mut items = Vec::new();
        while parser.at_keyword(KeywordKind::For) {
            items.push(ConfigurationItem::parse(parser)?);
        }
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::For)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(BlockConfiguration {
            block_spec,
            use_clauses,
            items,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Parse a name (simple_name). If followed by `(` it's a generate with spec.
        // We can't really distinguish architecture_name vs block_label vs generate_label
        // syntactically — they're all simple names. We'll parse as Architecture/Generate
        // based on whether there's a parenthesized spec.
        let name = SimpleName::parse(parser)?;
        if parser.at(TokenKind::LeftParen) {
            // generate_statement_label ( generate_specification )
            parser.consume(); // (
            let spec = GenerateOrIndexSpecification::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Ok(BlockSpecification::Generate {
                label: Label {
                    identifier: name.identifier,
                },
                specification: Some(spec),
            })
        } else {
            // Could be architecture_name, block_statement_label, or bare generate_label.
            // We treat it as Architecture (the most common case; semantic analysis
            // can reclassify).
            Ok(BlockSpecification::Architecture(name))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Try to parse as discrete_range first (using backtracking), then as expression.
        // discrete_range has a direction keyword (TO or DOWNTO), so we try that first.
        let save = parser.save();
        match super::type_def::DiscreteRange::parse(parser) {
            Ok(dr) => Ok(GenerateOrIndexSpecification::DiscreteRange(dr)),
            Err(_) => {
                parser.restore(save);
                // Fall back to expression
                let expr = Expression::parse(parser)?;
                Ok(GenerateOrIndexSpecification::Expression(expr))
            }
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // FOR component_specification
        //     [ binding_indication ; ]
        //     { verification_unit_binding_indication ; }
        //     [ block_configuration ]
        // END FOR ;
        parser.expect_keyword(KeywordKind::For)?;
        let component_spec = ComponentSpecification::parse(parser)?;

        // Optional binding_indication (starts with USE, GENERIC, or PORT)
        let binding = if parser.at_keyword(KeywordKind::Use)
            || parser.at_keyword(KeywordKind::Generic)
            || parser.at_keyword(KeywordKind::Port)
        {
            // But USE VUNIT is not a binding indication
            if parser.at_keyword(KeywordKind::Use) {
                if let Some(next) = parser.peek_nth(1) {
                    if next.kind == TokenKind::Keyword(KeywordKind::Vunit) {
                        None
                    } else {
                        let b = BindingIndication::parse(parser)?;
                        parser.expect(TokenKind::Semicolon)?;
                        Some(b)
                    }
                } else {
                    let b = BindingIndication::parse(parser)?;
                    parser.expect(TokenKind::Semicolon)?;
                    Some(b)
                }
            } else {
                let b = BindingIndication::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                Some(b)
            }
        } else {
            None
        };

        // { verification_unit_binding_indication ; }
        let mut verification_units = Vec::new();
        while parser.at_keyword(KeywordKind::Use) {
            if let Some(next) = parser.peek_nth(1)
                && next.kind == TokenKind::Keyword(KeywordKind::Vunit)
            {
                let vu = VerificationUnitBindingIndication::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                verification_units.push(vu);
                continue;
            }
            break;
        }

        // [ block_configuration ] — starts with FOR
        let block_configuration = if parser.at_keyword(KeywordKind::For) {
            Some(BlockConfiguration::parse(parser)?)
        } else {
            None
        };

        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::For)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ComponentConfiguration {
            component_spec,
            binding,
            verification_units,
            block_configuration,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // instantiation_list : component_name
        let instantiation_list = InstantiationList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let component_name = Box::new(Name::parse(parser)?);
        Ok(ComponentSpecification {
            instantiation_list,
            component_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.instantiation_list.format(f, indent_level)?;
        write!(f, " : ")?;
        self.component_name.format(f, indent_level)
    }
}

impl AstNode for BindingIndication {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ USE entity_aspect ] [ generic_map_aspect ] [ port_map_aspect ]
        let entity_aspect = if parser.at_keyword(KeywordKind::Use) {
            // But not USE VUNIT
            if let Some(next) = parser.peek_nth(1) {
                if next.kind == TokenKind::Keyword(KeywordKind::Vunit) {
                    None
                } else {
                    parser.consume(); // USE
                    Some(EntityAspect::parse(parser)?)
                }
            } else {
                parser.consume(); // USE
                Some(EntityAspect::parse(parser)?)
            }
        } else {
            None
        };

        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            Some(super::interface::GenericMapAspect::parse(parser)?)
        } else {
            None
        };

        let port_map_aspect = if parser.at_keyword(KeywordKind::Port) {
            Some(super::interface::PortMapAspect::parse(parser)?)
        } else {
            None
        };

        Ok(BindingIndication {
            entity_aspect,
            generic_map_aspect,
            port_map_aspect,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // ENTITY entity_name [ ( architecture_identifier ) ]
        // | CONFIGURATION configuration_name
        // | OPEN
        if parser.consume_if_keyword(KeywordKind::Entity).is_some() {
            let entity_name = Box::new(Name::parse(parser)?);
            let architecture = if parser.consume_if(TokenKind::LeftParen).is_some() {
                let arch = Identifier::parse(parser)?;
                parser.expect(TokenKind::RightParen)?;
                Some(arch)
            } else {
                None
            };
            Ok(EntityAspect::Entity {
                entity_name,
                architecture,
            })
        } else if parser
            .consume_if_keyword(KeywordKind::Configuration)
            .is_some()
        {
            let name = Box::new(Name::parse(parser)?);
            Ok(EntityAspect::Configuration(name))
        } else if parser.consume_if_keyword(KeywordKind::Open).is_some() {
            Ok(EntityAspect::Open)
        } else {
            Err(parser.error("expected entity aspect (entity, configuration, or open)"))
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // USE VUNIT verification_unit_list
        parser.expect_keyword(KeywordKind::Use)?;
        parser.expect_keyword(KeywordKind::Vunit)?;
        let unit_list = VerificationUnitList::parse(parser)?;
        Ok(VerificationUnitBindingIndication { unit_list })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "use vunit ")?;
        self.unit_list.format(f, indent_level)
    }
}

impl AstNode for VerificationUnitList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // verification_unit_name { , verification_unit_name }
        let mut names = vec![Box::new(Name::parse(parser)?)];
        while parser.consume_if(TokenKind::Comma).is_some() {
            names.push(Box::new(Name::parse(parser)?));
        }
        Ok(VerificationUnitList { names })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // ALL | OTHERS | instantiation_label { , instantiation_label }
        if parser.consume_if_keyword(KeywordKind::All).is_some() {
            Ok(InstantiationList::All)
        } else if parser.consume_if_keyword(KeywordKind::Others).is_some() {
            Ok(InstantiationList::Others)
        } else {
            let mut labels = vec![Label::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                labels.push(Label::parse(parser)?);
            }
            Ok(InstantiationList::Labels(labels))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            InstantiationList::Labels(labels) => format_comma_separated(labels, f, indent_level),
            InstantiationList::Others => write!(f, "others"),
            InstantiationList::All => write!(f, "all"),
        }
    }
}
