//! Generate statement AST nodes.

use super::common::*;
use super::concurrent::{BlockDeclarativePart, ConcurrentStatement};
use super::expression::{Choices, Condition, Expression};
use super::type_def::DiscreteRange;

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
