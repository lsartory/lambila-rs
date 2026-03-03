//! Generate statement AST nodes.

use super::common::*;
use super::concurrent::{BlockDeclarativePart, ConcurrentStatement};
use super::expression::{Choices, Condition, Expression};
use super::node::{AstNode, write_indent, format_lines};
use super::type_def::DiscreteRange;
use crate::parser::{Parser, ParseError};

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
    For(super::sequential::ParameterSpecification),
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
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for GenerateStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            IndexSpecification::DiscreteRange(range) => range.format(f, indent_level),
            IndexSpecification::Expression(expr) => expr.format(f, indent_level),
        }
    }
}
