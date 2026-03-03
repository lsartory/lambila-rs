//! Package declaration and body AST nodes.

use super::common::*;
use super::interface::GenericMapAspect;
use super::name::Name;
use super::node::{AstNode, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF (VHDL-2008): `package_declaration ::= PACKAGE identifier IS package_header
///     package_declarative_part END [ PACKAGE ] [ package_simple_name ] ;`
/// EBNF (VHDL-87/93): `package_declaration ::= PACKAGE identifier IS
///     package_declarative_part END [ PACKAGE ] [ package_simple_name ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDeclaration {
    pub identifier: Identifier,
    /// VHDL-2008.
    pub header: Option<PackageHeader>,
    pub declarative_part: PackageDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `package_header ::= [ generic_clause [ generic_map_aspect ; ] ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageHeader {
    pub generic_clause: Option<super::interface::GenericClause>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

/// EBNF: `package_declarative_part ::= { package_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDeclarativePart {
    pub items: Vec<PackageDeclarativeItem>,
}

/// EBNF (VHDL-2008): `package_declarative_item ::= subprogram_declaration
///     | subprogram_instantiation_declaration | package_declaration
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | signal_declaration | variable_declaration | file_declaration
///     | alias_declaration | component_declaration | attribute_declaration
///     | attribute_specification | disconnection_specification | use_clause
///     | group_template_declaration | group_declaration
///     | PSL_Property_Declaration | PSL_Sequence_Declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<PackageDeclaration>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    SignalDeclaration(Box<super::object_decl::SignalDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    ComponentDeclaration(Box<super::component::ComponentDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    DisconnectionSpecification(Box<super::signal::DisconnectionSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF (VHDL-2008): `package_body ::= PACKAGE BODY package_simple_name IS
///     package_body_declarative_part END [ PACKAGE BODY ] [ package_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ package_simple_name ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageBody {
    pub name: SimpleName,
    pub declarative_part: PackageBodyDeclarativePart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `package_body_declarative_part ::= { package_body_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageBodyDeclarativePart {
    pub items: Vec<PackageBodyDeclarativeItem>,
}

/// EBNF (VHDL-2008): `package_body_declarative_item ::= subprogram_declaration
///     | subprogram_body | subprogram_instantiation_declaration | package_declaration
///     | package_body | package_instantiation_declaration | type_declaration
///     | subtype_declaration | constant_declaration | variable_declaration | file_declaration
///     | alias_declaration | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageBodyDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramBody(Box<super::subprogram::SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    /// VHDL-2008.
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    /// VHDL-2008.
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `package_instantiation_declaration ::= PACKAGE identifier IS NEW
///     uninstantiated_package_name [ generic_map_aspect ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInstantiationDeclaration {
    pub identifier: Identifier,
    pub package_name: Box<Name>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for PackageDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // PACKAGE identifier IS package_header package_declarative_part
        //     END [ PACKAGE ] [ package_simple_name ] ;
        parser.expect_keyword(KeywordKind::Package)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        // Parse optional package header (VHDL-2008: starts with GENERIC)
        let header = if parser.at_keyword(KeywordKind::Generic) {
            Some(PackageHeader::parse(parser)?)
        } else {
            None
        };
        let declarative_part = PackageDeclarativePart::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        // Optional PACKAGE keyword
        parser.consume_if_keyword(KeywordKind::Package);
        // Optional end name
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(PackageDeclaration {
            identifier,
            header,
            declarative_part,
            end_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package ")?;
        self.identifier.format(f, indent_level)?;
        writeln!(f, " is")?;
        if let Some(header) = &self.header {
            header.format(f, indent_level + 1)?;
        }
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end package")?;
        if let Some(end_name) = &self.end_name {
            write!(f, " ")?;
            end_name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for PackageHeader {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ GENERIC ( generic_list ) ; [ generic_map_aspect ; ] ]
        // This is called only when we already know GENERIC is present.
        let generic_clause = Some(super::interface::GenericClause::parse(parser)?);
        // Optional generic_map_aspect followed by `;`
        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            // Peek to see if this is GENERIC MAP (not a new generic clause)
            if let Some(next) = parser.peek_nth(1) {
                if next.kind == TokenKind::Keyword(KeywordKind::Map) {
                    let gma = GenericMapAspect::parse(parser)?;
                    parser.expect(TokenKind::Semicolon)?;
                    Some(gma)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        Ok(PackageHeader {
            generic_clause,
            generic_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(generic_clause) = &self.generic_clause {
            generic_clause.format(f, indent_level)?;
        }
        if let Some(generic_map_aspect) = &self.generic_map_aspect {
            generic_map_aspect.format(f, indent_level)?;
            writeln!(f, ";")?;
        }
        Ok(())
    }
}

impl AstNode for PackageDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { package_declarative_item }
        // Parse items until we see END keyword (or EOF)
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            items.push(PackageDeclarativeItem::parse(parser)?);
        }
        Ok(PackageDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for PackageDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                Ok(PackageDeclarativeItem::TypeDeclaration(Box::new(
                    super::type_def::TypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                Ok(PackageDeclarativeItem::SubtypeDeclaration(Box::new(
                    super::type_def::SubtypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                Ok(PackageDeclarativeItem::ConstantDeclaration(Box::new(
                    super::object_decl::ConstantDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Signal)) => {
                Ok(PackageDeclarativeItem::SignalDeclaration(Box::new(
                    super::object_decl::SignalDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                // SHARED VARIABLE
                Ok(PackageDeclarativeItem::SharedVariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Variable)) => {
                Ok(PackageDeclarativeItem::VariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                Ok(PackageDeclarativeItem::FileDeclaration(Box::new(
                    super::object_decl::FileDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                Ok(PackageDeclarativeItem::AliasDeclaration(Box::new(
                    super::object_decl::AliasDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Component)) => {
                Ok(PackageDeclarativeItem::ComponentDeclaration(Box::new(
                    super::component::ComponentDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                // Disambiguate: ATTRIBUTE identifier : ... (declaration)
                //            vs ATTRIBUTE identifier OF ... (specification)
                // Use backtracking: save, consume ATTRIBUTE, consume identifier, check next.
                let save = parser.save();
                parser.consume(); // ATTRIBUTE
                parser.consume(); // identifier (or designator)
                let is_declaration = parser.at(TokenKind::Colon);
                parser.restore(save);
                if is_declaration {
                    Ok(PackageDeclarativeItem::AttributeDeclaration(Box::new(
                        super::attribute::AttributeDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(PackageDeclarativeItem::AttributeSpecification(Box::new(
                        super::attribute::AttributeSpecification::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Disconnect)) => {
                Ok(PackageDeclarativeItem::DisconnectionSpecification(
                    Box::new(super::signal::DisconnectionSpecification::parse(parser)?),
                ))
            }
            Some(TokenKind::Keyword(KeywordKind::Use)) => Ok(PackageDeclarativeItem::UseClause(
                super::context::UseClause::parse(parser)?,
            )),
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                // Disambiguate: GROUP identifier IS ... (template declaration)
                //            vs GROUP identifier : ... (group declaration)
                let save = parser.save();
                parser.consume(); // GROUP
                parser.consume(); // identifier
                let is_template = parser.at_keyword(KeywordKind::Is);
                parser.restore(save);
                if is_template {
                    Ok(PackageDeclarativeItem::GroupTemplateDeclaration(Box::new(
                        super::group::GroupTemplateDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(PackageDeclarativeItem::GroupDeclaration(Box::new(
                        super::group::GroupDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Function)) => {
                // Disambiguate: subprogram_instantiation_declaration vs subprogram_declaration
                // Instantiation: PROCEDURE/FUNCTION identifier IS NEW ...
                // Declaration: PROCEDURE/FUNCTION designator ... ;
                let save = parser.save();
                parser.consume(); // PROCEDURE or FUNCTION
                // Skip the designator (identifier or operator_symbol)
                if parser.at(TokenKind::StringLiteral) {
                    parser.consume(); // operator symbol
                } else {
                    parser.consume(); // identifier
                }
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                parser.restore(save);
                if is_instantiation {
                    Ok(PackageDeclarativeItem::SubprogramInstantiationDeclaration(
                        Box::new(
                            super::subprogram::SubprogramInstantiationDeclaration::parse(parser)?,
                        ),
                    ))
                } else {
                    Ok(PackageDeclarativeItem::SubprogramDeclaration(Box::new(
                        super::subprogram::SubprogramDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                // PURE/IMPURE FUNCTION ... -> subprogram declaration
                Ok(PackageDeclarativeItem::SubprogramDeclaration(Box::new(
                    super::subprogram::SubprogramDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // Disambiguate: PACKAGE identifier IS NEW ... (instantiation)
                //            vs PACKAGE identifier IS ... (nested package declaration)
                let save = parser.save();
                parser.consume(); // PACKAGE
                parser.consume(); // identifier
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                parser.restore(save);
                if is_instantiation {
                    Ok(PackageDeclarativeItem::PackageInstantiationDeclaration(
                        Box::new(PackageInstantiationDeclaration::parse(parser)?),
                    ))
                } else {
                    Ok(PackageDeclarativeItem::PackageDeclaration(Box::new(
                        PackageDeclaration::parse(parser)?,
                    )))
                }
            }
            _ => Err(parser.error("expected package declarative item")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::SubprogramDeclaration(inner) => inner.format(f, indent_level),
            Self::SubprogramInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageDeclaration(inner) => inner.format(f, indent_level),
            Self::PackageInstantiationDeclaration(inner) => inner.format(f, indent_level),
            Self::TypeDeclaration(inner) => inner.format(f, indent_level),
            Self::SubtypeDeclaration(inner) => inner.format(f, indent_level),
            Self::ConstantDeclaration(inner) => inner.format(f, indent_level),
            Self::SignalDeclaration(inner) => inner.format(f, indent_level),
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
            Self::VariableDeclaration(inner) => inner.format(f, indent_level),
            Self::FileDeclaration(inner) => inner.format(f, indent_level),
            Self::AliasDeclaration(inner) => inner.format(f, indent_level),
            Self::ComponentDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::DisconnectionSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for PackageBody {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // PACKAGE BODY package_simple_name IS
        //     package_body_declarative_part
        // END [ PACKAGE BODY ] [ package_simple_name ] ;
        parser.expect_keyword(KeywordKind::Package)?;
        parser.expect_keyword(KeywordKind::Body)?;
        let name = SimpleName::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let declarative_part = PackageBodyDeclarativePart::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        // Optional PACKAGE BODY
        if parser.consume_if_keyword(KeywordKind::Package).is_some() {
            parser.expect_keyword(KeywordKind::Body)?;
        }
        // Optional end name
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(PackageBody {
            name,
            declarative_part,
            end_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package body ")?;
        self.name.format(f, indent_level)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end package body")?;
        if let Some(end_name) = &self.end_name {
            write!(f, " ")?;
            end_name.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for PackageBodyDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { package_body_declarative_item }
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            items.push(PackageBodyDeclarativeItem::parse(parser)?);
        }
        Ok(PackageBodyDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for PackageBodyDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                Ok(PackageBodyDeclarativeItem::TypeDeclaration(Box::new(
                    super::type_def::TypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                Ok(PackageBodyDeclarativeItem::SubtypeDeclaration(Box::new(
                    super::type_def::SubtypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                Ok(PackageBodyDeclarativeItem::ConstantDeclaration(Box::new(
                    super::object_decl::ConstantDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                // SHARED VARIABLE
                Ok(PackageBodyDeclarativeItem::SharedVariableDeclaration(
                    Box::new(super::object_decl::VariableDeclaration::parse(parser)?),
                ))
            }
            Some(TokenKind::Keyword(KeywordKind::Variable)) => {
                Ok(PackageBodyDeclarativeItem::VariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                Ok(PackageBodyDeclarativeItem::FileDeclaration(Box::new(
                    super::object_decl::FileDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                Ok(PackageBodyDeclarativeItem::AliasDeclaration(Box::new(
                    super::object_decl::AliasDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                // Disambiguate: declaration (`:` after identifier) vs specification (`OF` after identifier)
                let save = parser.save();
                parser.consume(); // ATTRIBUTE
                parser.consume(); // identifier
                let is_declaration = parser.at(TokenKind::Colon);
                parser.restore(save);
                if is_declaration {
                    Ok(PackageBodyDeclarativeItem::AttributeDeclaration(Box::new(
                        super::attribute::AttributeDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(PackageBodyDeclarativeItem::AttributeSpecification(
                        Box::new(super::attribute::AttributeSpecification::parse(parser)?),
                    ))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Use)) => Ok(
                PackageBodyDeclarativeItem::UseClause(super::context::UseClause::parse(parser)?),
            ),
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                // Disambiguate: GROUP identifier IS ... (template) vs GROUP identifier : ... (declaration)
                let save = parser.save();
                parser.consume(); // GROUP
                parser.consume(); // identifier
                let is_template = parser.at_keyword(KeywordKind::Is);
                parser.restore(save);
                if is_template {
                    Ok(PackageBodyDeclarativeItem::GroupTemplateDeclaration(
                        Box::new(super::group::GroupTemplateDeclaration::parse(parser)?),
                    ))
                } else {
                    Ok(PackageBodyDeclarativeItem::GroupDeclaration(Box::new(
                        super::group::GroupDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Function)) => {
                // Disambiguate between:
                // 1. SubprogramInstantiationDeclaration: PROCEDURE/FUNCTION identifier IS NEW ...
                // 2. SubprogramBody: PROCEDURE/FUNCTION designator ... IS ... BEGIN ... END ;
                // 3. SubprogramDeclaration: PROCEDURE/FUNCTION designator ... ;
                //
                // Strategy: save, skip to find IS NEW (instantiation), or IS followed by non-NEW
                // (body), or ; before IS (declaration).
                let save = parser.save();
                parser.consume(); // PROCEDURE or FUNCTION
                // Skip the designator
                if parser.at(TokenKind::StringLiteral) {
                    parser.consume(); // operator symbol
                } else {
                    parser.consume(); // identifier
                }
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                parser.restore(save);

                if is_instantiation {
                    Ok(
                        PackageBodyDeclarativeItem::SubprogramInstantiationDeclaration(Box::new(
                            super::subprogram::SubprogramInstantiationDeclaration::parse(parser)?,
                        )),
                    )
                } else {
                    // Need to distinguish between declaration and body.
                    // A subprogram body has: specification IS ... BEGIN ... END ;
                    // A subprogram declaration has: specification ;
                    // We need to scan forward past the specification to find IS (body) or ; (declaration).
                    // Use a deeper lookahead: scan tokens until we find `;` or `IS` at nesting depth 0.
                    let save2 = parser.save();
                    parser.consume(); // PROCEDURE or FUNCTION
                    // Skip to find IS or ; at nesting level 0 (accounting for parens)
                    let mut depth = 0;
                    let mut found_body = false;
                    loop {
                        if parser.eof() {
                            break;
                        }
                        match parser.peek_kind() {
                            Some(TokenKind::LeftParen) => {
                                depth += 1;
                                parser.consume();
                            }
                            Some(TokenKind::RightParen) => {
                                depth -= 1;
                                parser.consume();
                            }
                            Some(TokenKind::Semicolon) if depth == 0 => {
                                // Declaration (specification ;)
                                found_body = false;
                                break;
                            }
                            Some(TokenKind::Keyword(KeywordKind::Is)) if depth == 0 => {
                                // Body (specification IS ...)
                                found_body = true;
                                break;
                            }
                            Some(TokenKind::Keyword(KeywordKind::Return)) if depth == 0 => {
                                // Part of function specification, keep scanning
                                parser.consume();
                            }
                            _ => {
                                parser.consume();
                            }
                        }
                    }
                    parser.restore(save2);
                    if found_body {
                        Ok(PackageBodyDeclarativeItem::SubprogramBody(Box::new(
                            super::subprogram::SubprogramBody::parse(parser)?,
                        )))
                    } else {
                        Ok(PackageBodyDeclarativeItem::SubprogramDeclaration(Box::new(
                            super::subprogram::SubprogramDeclaration::parse(parser)?,
                        )))
                    }
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                // PURE/IMPURE FUNCTION ... -> need to distinguish body vs declaration
                let save = parser.save();
                parser.consume(); // PURE or IMPURE
                parser.consume(); // FUNCTION
                // Skip past spec to find IS or ;
                let mut depth = 0;
                let mut found_body = false;
                loop {
                    if parser.eof() {
                        break;
                    }
                    match parser.peek_kind() {
                        Some(TokenKind::LeftParen) => {
                            depth += 1;
                            parser.consume();
                        }
                        Some(TokenKind::RightParen) => {
                            depth -= 1;
                            parser.consume();
                        }
                        Some(TokenKind::Semicolon) if depth == 0 => {
                            found_body = false;
                            break;
                        }
                        Some(TokenKind::Keyword(KeywordKind::Is)) if depth == 0 => {
                            found_body = true;
                            break;
                        }
                        _ => {
                            parser.consume();
                        }
                    }
                }
                parser.restore(save);
                if found_body {
                    Ok(PackageBodyDeclarativeItem::SubprogramBody(Box::new(
                        super::subprogram::SubprogramBody::parse(parser)?,
                    )))
                } else {
                    Ok(PackageBodyDeclarativeItem::SubprogramDeclaration(Box::new(
                        super::subprogram::SubprogramDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // Disambiguate between:
                // PACKAGE BODY ... (package body)
                // PACKAGE identifier IS NEW ... (package instantiation)
                // PACKAGE identifier IS ... (nested package declaration)
                if parser
                    .peek_nth(1)
                    .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::Body))
                {
                    Ok(PackageBodyDeclarativeItem::PackageBody(Box::new(
                        PackageBody::parse(parser)?,
                    )))
                } else {
                    let save = parser.save();
                    parser.consume(); // PACKAGE
                    parser.consume(); // identifier
                    let is_instantiation = parser.at_keyword(KeywordKind::Is)
                        && parser
                            .peek_nth(1)
                            .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                    parser.restore(save);
                    if is_instantiation {
                        Ok(PackageBodyDeclarativeItem::PackageInstantiationDeclaration(
                            Box::new(PackageInstantiationDeclaration::parse(parser)?),
                        ))
                    } else {
                        Ok(PackageBodyDeclarativeItem::PackageDeclaration(Box::new(
                            PackageDeclaration::parse(parser)?,
                        )))
                    }
                }
            }
            _ => Err(parser.error("expected package body declarative item")),
        }
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
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
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

impl AstNode for PackageInstantiationDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // PACKAGE identifier IS NEW uninstantiated_package_name [ generic_map_aspect ] ;
        parser.expect_keyword(KeywordKind::Package)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        parser.expect_keyword(KeywordKind::New)?;
        let package_name = Box::new(Name::parse(parser)?);
        // Optional generic_map_aspect
        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            Some(GenericMapAspect::parse(parser)?)
        } else {
            None
        };
        parser.expect(TokenKind::Semicolon)?;
        Ok(PackageInstantiationDeclaration {
            identifier,
            package_name,
            generic_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "package ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is new ")?;
        self.package_name.format(f, indent_level)?;
        if let Some(generic_map) = &self.generic_map_aspect {
            writeln!(f)?;
            generic_map.format(f, indent_level + 1)?;
        }
        writeln!(f, ";")
    }
}
