//! Top-level design file AST nodes.

use super::common::*;
use super::node::{AstNode, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `design_file ::= design_unit { design_unit }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignFile {
    pub design_units: Vec<DesignUnit>,
}

/// EBNF: `design_unit ::= context_clause library_unit`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignUnit {
    pub context_clause: super::context::ContextClause,
    pub library_unit: LibraryUnit,
}

/// EBNF: `library_unit ::= primary_unit | secondary_unit`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryUnit {
    Primary(PrimaryUnit),
    Secondary(SecondaryUnit),
}

/// EBNF (VHDL-2008): `primary_unit ::= entity_declaration | configuration_declaration
///     | package_declaration | package_instantiation_declaration | context_declaration
///     | PSL_Verification_Unit`
/// EBNF (VHDL-87/93): `primary_unit ::= entity_declaration | configuration_declaration
///     | package_declaration`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryUnit {
    Entity(Box<super::entity::EntityDeclaration>),
    Configuration(Box<super::configuration::ConfigurationDeclaration>),
    Package(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageInstantiation(Box<super::package::PackageInstantiationDeclaration>),
    /// VHDL-2008.
    Context(Box<super::context::ContextDeclaration>),
}

/// EBNF: `secondary_unit ::= architecture_body | package_body`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecondaryUnit {
    Architecture(Box<super::architecture::ArchitectureBody>),
    PackageBody(Box<super::package::PackageBody>),
}

/// EBNF (VHDL-2008): `declaration ::= type_declaration | subtype_declaration
///     | object_declaration | interface_declaration | alias_declaration
///     | attribute_declaration | component_declaration | group_template_declaration
///     | group_declaration | entity_declaration | configuration_declaration
///     | subprogram_declaration | package_declaration`
/// EBNF (VHDL-87): omits group_template_declaration and group_declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Type(Box<super::type_def::TypeDeclaration>),
    Subtype(Box<super::type_def::SubtypeDeclaration>),
    Object(Box<super::object_decl::ObjectDeclaration>),
    Interface(Box<super::interface::InterfaceDeclaration>),
    Alias(Box<super::object_decl::AliasDeclaration>),
    Attribute(Box<super::attribute::AttributeDeclaration>),
    Component(Box<super::component::ComponentDeclaration>),
    /// VHDL-93+.
    GroupTemplate(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    Group(Box<super::group::GroupDeclaration>),
    Entity(Box<super::entity::EntityDeclaration>),
    Configuration(Box<super::configuration::ConfigurationDeclaration>),
    Subprogram(Box<super::subprogram::SubprogramDeclaration>),
    Package(Box<super::package::PackageDeclaration>),
}

/// EBNF: `tool_directive ::= ` identifier { graphic_character }` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDirective {
    pub identifier: Identifier,
    pub content: String,
}

impl std::fmt::Display for DesignFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, 0)
    }
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for DesignFile {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // design_unit { design_unit } — parse until EOF
        let mut design_units = Vec::new();
        while !parser.eof() {
            design_units.push(DesignUnit::parse(parser)?);
        }
        Ok(DesignFile { design_units })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, unit) in self.design_units.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            unit.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for DesignUnit {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // context_clause library_unit
        let context_clause = super::context::ContextClause::parse(parser)?;
        let library_unit = LibraryUnit::parse(parser)?;
        Ok(DesignUnit {
            context_clause,
            library_unit,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.context_clause.items, f, indent_level)?;
        self.library_unit.format(f, indent_level)
    }
}

impl AstNode for LibraryUnit {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // library_unit ::= primary_unit | secondary_unit
        // Primary: ENTITY, CONFIGURATION, PACKAGE (not BODY), CONTEXT
        // Secondary: ARCHITECTURE, PACKAGE BODY
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Entity)) => Ok(LibraryUnit::Primary(
                PrimaryUnit::Entity(Box::new(super::entity::EntityDeclaration::parse(parser)?)),
            )),
            Some(TokenKind::Keyword(KeywordKind::Configuration)) => {
                Ok(LibraryUnit::Primary(PrimaryUnit::Configuration(Box::new(
                    super::configuration::ConfigurationDeclaration::parse(parser)?,
                ))))
            }
            Some(TokenKind::Keyword(KeywordKind::Architecture)) => {
                Ok(LibraryUnit::Secondary(SecondaryUnit::Architecture(
                    Box::new(super::architecture::ArchitectureBody::parse(parser)?),
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Context)) => Ok(LibraryUnit::Primary(
                PrimaryUnit::Context(Box::new(super::context::ContextDeclaration::parse(parser)?)),
            )),
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // Disambiguate: PACKAGE BODY -> secondary unit
                //               PACKAGE id IS NEW -> PackageInstantiation primary
                //               PACKAGE id IS -> PackageDeclaration primary
                if let Some(next) = parser.peek_nth(1)
                    && next.kind == TokenKind::Keyword(KeywordKind::Body)
                {
                    return Ok(LibraryUnit::Secondary(SecondaryUnit::PackageBody(
                        Box::new(super::package::PackageBody::parse(parser)?),
                    )));
                }
                // Check for PACKAGE id IS NEW (instantiation)
                let save = parser.save();
                parser.consume(); // PACKAGE
                parser.consume(); // identifier
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                parser.restore(save);
                if is_instantiation {
                    Ok(LibraryUnit::Primary(PrimaryUnit::PackageInstantiation(
                        Box::new(super::package::PackageInstantiationDeclaration::parse(
                            parser,
                        )?),
                    )))
                } else {
                    Ok(LibraryUnit::Primary(PrimaryUnit::Package(Box::new(
                        super::package::PackageDeclaration::parse(parser)?,
                    ))))
                }
            }
            _ => Err(parser.error(
                "expected library unit (entity, architecture, configuration, package, or context)",
            )),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            LibraryUnit::Primary(inner) => inner.format(f, indent_level),
            LibraryUnit::Secondary(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for PrimaryUnit {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Entity)) => Ok(PrimaryUnit::Entity(Box::new(
                super::entity::EntityDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Configuration)) => {
                Ok(PrimaryUnit::Configuration(Box::new(
                    super::configuration::ConfigurationDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Context)) => Ok(PrimaryUnit::Context(Box::new(
                super::context::ContextDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // Check for PACKAGE id IS NEW (instantiation)
                let save = parser.save();
                parser.consume(); // PACKAGE
                parser.consume(); // identifier
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                parser.restore(save);
                if is_instantiation {
                    Ok(PrimaryUnit::PackageInstantiation(Box::new(
                        super::package::PackageInstantiationDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(PrimaryUnit::Package(Box::new(
                        super::package::PackageDeclaration::parse(parser)?,
                    )))
                }
            }
            _ => {
                Err(parser
                    .error("expected primary unit (entity, configuration, package, or context)"))
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            PrimaryUnit::Entity(inner) => inner.format(f, indent_level),
            PrimaryUnit::Configuration(inner) => inner.format(f, indent_level),
            PrimaryUnit::Package(inner) => inner.format(f, indent_level),
            PrimaryUnit::PackageInstantiation(inner) => inner.format(f, indent_level),
            PrimaryUnit::Context(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for SecondaryUnit {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Architecture)) => Ok(SecondaryUnit::Architecture(
                Box::new(super::architecture::ArchitectureBody::parse(parser)?),
            )),
            Some(TokenKind::Keyword(KeywordKind::Package)) => Ok(SecondaryUnit::PackageBody(
                Box::new(super::package::PackageBody::parse(parser)?),
            )),
            _ => Err(parser.error("expected secondary unit (architecture or package body)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            SecondaryUnit::Architecture(inner) => inner.format(f, indent_level),
            SecondaryUnit::PackageBody(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for Declaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Type)) => Ok(Declaration::Type(Box::new(
                super::type_def::TypeDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => Ok(Declaration::Subtype(Box::new(
                super::type_def::SubtypeDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Constant))
            | Some(TokenKind::Keyword(KeywordKind::Signal))
            | Some(TokenKind::Keyword(KeywordKind::Variable))
            | Some(TokenKind::Keyword(KeywordKind::Shared))
            | Some(TokenKind::Keyword(KeywordKind::File)) => Ok(Declaration::Object(Box::new(
                super::object_decl::ObjectDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Alias)) => Ok(Declaration::Alias(Box::new(
                super::object_decl::AliasDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => Ok(Declaration::Attribute(
                Box::new(super::attribute::AttributeDeclaration::parse(parser)?),
            )),
            Some(TokenKind::Keyword(KeywordKind::Component)) => Ok(Declaration::Component(
                Box::new(super::component::ComponentDeclaration::parse(parser)?),
            )),
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                // Disambiguate: GROUP identifier IS ... (template)
                //            vs GROUP identifier : ... (group declaration)
                let save = parser.save();
                parser.consume(); // GROUP
                parser.consume(); // identifier
                let is_template = parser.at_keyword(KeywordKind::Is);
                parser.restore(save);
                if is_template {
                    Ok(Declaration::GroupTemplate(Box::new(
                        super::group::GroupTemplateDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(Declaration::Group(Box::new(
                        super::group::GroupDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Entity)) => Ok(Declaration::Entity(Box::new(
                super::entity::EntityDeclaration::parse(parser)?,
            ))),
            Some(TokenKind::Keyword(KeywordKind::Configuration)) => {
                Ok(Declaration::Configuration(Box::new(
                    super::configuration::ConfigurationDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Function))
            | Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => Ok(Declaration::Subprogram(
                Box::new(super::subprogram::SubprogramDeclaration::parse(parser)?),
            )),
            Some(TokenKind::Keyword(KeywordKind::Package)) => Ok(Declaration::Package(Box::new(
                super::package::PackageDeclaration::parse(parser)?,
            ))),
            _ => Err(parser.error("expected declaration")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Declaration::Type(inner) => inner.format(f, indent_level),
            Declaration::Subtype(inner) => inner.format(f, indent_level),
            Declaration::Object(inner) => inner.format(f, indent_level),
            Declaration::Interface(inner) => inner.format(f, indent_level),
            Declaration::Alias(inner) => inner.format(f, indent_level),
            Declaration::Attribute(inner) => inner.format(f, indent_level),
            Declaration::Component(inner) => inner.format(f, indent_level),
            Declaration::GroupTemplate(inner) => inner.format(f, indent_level),
            Declaration::Group(inner) => inner.format(f, indent_level),
            Declaration::Entity(inner) => inner.format(f, indent_level),
            Declaration::Configuration(inner) => inner.format(f, indent_level),
            Declaration::Subprogram(inner) => inner.format(f, indent_level),
            Declaration::Package(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ToolDirective {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Tool directives start with backtick ` — the lexer may not produce
        // a backtick token, so this may not appear in the token stream.
        // For now we parse a minimal form: just return an error since the
        // lexer does not produce backtick tokens.
        Err(parser.error("tool directives are not supported in the current lexer"))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "`")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " {}", self.content)
    }
}
