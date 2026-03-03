//! Entity declaration AST nodes.

use super::common::*;
use super::node::{AstNode, format_lines, write_indent};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF (VHDL-2008): `entity_declaration ::= ENTITY identifier IS entity_header
///     entity_declarative_part [ BEGIN entity_statement_part ] END [ ENTITY ]
///     [ entity_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ entity_simple_name ] ;` (no optional ENTITY keyword).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDeclaration {
    pub identifier: Identifier,
    pub header: EntityHeader,
    pub declarative_part: EntityDeclarativePart,
    pub statement_part: Option<EntityStatementPart>,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `entity_header ::= [ formal_generic_clause ] [ formal_port_clause ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityHeader {
    pub generic_clause: Option<super::interface::GenericClause>,
    pub port_clause: Option<super::interface::PortClause>,
}

/// EBNF: `entity_declarative_part ::= { entity_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDeclarativePart {
    pub items: Vec<EntityDeclarativeItem>,
}

/// EBNF (VHDL-2008): `entity_declarative_item ::= subprogram_declaration | subprogram_body
///     | subprogram_instantiation_declaration | package_declaration | package_body
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | signal_declaration | shared_variable_declaration
///     | file_declaration | alias_declaration | attribute_declaration
///     | attribute_specification | disconnection_specification | use_clause
///     | group_template_declaration | group_declaration
///     | PSL_Property_Declaration | PSL_Sequence_Declaration | PSL_Clock_Declaration`
/// Earlier versions have fewer alternatives (no package_body, no subprogram_instantiation, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityDeclarativeItem {
    SubprogramDeclaration(Box<super::subprogram::SubprogramDeclaration>),
    SubprogramBody(Box<super::subprogram::SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<super::subprogram::SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<super::package::PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<super::package::PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    SignalDeclaration(Box<super::object_decl::SignalDeclaration>),
    /// VHDL-93+.
    SharedVariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    DisconnectionSpecification(Box<super::signal::DisconnectionSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `entity_statement_part ::= { entity_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityStatementPart {
    pub statements: Vec<EntityStatement>,
}

/// EBNF (VHDL-2008): `entity_statement ::= concurrent_assertion_statement
///     | passive_concurrent_procedure_call_statement | passive_process_statement
///     | PSL_PSL_Directive`
/// EBNF (VHDL-87/93): omits PSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityStatement {
    ConcurrentAssertion(Box<super::concurrent::ConcurrentAssertionStatement>),
    PassiveProcedureCall(Box<super::concurrent::ConcurrentProcedureCallStatement>),
    PassiveProcess(Box<super::concurrent::ProcessStatement>),
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for EntityDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // ENTITY identifier IS entity_header entity_declarative_part
        //     [ BEGIN entity_statement_part ] END [ ENTITY ] [ entity_simple_name ] ;
        parser.expect_keyword(KeywordKind::Entity)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let header = EntityHeader::parse(parser)?;
        let declarative_part = EntityDeclarativePart::parse(parser)?;
        let statement_part = if parser.consume_if_keyword(KeywordKind::Begin).is_some() {
            Some(EntityStatementPart::parse(parser)?)
        } else {
            None
        };
        parser.expect_keyword(KeywordKind::End)?;
        parser.consume_if_keyword(KeywordKind::Entity);
        let end_name =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(SimpleName::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(EntityDeclaration {
            identifier,
            header,
            declarative_part,
            statement_part,
            end_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "entity ")?;
        self.identifier.format(f, 0)?;
        writeln!(f, " is")?;
        self.header.format(f, indent_level + 1)?;
        self.declarative_part.format(f, indent_level + 1)?;
        if let Some(ref stmt_part) = self.statement_part {
            write_indent(f, indent_level)?;
            writeln!(f, "begin")?;
            stmt_part.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end entity")?;
        if let Some(ref name) = self.end_name {
            write!(f, " ")?;
            name.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for EntityHeader {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ formal_generic_clause ] [ formal_port_clause ]
        let generic_clause = if parser.at_keyword(KeywordKind::Generic) {
            Some(super::interface::GenericClause::parse(parser)?)
        } else {
            None
        };
        let port_clause = if parser.at_keyword(KeywordKind::Port) {
            Some(super::interface::PortClause::parse(parser)?)
        } else {
            None
        };
        Ok(EntityHeader {
            generic_clause,
            port_clause,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(ref gc) = self.generic_clause {
            gc.format(f, indent_level)?;
        }
        if let Some(ref pc) = self.port_clause {
            pc.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for EntityDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { entity_declarative_item } — parse until BEGIN or END
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::Begin)
            && !parser.at_keyword(KeywordKind::End)
            && !parser.eof()
        {
            items.push(EntityDeclarativeItem::parse(parser)?);
        }
        Ok(EntityDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for EntityDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                Ok(EntityDeclarativeItem::TypeDeclaration(Box::new(
                    super::type_def::TypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                Ok(EntityDeclarativeItem::SubtypeDeclaration(Box::new(
                    super::type_def::SubtypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                Ok(EntityDeclarativeItem::ConstantDeclaration(Box::new(
                    super::object_decl::ConstantDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Signal)) => {
                Ok(EntityDeclarativeItem::SignalDeclaration(Box::new(
                    super::object_decl::SignalDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                // SHARED VARIABLE
                Ok(EntityDeclarativeItem::SharedVariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                Ok(EntityDeclarativeItem::FileDeclaration(Box::new(
                    super::object_decl::FileDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                Ok(EntityDeclarativeItem::AliasDeclaration(Box::new(
                    super::object_decl::AliasDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                // Disambiguate: ATTRIBUTE identifier : ... (declaration)
                //            vs ATTRIBUTE identifier OF ... (specification)
                let save = parser.save();
                parser.consume(); // ATTRIBUTE
                parser.consume(); // identifier
                let is_declaration = parser.at(TokenKind::Colon);
                parser.restore(save);
                if is_declaration {
                    Ok(EntityDeclarativeItem::AttributeDeclaration(Box::new(
                        super::attribute::AttributeDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(EntityDeclarativeItem::AttributeSpecification(Box::new(
                        super::attribute::AttributeSpecification::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Disconnect)) => {
                Ok(EntityDeclarativeItem::DisconnectionSpecification(Box::new(
                    super::signal::DisconnectionSpecification::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Use)) => Ok(EntityDeclarativeItem::UseClause(
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
                    Ok(EntityDeclarativeItem::GroupTemplateDeclaration(Box::new(
                        super::group::GroupTemplateDeclaration::parse(parser)?,
                    )))
                } else {
                    Ok(EntityDeclarativeItem::GroupDeclaration(Box::new(
                        super::group::GroupDeclaration::parse(parser)?,
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Function)) => {
                // Disambiguate: subprogram_instantiation (PROCEDURE/FUNCTION id IS NEW ...)
                //            vs subprogram_declaration/body
                let save = parser.save();
                parser.consume(); // PROCEDURE or FUNCTION
                // Skip the designator (identifier or operator_symbol)
                parser.consume();
                let is_instantiation = parser.at_keyword(KeywordKind::Is)
                    && parser
                        .peek_nth(1)
                        .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                // For body vs declaration: subprogram_spec IS ... (not NEW) => body
                // subprogram_spec ; => declaration
                // We need a different approach: parse the spec, then check for IS vs ;
                parser.restore(save);
                if is_instantiation {
                    Ok(EntityDeclarativeItem::SubprogramInstantiationDeclaration(
                        Box::new(
                            super::subprogram::SubprogramInstantiationDeclaration::parse(parser)?,
                        ),
                    ))
                } else {
                    // Try body first: spec IS ... BEGIN ... END
                    // Declaration is: spec ;
                    // Use backtracking: parse spec, check for IS
                    let save2 = parser.save();
                    match super::subprogram::SubprogramBody::parse(parser) {
                        Ok(body) => Ok(EntityDeclarativeItem::SubprogramBody(Box::new(body))),
                        Err(_) => {
                            parser.restore(save2);
                            Ok(EntityDeclarativeItem::SubprogramDeclaration(Box::new(
                                super::subprogram::SubprogramDeclaration::parse(parser)?,
                            )))
                        }
                    }
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                // PURE/IMPURE FUNCTION ... -> subprogram body or declaration
                let save = parser.save();
                match super::subprogram::SubprogramBody::parse(parser) {
                    Ok(body) => Ok(EntityDeclarativeItem::SubprogramBody(Box::new(body))),
                    Err(_) => {
                        parser.restore(save);
                        Ok(EntityDeclarativeItem::SubprogramDeclaration(Box::new(
                            super::subprogram::SubprogramDeclaration::parse(parser)?,
                        )))
                    }
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // Disambiguate: PACKAGE BODY ... | PACKAGE id IS NEW ... | PACKAGE id IS ...
                let save = parser.save();
                parser.consume(); // PACKAGE
                if parser.at_keyword(KeywordKind::Body) {
                    parser.restore(save);
                    Ok(EntityDeclarativeItem::PackageBody(Box::new(
                        super::package::PackageBody::parse(parser)?,
                    )))
                } else {
                    parser.consume(); // identifier
                    let is_instantiation = parser.at_keyword(KeywordKind::Is)
                        && parser
                            .peek_nth(1)
                            .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::New));
                    parser.restore(save);
                    if is_instantiation {
                        Ok(EntityDeclarativeItem::PackageInstantiationDeclaration(
                            Box::new(super::package::PackageInstantiationDeclaration::parse(
                                parser,
                            )?),
                        ))
                    } else {
                        Ok(EntityDeclarativeItem::PackageDeclaration(Box::new(
                            super::package::PackageDeclaration::parse(parser)?,
                        )))
                    }
                }
            }
            _ => Err(parser.error("expected entity declarative item")),
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
            Self::SignalDeclaration(inner) => inner.format(f, indent_level),
            Self::SharedVariableDeclaration(inner) => inner.format(f, indent_level),
            Self::FileDeclaration(inner) => inner.format(f, indent_level),
            Self::AliasDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::DisconnectionSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for EntityStatementPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { entity_statement } — parse until END
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(EntityStatement::parse(parser)?);
        }
        Ok(EntityStatementPart { statements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for EntityStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // entity_statement ::= concurrent_assertion_statement
        //     | passive_concurrent_procedure_call_statement
        //     | passive_process_statement
        //
        // ASSERT / POSTPONED ASSERT -> ConcurrentAssertionStatement
        // PROCESS / POSTPONED PROCESS -> ProcessStatement
        // label : ASSERT / POSTPONED ... -> labeled version
        // name ( args ) ; -> procedure call

        // Check for ASSERT keyword directly
        if parser.at_keyword(KeywordKind::Assert) {
            let stmt = super::concurrent::ConcurrentAssertionStatement::parse(parser)?;
            return Ok(EntityStatement::ConcurrentAssertion(Box::new(stmt)));
        }

        // Check for POSTPONED
        if parser.at_keyword(KeywordKind::Postponed)
            && let Some(next) = parser.peek_nth(1)
        {
            if next.kind == TokenKind::Keyword(KeywordKind::Assert) {
                let stmt = super::concurrent::ConcurrentAssertionStatement::parse(parser)?;
                return Ok(EntityStatement::ConcurrentAssertion(Box::new(stmt)));
            }
            if next.kind == TokenKind::Keyword(KeywordKind::Process) {
                let stmt = super::concurrent::ProcessStatement::parse(parser)?;
                return Ok(EntityStatement::PassiveProcess(Box::new(stmt)));
            }
        }

        // Check for PROCESS keyword
        if parser.at_keyword(KeywordKind::Process) {
            let stmt = super::concurrent::ProcessStatement::parse(parser)?;
            return Ok(EntityStatement::PassiveProcess(Box::new(stmt)));
        }

        // Check for label : ...
        if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
            let save = parser.save();
            parser.consume(); // identifier (potential label)
            if parser.at(TokenKind::Colon) {
                // It's a label. Look at what follows the colon.
                parser.consume(); // :
                let is_assert = parser.at_keyword(KeywordKind::Assert)
                    || (parser.at_keyword(KeywordKind::Postponed)
                        && parser
                            .peek_nth(1)
                            .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::Assert)));
                let is_process = parser.at_keyword(KeywordKind::Process)
                    || (parser.at_keyword(KeywordKind::Postponed)
                        && parser
                            .peek_nth(1)
                            .is_some_and(|t| t.kind == TokenKind::Keyword(KeywordKind::Process)));
                parser.restore(save);
                if is_assert {
                    let stmt = super::concurrent::ConcurrentAssertionStatement::parse(parser)?;
                    return Ok(EntityStatement::ConcurrentAssertion(Box::new(stmt)));
                } else if is_process {
                    let stmt = super::concurrent::ProcessStatement::parse(parser)?;
                    return Ok(EntityStatement::PassiveProcess(Box::new(stmt)));
                } else {
                    // Must be a procedure call with label
                    let stmt = super::concurrent::ConcurrentProcedureCallStatement::parse(parser)?;
                    return Ok(EntityStatement::PassiveProcedureCall(Box::new(stmt)));
                }
            } else {
                // No colon — could be a procedure call (name followed by (...) ;)
                parser.restore(save);
                let stmt = super::concurrent::ConcurrentProcedureCallStatement::parse(parser)?;
                return Ok(EntityStatement::PassiveProcedureCall(Box::new(stmt)));
            }
        }

        Err(parser.error("expected entity statement (assertion, procedure call, or process)"))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::ConcurrentAssertion(inner) => inner.format(f, indent_level),
            Self::PassiveProcedureCall(inner) => inner.format(f, indent_level),
            Self::PassiveProcess(inner) => inner.format(f, indent_level),
        }
    }
}
