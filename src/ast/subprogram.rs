//! Subprogram declaration and body AST nodes.

use super::common::*;
use super::interface::GenericMapAspect;
use super::name::Name;
use super::node::{AstNode, format_lines, write_indent};
use super::type_def::TypeMark;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `subprogram_declaration ::= subprogram_specification ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramDeclaration {
    pub specification: SubprogramSpecification,
}

/// EBNF (VHDL-2008): `subprogram_body ::= subprogram_specification IS
///     subprogram_declarative_part BEGIN subprogram_statement_part
///     END [ subprogram_kind ] [ designator ] ;`
/// EBNF (VHDL-87): `...END [ designator ] ;` (no optional subprogram_kind).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramBody {
    pub specification: SubprogramSpecification,
    pub declarative_part: SubprogramDeclarativePart,
    pub statement_part: SubprogramStatementPart,
    pub end_kind: Option<SubprogramKind>,
    pub end_designator: Option<Designator>,
}

/// EBNF (VHDL-2008): `subprogram_specification ::= procedure_specification
///     | function_specification`
/// EBNF (VHDL-87/93): Combined inline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubprogramSpecification {
    Procedure(ProcedureSpecification),
    Function(FunctionSpecification),
}

/// EBNF (VHDL-2008): `procedure_specification ::= PROCEDURE designator subprogram_header
///     [ [ PARAMETER ] ( formal_parameter_list ) ]`
/// EBNF (VHDL-87/93): `PROCEDURE designator [ ( formal_parameter_list ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureSpecification {
    pub designator: Designator,
    /// VHDL-2008.
    pub header: Option<SubprogramHeader>,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
}

/// EBNF (VHDL-2008): `function_specification ::= [ PURE | IMPURE ] FUNCTION designator
///     subprogram_header [ [ PARAMETER ] ( formal_parameter_list ) ] RETURN type_mark`
/// EBNF (VHDL-87): `FUNCTION designator [ ( formal_parameter_list ) ] RETURN type_mark`
/// EBNF (VHDL-93): `[ PURE | IMPURE ] FUNCTION designator
///     [ ( formal_parameter_list ) ] RETURN type_mark`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpecification {
    /// VHDL-93+.
    pub purity: Option<super::interface::Purity>,
    pub designator: Designator,
    /// VHDL-2008.
    pub header: Option<SubprogramHeader>,
    pub has_parameter_keyword: bool,
    pub parameter_list: Option<super::association::FormalParameterList>,
    pub return_type: TypeMark,
}

/// EBNF: `subprogram_header ::= [ GENERIC ( generic_list ) [ generic_map_aspect ] ]`
/// (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramHeader {
    pub generic_list: super::interface::GenericList,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

/// EBNF: `subprogram_kind ::= PROCEDURE | FUNCTION`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubprogramKind {
    Procedure,
    Function,
}

/// EBNF: `subprogram_declarative_part ::= { subprogram_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramDeclarativePart {
    pub items: Vec<SubprogramDeclarativeItem>,
}

/// EBNF (VHDL-2008): `subprogram_declarative_item ::= subprogram_declaration
///     | subprogram_body | subprogram_instantiation_declaration | package_declaration
///     | package_body | package_instantiation_declaration | type_declaration
///     | subtype_declaration | constant_declaration | variable_declaration | file_declaration
///     | alias_declaration | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubprogramDeclarativeItem {
    SubprogramDeclaration(Box<SubprogramDeclaration>),
    SubprogramBody(Box<SubprogramBody>),
    /// VHDL-2008.
    SubprogramInstantiationDeclaration(Box<SubprogramInstantiationDeclaration>),
    /// VHDL-2008.
    PackageDeclaration(Box<super::package::PackageDeclaration>),
    /// VHDL-2008.
    PackageBody(Box<super::package::PackageBody>),
    /// VHDL-2008.
    PackageInstantiationDeclaration(Box<super::package::PackageInstantiationDeclaration>),
    TypeDeclaration(Box<super::type_def::TypeDeclaration>),
    SubtypeDeclaration(Box<super::type_def::SubtypeDeclaration>),
    ConstantDeclaration(Box<super::object_decl::ConstantDeclaration>),
    VariableDeclaration(Box<super::object_decl::VariableDeclaration>),
    FileDeclaration(Box<super::object_decl::FileDeclaration>),
    AliasDeclaration(Box<super::object_decl::AliasDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `subprogram_statement_part ::= { sequential_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramStatementPart {
    pub statements: Vec<super::sequential::SequentialStatement>,
}

/// EBNF: `subprogram_instantiation_declaration ::= subprogram_kind identifier IS NEW
///     uninstantiated_subprogram_name [ signature ] [ generic_map_aspect ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprogramInstantiationDeclaration {
    pub kind: SubprogramKind,
    pub identifier: Identifier,
    pub subprogram_name: Box<Name>,
    pub signature: Option<Signature>,
    pub generic_map_aspect: Option<GenericMapAspect>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for SubprogramDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let specification = SubprogramSpecification::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(SubprogramDeclaration { specification })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.specification.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for SubprogramBody {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let specification = SubprogramSpecification::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        let declarative_part = SubprogramDeclarativePart::parse(parser)?;
        parser.expect_keyword(KeywordKind::Begin)?;
        let statement_part = SubprogramStatementPart::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;

        // Optional subprogram_kind (PROCEDURE | FUNCTION)
        let end_kind = if parser.at_keyword(KeywordKind::Procedure)
            || parser.at_keyword(KeywordKind::Function)
        {
            Some(SubprogramKind::parse(parser)?)
        } else {
            None
        };

        // Optional designator
        let end_designator = match parser.peek_kind() {
            Some(TokenKind::Identifier)
            | Some(TokenKind::ExtendedIdentifier)
            | Some(TokenKind::StringLiteral) => Some(Designator::parse(parser)?),
            _ => None,
        };

        parser.expect(TokenKind::Semicolon)?;
        Ok(SubprogramBody {
            specification,
            declarative_part,
            statement_part,
            end_kind,
            end_designator,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.specification.format(f, indent_level)?;
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "begin")?;
        self.statement_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end")?;
        if let Some(kind) = &self.end_kind {
            write!(f, " ")?;
            kind.format(f, indent_level)?;
        }
        if let Some(desig) = &self.end_designator {
            write!(f, " ")?;
            desig.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SubprogramSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Procedure)) => Ok(
                SubprogramSpecification::Procedure(ProcedureSpecification::parse(parser)?),
            ),
            Some(TokenKind::Keyword(KeywordKind::Function))
            | Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => Ok(
                SubprogramSpecification::Function(FunctionSpecification::parse(parser)?),
            ),
            _ => Err(parser.error("expected subprogram specification (procedure or function)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            SubprogramSpecification::Procedure(spec) => spec.format(f, indent_level),
            SubprogramSpecification::Function(spec) => spec.format(f, indent_level),
        }
    }
}

impl AstNode for ProcedureSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Procedure)?;
        let designator = Designator::parse(parser)?;

        // Optional subprogram_header (VHDL-2008): GENERIC ( generic_list ) [ generic_map_aspect ]
        let header = if parser.at_keyword(KeywordKind::Generic) {
            Some(SubprogramHeader::parse(parser)?)
        } else {
            None
        };

        // Optional [ PARAMETER ] ( formal_parameter_list )
        let has_parameter_keyword = parser.consume_if_keyword(KeywordKind::Parameter).is_some();
        let parameter_list = if parser.at(TokenKind::LeftParen) {
            parser.expect(TokenKind::LeftParen)?;
            let list = super::association::FormalParameterList::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Some(list)
        } else {
            None
        };

        Ok(ProcedureSpecification {
            designator,
            header,
            has_parameter_keyword,
            parameter_list,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "procedure ")?;
        self.designator.format(f, indent_level)?;
        if let Some(header) = &self.header {
            write!(f, " ")?;
            header.format(f, indent_level)?;
        }
        if self.has_parameter_keyword {
            write!(f, " parameter")?;
        }
        if let Some(params) = &self.parameter_list {
            if params.elements.len() <= 1 {
                write!(f, " (")?;
                params.format(f, 0)?;
                write!(f, ")")?;
            } else {
                writeln!(f, " (")?;
                params.format(f, indent_level + 1)?;
                writeln!(f)?;
                write_indent(f, indent_level)?;
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl AstNode for FunctionSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Optional PURE | IMPURE
        let purity =
            if parser.at_keyword(KeywordKind::Pure) || parser.at_keyword(KeywordKind::Impure) {
                Some(super::interface::Purity::parse(parser)?)
            } else {
                None
            };

        parser.expect_keyword(KeywordKind::Function)?;
        let designator = Designator::parse(parser)?;

        // Optional subprogram_header (VHDL-2008): GENERIC ( generic_list ) [ generic_map_aspect ]
        let header = if parser.at_keyword(KeywordKind::Generic) {
            Some(SubprogramHeader::parse(parser)?)
        } else {
            None
        };

        // Optional [ PARAMETER ] ( formal_parameter_list )
        let has_parameter_keyword = parser.consume_if_keyword(KeywordKind::Parameter).is_some();
        let parameter_list = if parser.at(TokenKind::LeftParen) {
            parser.expect(TokenKind::LeftParen)?;
            let list = super::association::FormalParameterList::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Some(list)
        } else {
            None
        };

        parser.expect_keyword(KeywordKind::Return)?;
        let return_type = TypeMark::parse(parser)?;

        Ok(FunctionSpecification {
            purity,
            designator,
            header,
            has_parameter_keyword,
            parameter_list,
            return_type,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(purity) = &self.purity {
            purity.format(f, indent_level)?;
            write!(f, " ")?;
        }
        write!(f, "function ")?;
        self.designator.format(f, indent_level)?;
        if let Some(header) = &self.header {
            write!(f, " ")?;
            header.format(f, indent_level)?;
        }
        if self.has_parameter_keyword {
            write!(f, " parameter")?;
        }
        if let Some(params) = &self.parameter_list {
            if params.elements.len() <= 1 {
                write!(f, " (")?;
                params.format(f, 0)?;
                write!(f, ")")?;
            } else {
                writeln!(f, " (")?;
                params.format(f, indent_level + 1)?;
                writeln!(f)?;
                write_indent(f, indent_level)?;
                write!(f, ")")?;
            }
        }
        write!(f, " return ")?;
        self.return_type.format(f, indent_level)
    }
}

impl AstNode for SubprogramHeader {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // GENERIC ( generic_list ) [ generic_map_aspect ]
        parser.expect_keyword(KeywordKind::Generic)?;
        parser.expect(TokenKind::LeftParen)?;
        let generic_list = super::interface::GenericList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;

        // Optional generic_map_aspect: GENERIC MAP ( ... )
        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            Some(GenericMapAspect::parse(parser)?)
        } else {
            None
        };

        Ok(SubprogramHeader {
            generic_list,
            generic_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "generic (")?;
        self.generic_list.format(f, indent_level)?;
        write!(f, ")")?;
        if let Some(generic_map) = &self.generic_map_aspect {
            write!(f, " ")?;
            generic_map.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for SubprogramKind {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Procedure).is_some() {
            Ok(SubprogramKind::Procedure)
        } else if parser.consume_if_keyword(KeywordKind::Function).is_some() {
            Ok(SubprogramKind::Function)
        } else {
            Err(parser.error("expected subprogram kind (procedure or function)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            SubprogramKind::Procedure => write!(f, "procedure"),
            SubprogramKind::Function => write!(f, "function"),
        }
    }
}

impl AstNode for SubprogramDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::Begin) && !parser.eof() {
            items.push(SubprogramDeclarativeItem::parse(parser)?);
        }
        Ok(SubprogramDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for SubprogramDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                Ok(SubprogramDeclarativeItem::TypeDeclaration(Box::new(
                    super::type_def::TypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                Ok(SubprogramDeclarativeItem::SubtypeDeclaration(Box::new(
                    super::type_def::SubtypeDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                Ok(SubprogramDeclarativeItem::ConstantDeclaration(Box::new(
                    super::object_decl::ConstantDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Variable)) => {
                Ok(SubprogramDeclarativeItem::VariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                // SHARED VARIABLE
                Ok(SubprogramDeclarativeItem::VariableDeclaration(Box::new(
                    super::object_decl::VariableDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                Ok(SubprogramDeclarativeItem::FileDeclaration(Box::new(
                    super::object_decl::FileDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                Ok(SubprogramDeclarativeItem::AliasDeclaration(Box::new(
                    super::object_decl::AliasDeclaration::parse(parser)?,
                )))
            }
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                // Distinguish: ATTRIBUTE identifier : ... => declaration
                //              ATTRIBUTE identifier OF ... => specification
                // Look at token after ATTRIBUTE identifier
                // peek(0) = ATTRIBUTE, peek(1) = identifier, peek(2) = : or OF
                match parser.peek_nth(2).map(|t| t.kind) {
                    Some(TokenKind::Colon) => Ok(SubprogramDeclarativeItem::AttributeDeclaration(
                        Box::new(super::attribute::AttributeDeclaration::parse(parser)?),
                    )),
                    _ => {
                        // Default to specification (OF case or error)
                        Ok(SubprogramDeclarativeItem::AttributeSpecification(Box::new(
                            super::attribute::AttributeSpecification::parse(parser)?,
                        )))
                    }
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Use)) => Ok(SubprogramDeclarativeItem::UseClause(
                super::context::UseClause::parse(parser)?,
            )),
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                // Distinguish: GROUP identifier IS ... => template declaration
                //              GROUP identifier : ...  => group declaration
                // peek(0) = GROUP, peek(1) = identifier, peek(2) = IS or :
                match parser.peek_nth(2).map(|t| t.kind) {
                    Some(TokenKind::Keyword(KeywordKind::Is)) => {
                        Ok(SubprogramDeclarativeItem::GroupTemplateDeclaration(
                            Box::new(super::group::GroupTemplateDeclaration::parse(parser)?),
                        ))
                    }
                    _ => Ok(SubprogramDeclarativeItem::GroupDeclaration(Box::new(
                        super::group::GroupDeclaration::parse(parser)?,
                    ))),
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Function))
            | Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                // Could be: subprogram_declaration, subprogram_body, or
                // subprogram_instantiation_declaration (VHDL-2008)
                // Instantiation: subprogram_kind identifier IS NEW ...
                // For PROCEDURE: peek(0)=PROCEDURE, peek(1)=identifier, peek(2)=IS, peek(3)=NEW
                // For FUNCTION: may have PURE/IMPURE prefix

                // Check for instantiation first
                // For PROCEDURE: PROCEDURE identifier IS NEW
                if parser.at_keyword(KeywordKind::Procedure)
                    && let (Some(t2), Some(t3)) = (parser.peek_nth(1), parser.peek_nth(2))
                    && (t2.kind == TokenKind::Identifier
                        || t2.kind == TokenKind::ExtendedIdentifier)
                    && t3.kind == TokenKind::Keyword(KeywordKind::Is)
                    && let Some(t4) = parser.peek_nth(3)
                    && t4.kind == TokenKind::Keyword(KeywordKind::New)
                {
                    return Ok(
                        SubprogramDeclarativeItem::SubprogramInstantiationDeclaration(Box::new(
                            SubprogramInstantiationDeclaration::parse(parser)?,
                        )),
                    );
                }
                // For FUNCTION or PURE/IMPURE FUNCTION: check similarly
                if parser.at_keyword(KeywordKind::Function)
                    && let (Some(t2), Some(t3)) = (parser.peek_nth(1), parser.peek_nth(2))
                    && (t2.kind == TokenKind::Identifier
                        || t2.kind == TokenKind::ExtendedIdentifier)
                    && t3.kind == TokenKind::Keyword(KeywordKind::Is)
                    && let Some(t4) = parser.peek_nth(3)
                    && t4.kind == TokenKind::Keyword(KeywordKind::New)
                {
                    return Ok(
                        SubprogramDeclarativeItem::SubprogramInstantiationDeclaration(Box::new(
                            SubprogramInstantiationDeclaration::parse(parser)?,
                        )),
                    );
                }
                if parser.at_keyword(KeywordKind::Pure) || parser.at_keyword(KeywordKind::Impure) {
                    // PURE/IMPURE FUNCTION identifier IS NEW ...
                    if let (Some(t2), Some(t3), Some(t4)) =
                        (parser.peek_nth(1), parser.peek_nth(2), parser.peek_nth(3))
                        && t2.kind == TokenKind::Keyword(KeywordKind::Function)
                        && (t3.kind == TokenKind::Identifier
                            || t3.kind == TokenKind::ExtendedIdentifier)
                        && t4.kind == TokenKind::Keyword(KeywordKind::Is)
                        && let Some(t5) = parser.peek_nth(4)
                        && t5.kind == TokenKind::Keyword(KeywordKind::New)
                    {
                        return Ok(
                            SubprogramDeclarativeItem::SubprogramInstantiationDeclaration(
                                Box::new(SubprogramInstantiationDeclaration::parse(parser)?),
                            ),
                        );
                    }
                }

                // Not an instantiation, parse the specification then decide: ; => decl, IS => body
                let specification = SubprogramSpecification::parse(parser)?;
                if parser.at_keyword(KeywordKind::Is) {
                    // Subprogram body
                    parser.expect_keyword(KeywordKind::Is)?;
                    let declarative_part = SubprogramDeclarativePart::parse(parser)?;
                    parser.expect_keyword(KeywordKind::Begin)?;
                    let statement_part = SubprogramStatementPart::parse(parser)?;
                    parser.expect_keyword(KeywordKind::End)?;

                    let end_kind = if parser.at_keyword(KeywordKind::Procedure)
                        || parser.at_keyword(KeywordKind::Function)
                    {
                        Some(SubprogramKind::parse(parser)?)
                    } else {
                        None
                    };

                    let end_designator = match parser.peek_kind() {
                        Some(TokenKind::Identifier)
                        | Some(TokenKind::ExtendedIdentifier)
                        | Some(TokenKind::StringLiteral) => Some(Designator::parse(parser)?),
                        _ => None,
                    };

                    parser.expect(TokenKind::Semicolon)?;
                    Ok(SubprogramDeclarativeItem::SubprogramBody(Box::new(
                        SubprogramBody {
                            specification,
                            declarative_part,
                            statement_part,
                            end_kind,
                            end_designator,
                        },
                    )))
                } else {
                    // Subprogram declaration (;)
                    parser.expect(TokenKind::Semicolon)?;
                    Ok(SubprogramDeclarativeItem::SubprogramDeclaration(Box::new(
                        SubprogramDeclaration { specification },
                    )))
                }
            }
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // VHDL-2008: package_declaration, package_body, or package_instantiation_declaration
                // PACKAGE BODY ... => PackageBody
                // PACKAGE identifier IS NEW ... => PackageInstantiationDeclaration
                // PACKAGE identifier IS ... => PackageDeclaration
                if let Some(t2) = parser.peek_nth(1)
                    && t2.kind == TokenKind::Keyword(KeywordKind::Body)
                {
                    return Ok(SubprogramDeclarativeItem::PackageBody(Box::new(
                        super::package::PackageBody::parse(parser)?,
                    )));
                }
                // Check for instantiation: PACKAGE identifier IS NEW
                if let (Some(t2), Some(t3)) = (parser.peek_nth(1), parser.peek_nth(2))
                    && (t2.kind == TokenKind::Identifier
                        || t2.kind == TokenKind::ExtendedIdentifier)
                    && t3.kind == TokenKind::Keyword(KeywordKind::Is)
                    && let Some(t4) = parser.peek_nth(3)
                    && t4.kind == TokenKind::Keyword(KeywordKind::New)
                {
                    return Ok(SubprogramDeclarativeItem::PackageInstantiationDeclaration(
                        Box::new(super::package::PackageInstantiationDeclaration::parse(
                            parser,
                        )?),
                    ));
                }
                // Otherwise, package declaration
                Ok(SubprogramDeclarativeItem::PackageDeclaration(Box::new(
                    super::package::PackageDeclaration::parse(parser)?,
                )))
            }
            _ => Err(parser.error("expected subprogram declarative item")),
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

impl AstNode for SubprogramStatementPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(super::sequential::SequentialStatement::parse(parser)?);
        }
        Ok(SubprogramStatementPart { statements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for SubprogramInstantiationDeclaration {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let kind = SubprogramKind::parse(parser)?;
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;
        parser.expect_keyword(KeywordKind::New)?;
        let subprogram_name = Box::new(Name::parse(parser)?);

        // Optional signature (starts with `[`)
        let signature = if parser.at(TokenKind::LeftBracket) {
            Some(Signature::parse(parser)?)
        } else {
            None
        };

        // Optional generic_map_aspect: GENERIC MAP ( ... )
        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            Some(GenericMapAspect::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;
        Ok(SubprogramInstantiationDeclaration {
            kind,
            identifier,
            subprogram_name,
            signature,
            generic_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.kind.format(f, indent_level)?;
        write!(f, " ")?;
        self.identifier.format(f, indent_level)?;
        write!(f, " is new ")?;
        self.subprogram_name.format(f, indent_level)?;
        if let Some(sig) = &self.signature {
            write!(f, " ")?;
            sig.format(f, indent_level)?;
        }
        if let Some(generic_map) = &self.generic_map_aspect {
            write!(f, " ")?;
            generic_map.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}
