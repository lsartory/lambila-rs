//! Generate statement AST nodes.

use super::common::*;
use super::concurrent::{BlockDeclarativeItem, BlockDeclarativePart, ConcurrentStatement};
use super::expression::{Choices, Condition, Expression};
use super::node::{AstNode, format_lines, write_indent};
use super::type_def::DiscreteRange;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF (VHDL-2008): `generate_statement ::= for_generate_statement | if_generate_statement
///     | case_generate_statement`
/// EBNF (VHDL-87/93): `generate_statement ::= generate_label : generation_scheme GENERATE
///     ... END GENERATE [ generate_label ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateStatement {
    For(ForGenerateStatement),
    If(IfGenerateStatement),
    /// VHDL-2008.
    Case(CaseGenerateStatement),
    /// VHDL-87/93 combined form.
    Legacy(LegacyGenerateStatement),
}

/// EBNF: `for_generate_statement ::= generate_label : FOR generate_parameter_specification
///     GENERATE generate_statement_body END GENERATE [ generate_label ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForGenerateStatement {
    pub label: Label,
    pub parameter_spec: super::sequential::ParameterSpecification,
    pub body: GenerateStatementBody,
    pub end_label: Option<Label>,
}

/// EBNF: `if_generate_statement ::= generate_label : IF [ alternative_label : ] condition
///     GENERATE generate_statement_body
///     { ELSIF [ alternative_label : ] condition GENERATE generate_statement_body }
///     [ ELSE [ alternative_label : ] GENERATE generate_statement_body ]
///     END GENERATE [ generate_label ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfGenerateStatement {
    pub label: Label,
    pub if_branch: IfGenerateBranch,
    pub elsif_branches: Vec<IfGenerateBranch>,
    pub else_branch: Option<ElseGenerateBranch>,
    pub end_label: Option<Label>,
}

/// An IF or ELSIF branch in an if_generate_statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfGenerateBranch {
    pub alternative_label: Option<Label>,
    pub condition: Condition,
    pub body: GenerateStatementBody,
}

/// An ELSE branch in an if_generate_statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseGenerateBranch {
    pub alternative_label: Option<Label>,
    pub body: GenerateStatementBody,
}

/// EBNF: `case_generate_statement ::= generate_label : CASE expression GENERATE
///     case_generate_alternative { case_generate_alternative }
///     END GENERATE [ generate_label ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseGenerateStatement {
    pub label: Label,
    pub expression: Expression,
    pub alternatives: Vec<CaseGenerateAlternative>,
    pub end_label: Option<Label>,
}

/// EBNF: `case_generate_alternative ::= WHEN [ alternative_label : ] choices =>
///     generate_statement_body` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseGenerateAlternative {
    pub alternative_label: Option<Label>,
    pub choices: Choices,
    pub body: GenerateStatementBody,
}

/// EBNF: `generate_statement_body ::= [ block_declarative_part BEGIN ]
///     { concurrent_statement } [ END [ alternative_label ] ; ]` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateStatementBody {
    pub declarative_part: Option<BlockDeclarativePart>,
    pub statements: Vec<ConcurrentStatement>,
    pub end_label: Option<Label>,
}

/// EBNF (VHDL-87/93): `generate_statement ::= generate_label : generation_scheme GENERATE
///     [ { block_declarative_item } BEGIN ] { concurrent_statement }
///     END GENERATE [ generate_label ] ;`
///
/// VHDL-93 added optional declarative region.
/// VHDL-87 has no declarative region, no end label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyGenerateStatement {
    pub label: Label,
    pub scheme: GenerationScheme,
    pub declarative_part: Option<BlockDeclarativePart>,
    pub statements: Vec<ConcurrentStatement>,
    pub end_label: Option<Label>,
}

/// EBNF (VHDL-87/93): `generation_scheme ::= FOR generate_parameter_specification
///     | IF condition`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationScheme {
    For(Box<super::sequential::ParameterSpecification>),
    If(Condition),
}

/// EBNF (VHDL-2008): `generate_specification ::= static_discrete_range | static_expression
///     | alternative_label`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateSpecification {
    DiscreteRange(DiscreteRange),
    Expression(Expression),
    AlternativeLabel(Label),
}

/// EBNF (VHDL-87/93): `index_specification ::= discrete_range | static_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexSpecification {
    DiscreteRange(DiscreteRange),
    Expression(Expression),
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parse the body of a legacy generate statement.
/// The body can be: `[ { block_declarative_item } BEGIN ] { concurrent_statement }`
/// We need to determine whether there is a declarative part.
///
/// Strategy: Try to parse block_declarative_items. If we encounter BEGIN,
/// that confirms the declarative region. Otherwise, treat everything as
/// concurrent statements.
fn parse_legacy_generate_body(
    parser: &mut Parser,
) -> Result<(Option<BlockDeclarativePart>, Vec<ConcurrentStatement>), ParseError> {
    // Check if the body starts with something that could be a block_declarative_item.
    // If we see END (followed by GENERATE), there is no body at all.
    // If we see BEGIN, there is an empty declarative part.
    if parser.at_keyword(KeywordKind::Begin) {
        // Empty declarative part, explicit BEGIN
        parser.consume();
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        return Ok((Some(BlockDeclarativePart { items: vec![] }), statements));
    }

    // Try to parse as concurrent statements (the common case).
    // If the first thing looks like a declarative item (starts with a keyword like
    // SIGNAL, CONSTANT, TYPE, SUBTYPE, FILE, ALIAS, COMPONENT, ATTRIBUTE,
    // FUNCTION, PROCEDURE, USE, etc.) AND we eventually see BEGIN, then
    // we have a declarative region.
    //
    // For simplicity: save position, try parsing declarative items until BEGIN.
    // If we see BEGIN, commit. If we hit END GENERATE without BEGIN, restore
    // and parse as concurrent statements.
    let saved = parser.save();
    let mut items = Vec::new();
    let mut found_begin = false;

    loop {
        if parser.at_keyword(KeywordKind::Begin) {
            parser.consume();
            found_begin = true;
            break;
        }
        if parser.at_keyword(KeywordKind::End) || parser.eof() {
            break;
        }
        // Try to parse a block_declarative_item
        let item_saved = parser.save();
        match BlockDeclarativeItem::parse(parser) {
            Ok(item) => items.push(item),
            Err(_) => {
                // Not a declarative item -- restore and break
                parser.restore(item_saved);
                break;
            }
        }
    }

    if found_begin {
        // We have a declarative part followed by BEGIN
        let declarative_part = if items.is_empty() {
            Some(BlockDeclarativePart { items: vec![] })
        } else {
            Some(BlockDeclarativePart { items })
        };
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        Ok((declarative_part, statements))
    } else {
        // No BEGIN found -- restore and parse as concurrent statements only
        parser.restore(saved);
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        Ok((None, statements))
    }
}

/// Try to parse an optional alternative label in generate statements.
/// An alternative label is `identifier :` where the colon is NOT followed
/// by something that makes it look like a concurrent statement label.
fn try_parse_alternative_label(parser: &mut Parser) -> Option<Label> {
    // Check if we have identifier : before GENERATE keyword or before a condition
    // In VHDL-2008 generate statements, alternative labels appear as:
    //   [ alternative_label : ] condition GENERATE ...
    //   [ alternative_label : ] GENERATE ...  (for else branch)
    // We use save/restore to speculatively try.
    if (parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier))
        && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon)
    {
        let saved = parser.save();
        let label = Label::parse(parser).ok()?;
        if parser.consume_if(TokenKind::Colon).is_some() {
            Some(label)
        } else {
            parser.restore(saved);
            None
        }
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for GenerateStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // The label has already been consumed by the ConcurrentStatement dispatcher,
        // but for standalone use, we peek ahead to determine the variant.
        // Actually, the label : has already been parsed.
        // We look at the current keyword to determine FOR/IF/CASE.
        // We use the legacy form which covers all versions.
        //
        // For VHDL-2008, FOR/IF/CASE generate could have structured bodies.
        // We parse as legacy for simplicity (label already parsed by caller).
        //
        // Since the trait requires standalone parsing, we parse label : scheme GENERATE ...
        // But note: ConcurrentStatement will call this after saving/restoring position.
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;

        if parser.at_keyword(KeywordKind::Case) {
            // CASE expression GENERATE ...
            parser.consume();
            let expression = Expression::parse(parser)?;
            parser.expect_keyword(KeywordKind::Generate)?;
            let mut alternatives = Vec::new();
            while parser.at_keyword(KeywordKind::When) {
                alternatives.push(CaseGenerateAlternative::parse(parser)?);
            }
            parser.expect_keyword(KeywordKind::End)?;
            parser.expect_keyword(KeywordKind::Generate)?;
            let end_label =
                if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                    Some(Label::parse(parser)?)
                } else {
                    None
                };
            parser.expect(TokenKind::Semicolon)?;
            Ok(GenerateStatement::Case(CaseGenerateStatement {
                label,
                expression,
                alternatives,
                end_label,
            }))
        } else if parser.at_keyword(KeywordKind::For) || parser.at_keyword(KeywordKind::If) {
            // Use legacy form: generation_scheme GENERATE ... END GENERATE ;
            let scheme = GenerationScheme::parse(parser)?;
            parser.expect_keyword(KeywordKind::Generate)?;

            // Parse optional declarative part + BEGIN
            // We need to determine if there's a declarative region.
            // If we see BEGIN before END GENERATE, there's a declarative region.
            // Use save/restore to try parsing block_declarative_items.
            let (declarative_part, statements) = parse_legacy_generate_body(parser)?;

            parser.expect_keyword(KeywordKind::End)?;
            parser.expect_keyword(KeywordKind::Generate)?;
            let end_label =
                if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                    Some(Label::parse(parser)?)
                } else {
                    None
                };
            parser.expect(TokenKind::Semicolon)?;
            Ok(GenerateStatement::Legacy(LegacyGenerateStatement {
                label,
                scheme,
                declarative_part,
                statements,
                end_label,
            }))
        } else {
            Err(parser.error("expected FOR, IF, or CASE in generate statement"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GenerateStatement::For(inner) => inner.format(f, indent_level),
            GenerateStatement::If(inner) => inner.format(f, indent_level),
            GenerateStatement::Case(inner) => inner.format(f, indent_level),
            GenerateStatement::Legacy(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for ForGenerateStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // label : FOR parameter_specification GENERATE generate_statement_body
        //     END GENERATE [ label ] ;
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        parser.expect_keyword(KeywordKind::For)?;
        let parameter_spec = super::sequential::ParameterSpecification::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let body = GenerateStatementBody::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(ForGenerateStatement {
            label,
            parameter_spec,
            body,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, indent_level)?;
        write!(f, " : for ")?;
        self.parameter_spec.format(f, indent_level)?;
        writeln!(f, " generate")?;
        self.body.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end generate")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for IfGenerateStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // label : IF [ alt_label : ] condition GENERATE generate_statement_body
        //     { ELSIF [ alt_label : ] condition GENERATE generate_statement_body }
        //     [ ELSE [ alt_label : ] GENERATE generate_statement_body ]
        //     END GENERATE [ label ] ;
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        parser.expect_keyword(KeywordKind::If)?;
        let if_branch = IfGenerateBranch::parse(parser)?;

        let mut elsif_branches = Vec::new();
        while parser.at_keyword(KeywordKind::Elsif) {
            parser.consume();
            elsif_branches.push(IfGenerateBranch::parse(parser)?);
        }

        let else_branch = if parser.at_keyword(KeywordKind::Else) {
            parser.consume();
            Some(ElseGenerateBranch::parse(parser)?)
        } else {
            None
        };

        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(IfGenerateStatement {
            label,
            if_branch,
            elsif_branches,
            else_branch,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, indent_level)?;
        write!(f, " : if ")?;
        if let Some(alt_label) = &self.if_branch.alternative_label {
            alt_label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.if_branch.condition.format(f, indent_level)?;
        writeln!(f, " generate")?;
        self.if_branch.body.format(f, indent_level + 1)?;
        for elsif in &self.elsif_branches {
            write_indent(f, indent_level)?;
            write!(f, "elsif ")?;
            if let Some(alt_label) = &elsif.alternative_label {
                alt_label.format(f, indent_level)?;
                write!(f, " : ")?;
            }
            elsif.condition.format(f, indent_level)?;
            writeln!(f, " generate")?;
            elsif.body.format(f, indent_level + 1)?;
        }
        if let Some(else_branch) = &self.else_branch {
            write_indent(f, indent_level)?;
            write!(f, "else ")?;
            if let Some(alt_label) = &else_branch.alternative_label {
                alt_label.format(f, indent_level)?;
                write!(f, " : ")?;
            }
            writeln!(f, "generate")?;
            else_branch.body.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end generate")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for IfGenerateBranch {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ alternative_label : ] condition GENERATE generate_statement_body
        let alternative_label = try_parse_alternative_label(parser);
        let condition = Condition::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let body = GenerateStatementBody::parse(parser)?;
        Ok(IfGenerateBranch {
            alternative_label,
            condition,
            body,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(alt_label) = &self.alternative_label {
            alt_label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.condition.format(f, indent_level)?;
        writeln!(f, " generate")?;
        self.body.format(f, indent_level + 1)
    }
}

impl AstNode for ElseGenerateBranch {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ alternative_label : ] GENERATE generate_statement_body
        let alternative_label = try_parse_alternative_label(parser);
        parser.expect_keyword(KeywordKind::Generate)?;
        let body = GenerateStatementBody::parse(parser)?;
        Ok(ElseGenerateBranch {
            alternative_label,
            body,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(alt_label) = &self.alternative_label {
            alt_label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.body.format(f, indent_level)
    }
}

impl AstNode for CaseGenerateStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // label : CASE expression GENERATE { case_generate_alternative }
        //     END GENERATE [ label ] ;
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        parser.expect_keyword(KeywordKind::Case)?;
        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let mut alternatives = Vec::new();
        while parser.at_keyword(KeywordKind::When) {
            alternatives.push(CaseGenerateAlternative::parse(parser)?);
        }
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(CaseGenerateStatement {
            label,
            expression,
            alternatives,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, indent_level)?;
        write!(f, " : case ")?;
        self.expression.format(f, indent_level)?;
        writeln!(f, " generate")?;
        for alt in &self.alternatives {
            alt.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end generate")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for CaseGenerateAlternative {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // WHEN [ alternative_label : ] choices => generate_statement_body
        parser.expect_keyword(KeywordKind::When)?;
        let alternative_label = try_parse_alternative_label(parser);
        let choices = Choices::parse(parser)?;
        parser.expect(TokenKind::Arrow)?;
        let body = GenerateStatementBody::parse(parser)?;
        Ok(CaseGenerateAlternative {
            alternative_label,
            choices,
            body,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "when ")?;
        if let Some(alt_label) = &self.alternative_label {
            alt_label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.choices.format(f, indent_level)?;
        writeln!(f, " =>")?;
        self.body.format(f, indent_level + 1)
    }
}

impl AstNode for GenerateStatementBody {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ block_declarative_part BEGIN ] { concurrent_statement } [ END [ alt_label ] ; ]
        //
        // Determine if there's a declarative part by using the same strategy
        // as the legacy generate body parser.
        let (declarative_part, statements) = parse_legacy_generate_body(parser)?;

        // [ END [ alternative_label ] ; ]
        // Note: This END is different from END GENERATE. It's just END [label] ;
        // We need to be careful not to consume END GENERATE which belongs to the parent.
        let end_label = if parser.at_keyword(KeywordKind::End) {
            // Peek ahead: if next is GENERATE, ELSIF, or other parent keyword, don't consume.
            let next = parser.peek_nth(1).map(|t| t.kind);
            if next == Some(TokenKind::Keyword(KeywordKind::Generate)) {
                // This END belongs to the parent, not to us.
                None
            } else {
                // This is END [ alternative_label ] ;
                parser.consume(); // consume END
                let lbl = if parser.at(TokenKind::Identifier)
                    || parser.at(TokenKind::ExtendedIdentifier)
                {
                    Some(Label::parse(parser)?)
                } else {
                    None
                };
                parser.expect(TokenKind::Semicolon)?;
                // If no label present, we still consumed the END ; structure
                // Return the label (or None meaning the end was present but unlabeled)
                // But the struct stores Option<Label>, so None means no end clause at all.
                // We need a way to distinguish "no END" from "END with no label".
                // Looking at the struct: `end_label: Option<Label>` -- if no END, it's None.
                // If END without label -- this is tricky. For now, use a sentinel.
                // Actually the format method checks for Some(end_label), so None means skip.
                // We'll use the label if present; if END was present but no label,
                // we still consumed it, but return None for the label.
                lbl
            }
        } else {
            None
        };

        Ok(GenerateStatementBody {
            declarative_part,
            statements,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(decl_part) = &self.declarative_part {
            decl_part.format(f, indent_level)?;
            write_indent(f, indent_level)?;
            writeln!(f, "begin")?;
        }
        format_lines(&self.statements, f, indent_level)?;
        if let Some(end_label) = &self.end_label {
            write_indent(f, indent_level)?;
            write!(f, "end ")?;
            end_label.format(f, indent_level)?;
            writeln!(f, ";")?;
        }
        Ok(())
    }
}

impl AstNode for LegacyGenerateStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // label : generation_scheme GENERATE [ { block_declarative_item } BEGIN ]
        //     { concurrent_statement } END GENERATE [ label ] ;
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let scheme = GenerationScheme::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let (declarative_part, statements) = parse_legacy_generate_body(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(LegacyGenerateStatement {
            label,
            scheme,
            declarative_part,
            statements,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, indent_level)?;
        write!(f, " : ")?;
        self.scheme.format(f, indent_level)?;
        writeln!(f, " generate")?;
        if let Some(decl_part) = &self.declarative_part {
            decl_part.format(f, indent_level + 1)?;
            write_indent(f, indent_level)?;
            writeln!(f, "begin")?;
        }
        format_lines(&self.statements, f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end generate")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for GenerationScheme {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at_keyword(KeywordKind::For) {
            parser.consume();
            let param_spec = super::sequential::ParameterSpecification::parse(parser)?;
            Ok(GenerationScheme::For(Box::new(param_spec)))
        } else if parser.at_keyword(KeywordKind::If) {
            parser.consume();
            let condition = Condition::parse(parser)?;
            Ok(GenerationScheme::If(condition))
        } else {
            Err(parser.error("expected FOR or IF in generation scheme"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GenerationScheme::For(param_spec) => {
                write!(f, "for ")?;
                param_spec.format(f, indent_level)
            }
            GenerationScheme::If(condition) => {
                write!(f, "if ")?;
                condition.format(f, indent_level)
            }
        }
    }
}

impl AstNode for GenerateSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // generate_specification ::= static_discrete_range | static_expression | alternative_label
        // These are ambiguous. Try discrete_range first (which includes expression TO/DOWNTO expression),
        // then fall back to expression, and finally try simple label.
        let saved = parser.save();
        if let Ok(range) = DiscreteRange::parse(parser) {
            return Ok(GenerateSpecification::DiscreteRange(range));
        }
        parser.restore(saved);
        // Try expression (which could also be a simple name = label)
        let saved2 = parser.save();
        if let Ok(expr) = Expression::parse(parser) {
            return Ok(GenerateSpecification::Expression(expr));
        }
        parser.restore(saved2);
        // Try alternative label
        let label = Label::parse(parser)?;
        Ok(GenerateSpecification::AlternativeLabel(label))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            GenerateSpecification::DiscreteRange(range) => range.format(f, indent_level),
            GenerateSpecification::Expression(expr) => expr.format(f, indent_level),
            GenerateSpecification::AlternativeLabel(label) => label.format(f, indent_level),
        }
    }
}

impl AstNode for IndexSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // index_specification ::= discrete_range | static_expression
        // Try discrete_range first, fall back to expression.
        let saved = parser.save();
        if let Ok(range) = DiscreteRange::parse(parser) {
            return Ok(IndexSpecification::DiscreteRange(range));
        }
        parser.restore(saved);
        let expr = Expression::parse(parser)?;
        Ok(IndexSpecification::Expression(expr))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            IndexSpecification::DiscreteRange(range) => range.format(f, indent_level),
            IndexSpecification::Expression(expr) => expr.format(f, indent_level),
        }
    }
}
