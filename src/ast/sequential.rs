//! Sequential statement AST nodes.

use super::common::*;
use super::expression::*;
use super::name::Name;
use super::node::{AstNode, format_comma_separated, format_lines, write_indent};
use super::type_def::DiscreteRange;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF (VHDL-2008): `sequential_statement ::= wait_statement | assertion_statement
///     | report_statement | signal_assignment_statement | variable_assignment_statement
///     | procedure_call_statement | if_statement | case_statement | loop_statement
///     | next_statement | exit_statement | return_statement | null_statement`
/// EBNF (VHDL-87): omits `report_statement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequentialStatement {
    Wait(Box<WaitStatement>),
    Assertion(Box<AssertionStatement>),
    /// VHDL-93+.
    Report(Box<ReportStatement>),
    SignalAssignment(Box<SignalAssignmentStatement>),
    VariableAssignment(Box<VariableAssignmentStatement>),
    ProcedureCall(Box<ProcedureCallStatement>),
    If(Box<IfStatement>),
    Case(Box<CaseStatement>),
    Loop(Box<LoopStatement>),
    Next(Box<NextStatement>),
    Exit(Box<ExitStatement>),
    Return(Box<ReturnStatement>),
    Null(Box<NullStatement>),
}

/// EBNF: `sequence_of_statements ::= { sequential_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceOfStatements {
    pub statements: Vec<SequentialStatement>,
}

// ─── Wait ───────────────────────────────────────────────────────────────

/// EBNF: `wait_statement ::= [ label : ] WAIT [ sensitivity_clause ]
///     [ condition_clause ] [ timeout_clause ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitStatement {
    pub label: Option<Label>,
    pub sensitivity_clause: Option<SensitivityClause>,
    pub condition_clause: Option<ConditionClause>,
    pub timeout_clause: Option<TimeoutClause>,
}

/// EBNF: `sensitivity_clause ::= ON sensitivity_list`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivityClause {
    pub sensitivity_list: SensitivityList,
}

/// EBNF: `sensitivity_list ::= signal_name { , signal_name }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivityList {
    pub signals: Vec<Name>,
}

/// EBNF: `condition_clause ::= UNTIL condition`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionClause {
    pub condition: Condition,
}

/// EBNF: `timeout_clause ::= FOR time_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutClause {
    pub time_expression: Expression,
}

// ─── Assertion / Report ─────────────────────────────────────────────────

/// EBNF: `assertion ::= ASSERT condition [ REPORT expression ] [ SEVERITY expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assertion {
    pub condition: Condition,
    pub report: Option<Expression>,
    pub severity: Option<Expression>,
}

/// EBNF: `assertion_statement ::= [ label : ] assertion ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionStatement {
    pub label: Option<Label>,
    pub assertion: Assertion,
}

/// EBNF: `report_statement ::= [ label : ] REPORT expression [ SEVERITY expression ] ;`
/// (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportStatement {
    pub label: Option<Label>,
    pub report_expression: Expression,
    pub severity: Option<Expression>,
}

// ─── Signal assignment ──────────────────────────────────────────────────

/// EBNF (VHDL-2008): `signal_assignment_statement ::= [ label : ] simple_signal_assignment
///     | [ label : ] conditional_signal_assignment
///     | [ label : ] selected_signal_assignment`
/// EBNF (VHDL-87/93): `signal_assignment_statement ::= [ label : ] target <=
///     [ delay_mechanism ] waveform ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalAssignmentStatement {
    Simple {
        label: Option<Label>,
        assignment: SimpleSignalAssignment,
    },
    /// VHDL-2008.
    Conditional {
        label: Option<Label>,
        assignment: ConditionalSignalAssignment,
    },
    /// VHDL-2008.
    Selected {
        label: Option<Label>,
        assignment: SelectedSignalAssignment,
    },
}

/// EBNF (VHDL-2008): `simple_signal_assignment ::= simple_waveform_assignment
///     | simple_force_assignment | simple_release_assignment`
/// EBNF (VHDL-87/93): just `target <= [ delay_mechanism ] waveform ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSignalAssignment {
    Waveform(SimpleWaveformAssignment),
    /// VHDL-2008.
    Force(SimpleForceAssignment),
    /// VHDL-2008.
    Release(SimpleReleaseAssignment),
}

/// EBNF: `simple_waveform_assignment ::= target <= [ delay_mechanism ] waveform ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleWaveformAssignment {
    pub target: Target,
    pub delay_mechanism: Option<DelayMechanism>,
    pub waveform: Waveform,
}

/// EBNF: `simple_force_assignment ::= target <= FORCE [ force_mode ] expression ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleForceAssignment {
    pub target: Target,
    pub force_mode: Option<ForceMode>,
    pub expression: Expression,
}

/// EBNF: `simple_release_assignment ::= target <= RELEASE [ force_mode ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleReleaseAssignment {
    pub target: Target,
    pub force_mode: Option<ForceMode>,
}

/// EBNF (VHDL-2008): `conditional_signal_assignment ::= conditional_waveform_assignment
///     | conditional_force_assignment`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionalSignalAssignment {
    Waveform(ConditionalWaveformAssignment),
    Force(ConditionalForceAssignment),
}

/// EBNF: `conditional_waveform_assignment ::= target <= [ delay_mechanism ]
///     conditional_waveforms ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalWaveformAssignment {
    pub target: Target,
    pub delay_mechanism: Option<DelayMechanism>,
    pub conditional_waveforms: ConditionalWaveforms,
}

/// EBNF: `conditional_force_assignment ::= target <= FORCE [ force_mode ]
///     conditional_expressions ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalForceAssignment {
    pub target: Target,
    pub force_mode: Option<ForceMode>,
    pub conditional_expressions: ConditionalExpressions,
}

/// EBNF (VHDL-2008): `selected_signal_assignment ::= selected_waveform_assignment
///     | selected_force_assignment`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedSignalAssignment {
    Waveform(SelectedWaveformAssignment),
    Force(SelectedForceAssignment),
}

/// EBNF: `selected_waveform_assignment ::= WITH expression SELECT [ ? ] target <=
///     [ delay_mechanism ] selected_waveforms ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedWaveformAssignment {
    pub selector: Expression,
    pub matching: bool,
    pub target: Target,
    pub delay_mechanism: Option<DelayMechanism>,
    pub selected_waveforms: SelectedWaveforms,
}

/// EBNF: `selected_force_assignment ::= WITH expression SELECT [ ? ] target <=
///     FORCE [ force_mode ] selected_expressions ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedForceAssignment {
    pub selector: Expression,
    pub matching: bool,
    pub target: Target,
    pub force_mode: Option<ForceMode>,
    pub selected_expressions: SelectedExpressions,
}

/// EBNF: `delay_mechanism ::= TRANSPORT | [ REJECT time_expression ] INERTIAL`
/// EBNF (VHDL-87): `delay_mechanism ::= TRANSPORT`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelayMechanism {
    Transport,
    /// VHDL-93+: `[ REJECT time_expression ] INERTIAL`
    Inertial {
        reject_time: Option<Expression>,
    },
}

/// EBNF: `waveform ::= waveform_element { , waveform_element } | UNAFFECTED`
/// EBNF (VHDL-87): `waveform ::= waveform_element { , waveform_element }` (no UNAFFECTED).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waveform {
    Elements(Vec<WaveformElement>),
    /// VHDL-93+.
    Unaffected,
}

/// EBNF: `waveform_element ::= value_expression [ AFTER time_expression ]
///     | NULL [ AFTER time_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveformElement {
    Value {
        expression: Expression,
        after: Option<Expression>,
    },
    Null {
        after: Option<Expression>,
    },
}

/// EBNF (VHDL-2008): `conditional_waveforms ::= waveform WHEN condition
///     { ELSE waveform WHEN condition } [ ELSE waveform ]`
/// EBNF (VHDL-87/93): `conditional_waveforms ::= { waveform WHEN condition ELSE }
///     waveform [ WHEN condition ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalWaveforms {
    pub alternatives: Vec<ConditionalWaveformAlternative>,
    pub else_waveform: Option<Waveform>,
}

/// A single conditional waveform alternative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalWaveformAlternative {
    pub waveform: Waveform,
    pub condition: Condition,
}

/// EBNF: `selected_waveforms ::= { waveform WHEN choices , } waveform WHEN choices`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedWaveforms {
    pub alternatives: Vec<SelectedWaveformAlternative>,
}

/// A single selected waveform alternative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedWaveformAlternative {
    pub waveform: Waveform,
    pub choices: Choices,
}

/// EBNF: `force_mode ::= IN | OUT` (VHDL-2008)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceMode {
    In,
    Out,
}

/// EBNF: `target ::= name | aggregate`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Name(Name),
    Aggregate(Aggregate),
}

// ─── Variable assignment ────────────────────────────────────────────────

/// EBNF (VHDL-2008): `variable_assignment_statement ::= [ label : ] simple_variable_assignment
///     | [ label : ] conditional_variable_assignment
///     | [ label : ] selected_variable_assignment`
/// EBNF (VHDL-87/93): `variable_assignment_statement ::= [ label : ] target := expression ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableAssignmentStatement {
    Simple {
        label: Option<Label>,
        assignment: SimpleVariableAssignment,
    },
    /// VHDL-2008.
    Conditional {
        label: Option<Label>,
        assignment: ConditionalVariableAssignment,
    },
    /// VHDL-2008.
    Selected {
        label: Option<Label>,
        assignment: SelectedVariableAssignment,
    },
}

/// EBNF: `simple_variable_assignment ::= target := expression ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleVariableAssignment {
    pub target: Target,
    pub expression: Expression,
}

/// EBNF: `conditional_variable_assignment ::= target := conditional_expressions ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalVariableAssignment {
    pub target: Target,
    pub conditional_expressions: ConditionalExpressions,
}

/// EBNF: `selected_variable_assignment ::= WITH expression SELECT [ ? ]
///     target := selected_expressions ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedVariableAssignment {
    pub selector: Expression,
    pub matching: bool,
    pub target: Target,
    pub selected_expressions: SelectedExpressions,
}

// ─── Procedure call ─────────────────────────────────────────────────────

/// EBNF: `procedure_call ::= procedure_name [ ( actual_parameter_part ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureCall {
    pub procedure_name: Name,
    pub parameters: Option<super::association::ActualParameterPart>,
}

/// EBNF: `procedure_call_statement ::= [ label : ] procedure_call ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureCallStatement {
    pub label: Option<Label>,
    pub procedure_call: ProcedureCall,
}

// ─── If statement ───────────────────────────────────────────────────────

/// EBNF (VHDL-93+): `if_statement ::= [ if_label : ] IF condition THEN
///     sequence_of_statements { ELSIF condition THEN sequence_of_statements }
///     [ ELSE sequence_of_statements ] END IF [ if_label ] ;`
/// EBNF (VHDL-87): no label, no end label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub label: Option<Label>,
    pub condition: Condition,
    pub then_statements: SequenceOfStatements,
    pub elsif_branches: Vec<ElsifBranch>,
    pub else_statements: Option<SequenceOfStatements>,
    pub end_label: Option<Label>,
}

/// An ELSIF branch within an if statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElsifBranch {
    pub condition: Condition,
    pub statements: SequenceOfStatements,
}

// ─── Case statement ─────────────────────────────────────────────────────

/// EBNF (VHDL-2008): `case_statement ::= [ case_label : ] CASE [ ? ] expression IS
///     case_statement_alternative { case_statement_alternative }
///     END CASE [ ? ] [ case_label ] ;`
/// EBNF (VHDL-87): `case_statement ::= CASE expression IS ... END CASE ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseStatement {
    pub label: Option<Label>,
    /// VHDL-2008: matching case (`?`).
    pub matching: bool,
    pub expression: Expression,
    pub alternatives: Vec<CaseStatementAlternative>,
    pub end_label: Option<Label>,
}

/// EBNF: `case_statement_alternative ::= WHEN choices => sequence_of_statements`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseStatementAlternative {
    pub choices: Choices,
    pub statements: SequenceOfStatements,
}

// ─── Loop statement ─────────────────────────────────────────────────────

/// EBNF (VHDL-93+): `loop_statement ::= [ loop_label : ] [ iteration_scheme ] LOOP
///     sequence_of_statements END LOOP [ loop_label ] ;`
/// EBNF (VHDL-87): no end label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopStatement {
    pub label: Option<Label>,
    pub iteration_scheme: Option<IterationScheme>,
    pub statements: SequenceOfStatements,
    pub end_label: Option<Label>,
}

/// EBNF: `iteration_scheme ::= WHILE condition | FOR loop_parameter_specification`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IterationScheme {
    While(Condition),
    For(Box<ParameterSpecification>),
}

/// EBNF: `parameter_specification ::= identifier IN discrete_range`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterSpecification {
    pub identifier: Identifier,
    pub discrete_range: DiscreteRange,
}

// ─── Simple statements ──────────────────────────────────────────────────

/// EBNF: `next_statement ::= [ label : ] NEXT [ loop_label ] [ WHEN condition ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextStatement {
    pub label: Option<Label>,
    pub loop_label: Option<Label>,
    pub condition: Option<Condition>,
}

/// EBNF: `exit_statement ::= [ label : ] EXIT [ loop_label ] [ WHEN condition ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStatement {
    pub label: Option<Label>,
    pub loop_label: Option<Label>,
    pub condition: Option<Condition>,
}

/// EBNF: `return_statement ::= [ label : ] RETURN [ expression ] ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub label: Option<Label>,
    pub expression: Option<Expression>,
}

/// EBNF: `null_statement ::= [ label : ] NULL ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullStatement {
    pub label: Option<Label>,
}

// ─── Helper functions ────────────────────────────────────────────────────

/// Try to parse an optional `ForceMode` (IN | OUT). Returns `None` if the
/// current token is neither `IN` nor `OUT`.
fn parse_optional_force_mode(parser: &mut Parser) -> Option<ForceMode> {
    if parser.at_keyword(KeywordKind::In) {
        parser.consume();
        Some(ForceMode::In)
    } else if parser.at_keyword(KeywordKind::Out) {
        parser.consume();
        Some(ForceMode::Out)
    } else {
        None
    }
}

/// Try to parse an optional `DelayMechanism`. Returns `Ok(None)` if the
/// current token does not start a delay mechanism.
fn parse_optional_delay_mechanism(
    parser: &mut Parser,
) -> Result<Option<DelayMechanism>, ParseError> {
    if parser.at_keyword(KeywordKind::Transport)
        || parser.at_keyword(KeywordKind::Reject)
        || parser.at_keyword(KeywordKind::Inertial)
    {
        Ok(Some(DelayMechanism::parse(parser)?))
    } else {
        Ok(None)
    }
}

/// After parsing `target <=`, determine which kind of signal assignment
/// follows. Returns a `SignalAssignmentStatement` with `label: None`
/// (the caller sets the label).
fn parse_signal_assignment_after_arrow(
    parser: &mut Parser,
    target: Target,
) -> Result<SignalAssignmentStatement, ParseError> {
    if parser.at_keyword(KeywordKind::Force) {
        // FORCE [force_mode] expression_or_conditional ;
        parser.expect_keyword(KeywordKind::Force)?;
        let force_mode = parse_optional_force_mode(parser);
        let expression = Expression::parse(parser)?;
        // Check for WHEN → conditional force assignment.
        if parser.at_keyword(KeywordKind::When) {
            // This is a conditional force assignment.
            // We need to build ConditionalExpressions. The first expression
            // is already parsed; now parse WHEN condition {ELSE expression WHEN condition} [ELSE expression].
            parser.expect_keyword(KeywordKind::When)?;
            let first_condition = Expression::parse(parser)?;
            let mut alternatives = vec![ConditionalAlternative {
                expression: Box::new(expression),
                condition: first_condition,
            }];
            let mut else_expression = None;
            while parser.consume_if_keyword(KeywordKind::Else).is_some() {
                let expr = Expression::parse(parser)?;
                if parser.consume_if_keyword(KeywordKind::When).is_some() {
                    let cond = Expression::parse(parser)?;
                    alternatives.push(ConditionalAlternative {
                        expression: Box::new(expr),
                        condition: cond,
                    });
                } else {
                    else_expression = Some(Box::new(expr));
                    break;
                }
            }
            Ok(SignalAssignmentStatement::Conditional {
                label: None,
                assignment: ConditionalSignalAssignment::Force(ConditionalForceAssignment {
                    target,
                    force_mode,
                    conditional_expressions: ConditionalExpressions {
                        alternatives,
                        else_expression,
                    },
                }),
            })
        } else {
            Ok(SignalAssignmentStatement::Simple {
                label: None,
                assignment: SimpleSignalAssignment::Force(SimpleForceAssignment {
                    target,
                    force_mode,
                    expression,
                }),
            })
        }
    } else if parser.at_keyword(KeywordKind::Release) {
        parser.expect_keyword(KeywordKind::Release)?;
        let force_mode = parse_optional_force_mode(parser);
        Ok(SignalAssignmentStatement::Simple {
            label: None,
            assignment: SimpleSignalAssignment::Release(SimpleReleaseAssignment {
                target,
                force_mode,
            }),
        })
    } else {
        // [delay_mechanism] waveform [WHEN condition ...] ;
        let delay_mechanism = parse_optional_delay_mechanism(parser)?;
        let waveform = Waveform::parse(parser)?;

        if parser.at_keyword(KeywordKind::When) {
            // Conditional waveform assignment.
            // waveform WHEN condition { ELSE waveform WHEN condition } [ELSE waveform]
            parser.expect_keyword(KeywordKind::When)?;
            let first_condition = Expression::parse(parser)?;
            let mut alternatives = vec![ConditionalWaveformAlternative {
                waveform,
                condition: first_condition,
            }];
            let mut else_waveform = None;
            while parser.consume_if_keyword(KeywordKind::Else).is_some() {
                let wf = Waveform::parse(parser)?;
                if parser.consume_if_keyword(KeywordKind::When).is_some() {
                    let cond = Expression::parse(parser)?;
                    alternatives.push(ConditionalWaveformAlternative {
                        waveform: wf,
                        condition: cond,
                    });
                } else {
                    else_waveform = Some(wf);
                    break;
                }
            }
            Ok(SignalAssignmentStatement::Conditional {
                label: None,
                assignment: ConditionalSignalAssignment::Waveform(ConditionalWaveformAssignment {
                    target,
                    delay_mechanism,
                    conditional_waveforms: ConditionalWaveforms {
                        alternatives,
                        else_waveform,
                    },
                }),
            })
        } else {
            Ok(SignalAssignmentStatement::Simple {
                label: None,
                assignment: SimpleSignalAssignment::Waveform(SimpleWaveformAssignment {
                    target,
                    delay_mechanism,
                    waveform,
                }),
            })
        }
    }
}

/// After parsing `target :=`, determine which kind of variable assignment
/// follows. Returns a `VariableAssignmentStatement` with `label: None`
/// (the caller sets the label).
fn parse_variable_assignment_after_assign(
    parser: &mut Parser,
    target: Target,
) -> Result<VariableAssignmentStatement, ParseError> {
    // Parse expression; then check for WHEN (conditional).
    let expression = Expression::parse(parser)?;

    if parser.at_keyword(KeywordKind::When) {
        // Conditional variable assignment.
        // expression WHEN condition { ELSE expression WHEN condition } [ ELSE expression ]
        parser.expect_keyword(KeywordKind::When)?;
        let first_condition = Expression::parse(parser)?;
        let mut alternatives = vec![ConditionalAlternative {
            expression: Box::new(expression),
            condition: first_condition,
        }];
        let mut else_expression = None;
        while parser.consume_if_keyword(KeywordKind::Else).is_some() {
            let expr = Expression::parse(parser)?;
            if parser.consume_if_keyword(KeywordKind::When).is_some() {
                let cond = Expression::parse(parser)?;
                alternatives.push(ConditionalAlternative {
                    expression: Box::new(expr),
                    condition: cond,
                });
            } else {
                else_expression = Some(Box::new(expr));
                break;
            }
        }
        Ok(VariableAssignmentStatement::Conditional {
            label: None,
            assignment: ConditionalVariableAssignment {
                target,
                conditional_expressions: ConditionalExpressions {
                    alternatives,
                    else_expression,
                },
            },
        })
    } else {
        Ok(VariableAssignmentStatement::Simple {
            label: None,
            assignment: SimpleVariableAssignment { target, expression },
        })
    }
}

// ─── AstNode implementations ────────────────────────────────────────────

impl AstNode for SequentialStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Check for labeled statement: Identifier followed by ':'
        let label = if (parser.at(TokenKind::Identifier)
            || parser.at(TokenKind::ExtendedIdentifier))
            && parser
                .peek_nth(1)
                .is_some_and(|t| t.kind == TokenKind::Colon)
        {
            let lbl = Label::parse(parser)?;
            parser.expect(TokenKind::Colon)?;
            Some(lbl)
        } else {
            None
        };

        // Dispatch based on the current token.
        let stmt = if parser.at_keyword(KeywordKind::Wait) {
            let mut s = WaitStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Wait(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Assert) {
            let mut s = AssertionStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Assertion(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Report) {
            let mut s = ReportStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Report(Box::new(s))
        } else if parser.at_keyword(KeywordKind::If) {
            let mut s = IfStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::If(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Case) {
            let mut s = CaseStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Case(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Loop)
            || parser.at_keyword(KeywordKind::While)
            || parser.at_keyword(KeywordKind::For)
        {
            let mut s = LoopStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Loop(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Next) {
            let mut s = NextStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Next(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Exit) {
            let mut s = ExitStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Exit(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Return) {
            let mut s = ReturnStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Return(Box::new(s))
        } else if parser.at_keyword(KeywordKind::Null) {
            let mut s = NullStatement::parse(parser)?;
            s.label = label.clone();
            SequentialStatement::Null(Box::new(s))
        } else if parser.at_keyword(KeywordKind::With) {
            // WITH expression SELECT [?] target <= ... or target := ...
            // Parse: WITH expression SELECT [?] target
            parser.expect_keyword(KeywordKind::With)?;
            let selector = Expression::parse(parser)?;
            parser.expect_keyword(KeywordKind::Select)?;
            let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
            let target = Target::parse(parser)?;

            if parser.at(TokenKind::LtEquals) {
                // Selected signal assignment.
                parser.expect(TokenKind::LtEquals)?;
                if parser.at_keyword(KeywordKind::Force) {
                    parser.expect_keyword(KeywordKind::Force)?;
                    let force_mode = parse_optional_force_mode(parser);
                    let selected_expressions = SelectedExpressions::parse(parser)?;
                    parser.expect(TokenKind::Semicolon)?;
                    SequentialStatement::SignalAssignment(Box::new(
                        SignalAssignmentStatement::Selected {
                            label: label.clone(),
                            assignment: SelectedSignalAssignment::Force(SelectedForceAssignment {
                                selector,
                                matching,
                                target,
                                force_mode,
                                selected_expressions,
                            }),
                        },
                    ))
                } else {
                    let delay_mechanism = parse_optional_delay_mechanism(parser)?;
                    let selected_waveforms = SelectedWaveforms::parse(parser)?;
                    parser.expect(TokenKind::Semicolon)?;
                    SequentialStatement::SignalAssignment(Box::new(
                        SignalAssignmentStatement::Selected {
                            label: label.clone(),
                            assignment: SelectedSignalAssignment::Waveform(
                                SelectedWaveformAssignment {
                                    selector,
                                    matching,
                                    target,
                                    delay_mechanism,
                                    selected_waveforms,
                                },
                            ),
                        },
                    ))
                }
            } else if parser.at(TokenKind::VarAssign) {
                // Selected variable assignment.
                parser.expect(TokenKind::VarAssign)?;
                let selected_expressions = SelectedExpressions::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                SequentialStatement::VariableAssignment(Box::new(
                    VariableAssignmentStatement::Selected {
                        label: label.clone(),
                        assignment: SelectedVariableAssignment {
                            selector,
                            matching,
                            target,
                            selected_expressions,
                        },
                    },
                ))
            } else {
                return Err(parser.error("expected '<=' or ':=' after selected target"));
            }
        } else {
            // Must be a signal assignment, variable assignment, or procedure call.
            // Parse name/target, then look at what follows.
            let target = Target::parse(parser)?;

            if parser.at(TokenKind::LtEquals) {
                // Signal assignment.
                parser.expect(TokenKind::LtEquals)?;
                let mut result = parse_signal_assignment_after_arrow(parser, target)?;
                // Set label on the result.
                match &mut result {
                    SignalAssignmentStatement::Simple { label: lbl, .. }
                    | SignalAssignmentStatement::Conditional { label: lbl, .. }
                    | SignalAssignmentStatement::Selected { label: lbl, .. } => {
                        *lbl = label.clone();
                    }
                }
                parser.expect(TokenKind::Semicolon)?;
                SequentialStatement::SignalAssignment(Box::new(result))
            } else if parser.at(TokenKind::VarAssign) {
                // Variable assignment.
                parser.expect(TokenKind::VarAssign)?;
                let mut result = parse_variable_assignment_after_assign(parser, target)?;
                // Set label on the result.
                match &mut result {
                    VariableAssignmentStatement::Simple { label: lbl, .. }
                    | VariableAssignmentStatement::Conditional { label: lbl, .. }
                    | VariableAssignmentStatement::Selected { label: lbl, .. } => {
                        *lbl = label.clone();
                    }
                }
                parser.expect(TokenKind::Semicolon)?;
                SequentialStatement::VariableAssignment(Box::new(result))
            } else {
                // Procedure call: the target was actually the procedure name.
                // Extract the Name from the Target.
                let procedure_name = match target {
                    Target::Name(name) => name,
                    Target::Aggregate(_) => {
                        return Err(parser.error(
                            "expected procedure call, signal assignment, or variable assignment",
                        ));
                    }
                };
                parser.expect(TokenKind::Semicolon)?;
                SequentialStatement::ProcedureCall(Box::new(ProcedureCallStatement {
                    label: label.clone(),
                    procedure_call: ProcedureCall {
                        procedure_name,
                        parameters: None,
                    },
                }))
            }
        };

        Ok(stmt)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Wait(s) => s.format(f, indent_level),
            Self::Assertion(s) => s.format(f, indent_level),
            Self::Report(s) => s.format(f, indent_level),
            Self::SignalAssignment(s) => s.format(f, indent_level),
            Self::VariableAssignment(s) => s.format(f, indent_level),
            Self::ProcedureCall(s) => s.format(f, indent_level),
            Self::If(s) => s.format(f, indent_level),
            Self::Case(s) => s.format(f, indent_level),
            Self::Loop(s) => s.format(f, indent_level),
            Self::Next(s) => s.format(f, indent_level),
            Self::Exit(s) => s.format(f, indent_level),
            Self::Return(s) => s.format(f, indent_level),
            Self::Null(s) => s.format(f, indent_level),
        }
    }
}

impl AstNode for SequenceOfStatements {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut statements = Vec::new();
        // Parse statements until a terminator keyword or EOF.
        while !parser.eof()
            && !parser.at_keyword(KeywordKind::End)
            && !parser.at_keyword(KeywordKind::Elsif)
            && !parser.at_keyword(KeywordKind::Else)
            && !parser.at_keyword(KeywordKind::When)
        {
            statements.push(SequentialStatement::parse(parser)?);
        }
        Ok(SequenceOfStatements { statements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for WaitStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse; here we start at WAIT.
        parser.expect_keyword(KeywordKind::Wait)?;

        let sensitivity_clause = if parser.at_keyword(KeywordKind::On) {
            Some(SensitivityClause::parse(parser)?)
        } else {
            None
        };

        let condition_clause = if parser.at_keyword(KeywordKind::Until) {
            Some(ConditionClause::parse(parser)?)
        } else {
            None
        };

        let timeout_clause = if parser.at_keyword(KeywordKind::For) {
            Some(TimeoutClause::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;

        Ok(WaitStatement {
            label: None,
            sensitivity_clause,
            condition_clause,
            timeout_clause,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "wait")?;
        if let Some(sensitivity) = &self.sensitivity_clause {
            write!(f, " ")?;
            sensitivity.format(f, indent_level)?;
        }
        if let Some(condition) = &self.condition_clause {
            write!(f, " ")?;
            condition.format(f, indent_level)?;
        }
        if let Some(timeout) = &self.timeout_clause {
            write!(f, " ")?;
            timeout.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SensitivityClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::On)?;
        let sensitivity_list = SensitivityList::parse(parser)?;
        Ok(SensitivityClause { sensitivity_list })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "on ")?;
        self.sensitivity_list.format(f, indent_level)
    }
}

impl AstNode for SensitivityList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut signals = vec![Name::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            signals.push(Name::parse(parser)?);
        }
        Ok(SensitivityList { signals })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.signals, f, indent_level)
    }
}

impl AstNode for ConditionClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Until)?;
        let condition = Expression::parse(parser)?;
        Ok(ConditionClause { condition })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "until ")?;
        self.condition.format(f, indent_level)
    }
}

impl AstNode for TimeoutClause {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::For)?;
        let time_expression = Expression::parse(parser)?;
        Ok(TimeoutClause { time_expression })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "for ")?;
        self.time_expression.format(f, indent_level)
    }
}

impl AstNode for Assertion {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Assert)?;
        let condition = Expression::parse(parser)?;

        let report = if parser.consume_if_keyword(KeywordKind::Report).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        let severity = if parser.consume_if_keyword(KeywordKind::Severity).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        Ok(Assertion {
            condition,
            report,
            severity,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "assert ")?;
        self.condition.format(f, indent_level)?;
        if let Some(report) = &self.report {
            write!(f, " report ")?;
            report.format(f, indent_level)?;
        }
        if let Some(severity) = &self.severity {
            write!(f, " severity ")?;
            severity.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for AssertionStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        let assertion = Assertion::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(AssertionStatement {
            label: None,
            assertion,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.assertion.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for ReportStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Report)?;
        let report_expression = Expression::parse(parser)?;

        let severity = if parser.consume_if_keyword(KeywordKind::Severity).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;

        Ok(ReportStatement {
            label: None,
            report_expression,
            severity,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "report ")?;
        self.report_expression.format(f, indent_level)?;
        if let Some(severity) = &self.severity {
            write!(f, " severity ")?;
            severity.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for SignalAssignmentStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // This standalone parse handles:
        //   target <= ...  (simple/conditional)
        //   WITH expr SELECT [?] target <= ...  (selected)
        if parser.at_keyword(KeywordKind::With) {
            let assignment = SelectedSignalAssignment::parse(parser)?;
            parser.expect(TokenKind::Semicolon)?;
            Ok(SignalAssignmentStatement::Selected {
                label: None,
                assignment,
            })
        } else {
            let target = Target::parse(parser)?;
            parser.expect(TokenKind::LtEquals)?;
            let result = parse_signal_assignment_after_arrow(parser, target)?;
            parser.expect(TokenKind::Semicolon)?;
            Ok(result)
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Simple { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
            Self::Conditional { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
            Self::Selected { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
        }
    }
}

impl AstNode for SimpleSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Target and <= have already been consumed by the caller in the
        // SignalAssignmentStatement flow. This standalone parse is provided
        // for completeness but in practice the dispatching happens inside
        // parse_signal_assignment_after_target.
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;

        if parser.at_keyword(KeywordKind::Force) {
            parser.expect_keyword(KeywordKind::Force)?;
            let force_mode = parse_optional_force_mode(parser);
            let expression = Expression::parse(parser)?;
            Ok(SimpleSignalAssignment::Force(SimpleForceAssignment {
                target,
                force_mode,
                expression,
            }))
        } else if parser.at_keyword(KeywordKind::Release) {
            parser.expect_keyword(KeywordKind::Release)?;
            let force_mode = parse_optional_force_mode(parser);
            Ok(SimpleSignalAssignment::Release(SimpleReleaseAssignment {
                target,
                force_mode,
            }))
        } else {
            let delay_mechanism = parse_optional_delay_mechanism(parser)?;
            let waveform = Waveform::parse(parser)?;
            Ok(SimpleSignalAssignment::Waveform(SimpleWaveformAssignment {
                target,
                delay_mechanism,
                waveform,
            }))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Waveform(a) => a.format(f, indent_level),
            Self::Force(a) => a.format(f, indent_level),
            Self::Release(a) => a.format(f, indent_level),
        }
    }
}

impl AstNode for SimpleWaveformAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        let delay_mechanism = parse_optional_delay_mechanism(parser)?;
        let waveform = Waveform::parse(parser)?;
        Ok(SimpleWaveformAssignment {
            target,
            delay_mechanism,
            waveform,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= ")?;
        if let Some(delay) = &self.delay_mechanism {
            delay.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.waveform.format(f, indent_level)
    }
}

impl AstNode for SimpleForceAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        parser.expect_keyword(KeywordKind::Force)?;
        let force_mode = parse_optional_force_mode(parser);
        let expression = Expression::parse(parser)?;
        Ok(SimpleForceAssignment {
            target,
            force_mode,
            expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= force ")?;
        if let Some(mode) = &self.force_mode {
            mode.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.expression.format(f, indent_level)
    }
}

impl AstNode for SimpleReleaseAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        parser.expect_keyword(KeywordKind::Release)?;
        let force_mode = parse_optional_force_mode(parser);
        Ok(SimpleReleaseAssignment { target, force_mode })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= release")?;
        if let Some(mode) = &self.force_mode {
            write!(f, " ")?;
            mode.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ConditionalSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // In practice, dispatching happens in parse_signal_assignment_after_target.
        // This standalone parse is provided for completeness.
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;

        if parser.at_keyword(KeywordKind::Force) {
            parser.expect_keyword(KeywordKind::Force)?;
            let force_mode = parse_optional_force_mode(parser);
            let conditional_expressions = ConditionalExpressions::parse(parser)?;
            Ok(ConditionalSignalAssignment::Force(
                ConditionalForceAssignment {
                    target,
                    force_mode,
                    conditional_expressions,
                },
            ))
        } else {
            let delay_mechanism = parse_optional_delay_mechanism(parser)?;
            let conditional_waveforms = ConditionalWaveforms::parse(parser)?;
            Ok(ConditionalSignalAssignment::Waveform(
                ConditionalWaveformAssignment {
                    target,
                    delay_mechanism,
                    conditional_waveforms,
                },
            ))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Waveform(a) => a.format(f, indent_level),
            Self::Force(a) => a.format(f, indent_level),
        }
    }
}

impl AstNode for ConditionalWaveformAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        let delay_mechanism = parse_optional_delay_mechanism(parser)?;
        let conditional_waveforms = ConditionalWaveforms::parse(parser)?;
        Ok(ConditionalWaveformAssignment {
            target,
            delay_mechanism,
            conditional_waveforms,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= ")?;
        if let Some(delay) = &self.delay_mechanism {
            delay.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.conditional_waveforms.format(f, indent_level)
    }
}

impl AstNode for ConditionalForceAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        parser.expect_keyword(KeywordKind::Force)?;
        let force_mode = parse_optional_force_mode(parser);
        let conditional_expressions = ConditionalExpressions::parse(parser)?;
        Ok(ConditionalForceAssignment {
            target,
            force_mode,
            conditional_expressions,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= force ")?;
        if let Some(mode) = &self.force_mode {
            mode.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.conditional_expressions.format(f, indent_level)
    }
}

impl AstNode for SelectedSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // WITH expression SELECT [?] target <= ...
        parser.expect_keyword(KeywordKind::With)?;
        let selector = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Select)?;
        let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;

        if parser.at_keyword(KeywordKind::Force) {
            parser.expect_keyword(KeywordKind::Force)?;
            let force_mode = parse_optional_force_mode(parser);
            let selected_expressions = SelectedExpressions::parse(parser)?;
            Ok(SelectedSignalAssignment::Force(SelectedForceAssignment {
                selector,
                matching,
                target,
                force_mode,
                selected_expressions,
            }))
        } else {
            let delay_mechanism = parse_optional_delay_mechanism(parser)?;
            let selected_waveforms = SelectedWaveforms::parse(parser)?;
            Ok(SelectedSignalAssignment::Waveform(
                SelectedWaveformAssignment {
                    selector,
                    matching,
                    target,
                    delay_mechanism,
                    selected_waveforms,
                },
            ))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Waveform(a) => a.format(f, indent_level),
            Self::Force(a) => a.format(f, indent_level),
        }
    }
}

impl AstNode for SelectedWaveformAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::With)?;
        let selector = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Select)?;
        let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        let delay_mechanism = parse_optional_delay_mechanism(parser)?;
        let selected_waveforms = SelectedWaveforms::parse(parser)?;
        Ok(SelectedWaveformAssignment {
            selector,
            matching,
            target,
            delay_mechanism,
            selected_waveforms,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "with ")?;
        self.selector.format(f, indent_level)?;
        write!(f, " select")?;
        if self.matching {
            write!(f, " ?")?;
        }
        write!(f, " ")?;
        self.target.format(f, indent_level)?;
        write!(f, " <= ")?;
        if let Some(delay) = &self.delay_mechanism {
            delay.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.selected_waveforms.format(f, indent_level)
    }
}

impl AstNode for SelectedForceAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::With)?;
        let selector = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Select)?;
        let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        parser.expect_keyword(KeywordKind::Force)?;
        let force_mode = parse_optional_force_mode(parser);
        let selected_expressions = SelectedExpressions::parse(parser)?;
        Ok(SelectedForceAssignment {
            selector,
            matching,
            target,
            force_mode,
            selected_expressions,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "with ")?;
        self.selector.format(f, indent_level)?;
        write!(f, " select")?;
        if self.matching {
            write!(f, " ?")?;
        }
        write!(f, " ")?;
        self.target.format(f, indent_level)?;
        write!(f, " <= force ")?;
        if let Some(mode) = &self.force_mode {
            mode.format(f, indent_level)?;
            write!(f, " ")?;
        }
        self.selected_expressions.format(f, indent_level)
    }
}

impl AstNode for DelayMechanism {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Transport).is_some() {
            Ok(DelayMechanism::Transport)
        } else if parser.at_keyword(KeywordKind::Reject) {
            parser.expect_keyword(KeywordKind::Reject)?;
            let reject_time = Expression::parse(parser)?;
            parser.expect_keyword(KeywordKind::Inertial)?;
            Ok(DelayMechanism::Inertial {
                reject_time: Some(reject_time),
            })
        } else if parser.consume_if_keyword(KeywordKind::Inertial).is_some() {
            Ok(DelayMechanism::Inertial { reject_time: None })
        } else {
            Err(parser.error("expected delay mechanism (transport, inertial, or reject)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Transport => write!(f, "transport"),
            Self::Inertial { reject_time } => {
                if let Some(time) = reject_time {
                    write!(f, "reject ")?;
                    time.format(f, indent_level)?;
                    write!(f, " ")?;
                }
                write!(f, "inertial")
            }
        }
    }
}

impl AstNode for Waveform {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Unaffected).is_some() {
            Ok(Waveform::Unaffected)
        } else {
            let mut elements = vec![WaveformElement::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                elements.push(WaveformElement::parse(parser)?);
            }
            Ok(Waveform::Elements(elements))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Elements(elements) => format_comma_separated(elements, f, indent_level),
            Self::Unaffected => write!(f, "unaffected"),
        }
    }
}

impl AstNode for WaveformElement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Null).is_some() {
            let after = if parser.consume_if_keyword(KeywordKind::After).is_some() {
                Some(Expression::parse(parser)?)
            } else {
                None
            };
            Ok(WaveformElement::Null { after })
        } else {
            let expression = Expression::parse(parser)?;
            let after = if parser.consume_if_keyword(KeywordKind::After).is_some() {
                Some(Expression::parse(parser)?)
            } else {
                None
            };
            Ok(WaveformElement::Value { expression, after })
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Value { expression, after } => {
                expression.format(f, indent_level)?;
                if let Some(after) = after {
                    write!(f, " after ")?;
                    after.format(f, indent_level)?;
                }
                Ok(())
            }
            Self::Null { after } => {
                write!(f, "null")?;
                if let Some(after) = after {
                    write!(f, " after ")?;
                    after.format(f, indent_level)?;
                }
                Ok(())
            }
        }
    }
}

impl AstNode for ConditionalWaveforms {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // conditional_waveforms ::= waveform WHEN condition
        //     { ELSE waveform WHEN condition } [ ELSE waveform ]
        let first = ConditionalWaveformAlternative::parse(parser)?;
        let mut alternatives = vec![first];
        let mut else_waveform = None;

        while parser.consume_if_keyword(KeywordKind::Else).is_some() {
            // Could be another conditional alternative or the final else waveform.
            let waveform = Waveform::parse(parser)?;
            if parser.consume_if_keyword(KeywordKind::When).is_some() {
                let condition = Expression::parse(parser)?;
                alternatives.push(ConditionalWaveformAlternative {
                    waveform,
                    condition,
                });
            } else {
                // Final else waveform (no WHEN).
                else_waveform = Some(waveform);
                break;
            }
        }

        Ok(ConditionalWaveforms {
            alternatives,
            else_waveform,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, alt) in self.alternatives.iter().enumerate() {
            if i > 0 {
                write!(f, " else ")?;
            }
            alt.format(f, indent_level)?;
        }
        if let Some(else_wf) = &self.else_waveform {
            write!(f, " else ")?;
            else_wf.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ConditionalWaveformAlternative {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let waveform = Waveform::parse(parser)?;
        parser.expect_keyword(KeywordKind::When)?;
        let condition = Expression::parse(parser)?;
        Ok(ConditionalWaveformAlternative {
            waveform,
            condition,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.waveform.format(f, indent_level)?;
        write!(f, " when ")?;
        self.condition.format(f, indent_level)
    }
}

impl AstNode for SelectedWaveforms {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // selected_waveforms ::= { waveform WHEN choices , } waveform WHEN choices
        let mut alternatives = vec![SelectedWaveformAlternative::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            alternatives.push(SelectedWaveformAlternative::parse(parser)?);
        }
        Ok(SelectedWaveforms { alternatives })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.alternatives, f, indent_level)
    }
}

impl AstNode for SelectedWaveformAlternative {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let waveform = Waveform::parse(parser)?;
        parser.expect_keyword(KeywordKind::When)?;
        let choices = Choices::parse(parser)?;
        Ok(SelectedWaveformAlternative { waveform, choices })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.waveform.format(f, indent_level)?;
        write!(f, " when ")?;
        self.choices.format(f, indent_level)
    }
}

impl AstNode for ForceMode {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::In).is_some() {
            Ok(ForceMode::In)
        } else if parser.consume_if_keyword(KeywordKind::Out).is_some() {
            Ok(ForceMode::Out)
        } else {
            Err(parser.error("expected force mode (in or out)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
        }
    }
}

impl AstNode for Target {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.at(TokenKind::LeftParen) {
            let aggregate = Aggregate::parse(parser)?;
            Ok(Target::Aggregate(aggregate))
        } else {
            let name = Name::parse(parser)?;
            Ok(Target::Name(name))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Name(name) => name.format(f, indent_level),
            Self::Aggregate(agg) => agg.format(f, indent_level),
        }
    }
}

impl AstNode for VariableAssignmentStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // This standalone parse handles:
        //   target := ...  (simple/conditional)
        //   WITH expr SELECT [?] target := ...  (selected)
        if parser.at_keyword(KeywordKind::With) {
            let assignment = SelectedVariableAssignment::parse(parser)?;
            parser.expect(TokenKind::Semicolon)?;
            Ok(VariableAssignmentStatement::Selected {
                label: None,
                assignment,
            })
        } else {
            let target = Target::parse(parser)?;
            parser.expect(TokenKind::VarAssign)?;
            let result = parse_variable_assignment_after_assign(parser, target)?;
            parser.expect(TokenKind::Semicolon)?;
            Ok(result)
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Simple { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
            Self::Conditional { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
            Self::Selected { label, assignment } => {
                write_indent(f, indent_level)?;
                if let Some(label) = label {
                    label.format(f, indent_level)?;
                    write!(f, " : ")?;
                }
                assignment.format(f, indent_level)?;
                writeln!(f, ";")
            }
        }
    }
}

impl AstNode for SimpleVariableAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::VarAssign)?;
        let expression = Expression::parse(parser)?;
        Ok(SimpleVariableAssignment { target, expression })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " := ")?;
        self.expression.format(f, indent_level)
    }
}

impl AstNode for ConditionalVariableAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::VarAssign)?;
        let conditional_expressions = ConditionalExpressions::parse(parser)?;
        Ok(ConditionalVariableAssignment {
            target,
            conditional_expressions,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " := ")?;
        self.conditional_expressions.format(f, indent_level)
    }
}

impl AstNode for SelectedVariableAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::With)?;
        let selector = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Select)?;
        let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::VarAssign)?;
        let selected_expressions = SelectedExpressions::parse(parser)?;
        Ok(SelectedVariableAssignment {
            selector,
            matching,
            target,
            selected_expressions,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "with ")?;
        self.selector.format(f, indent_level)?;
        write!(f, " select")?;
        if self.matching {
            write!(f, " ?")?;
        }
        write!(f, " ")?;
        self.target.format(f, indent_level)?;
        write!(f, " := ")?;
        self.selected_expressions.format(f, indent_level)
    }
}

impl AstNode for ProcedureCall {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // procedure_call ::= procedure_name [ ( actual_parameter_part ) ]
        // Name::parse already handles suffixed forms including (args).
        // We parse the name which may absorb the parenthesized arguments.
        let procedure_name = Name::parse(parser)?;
        Ok(ProcedureCall {
            procedure_name,
            parameters: None,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.procedure_name.format(f, indent_level)?;
        if let Some(params) = &self.parameters {
            write!(f, "(")?;
            format_comma_separated(&params.elements, f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for ProcedureCallStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let procedure_call = ProcedureCall::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ProcedureCallStatement {
            label: None,
            procedure_call,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        self.procedure_call.format(f, indent_level)?;
        writeln!(f, ";")
    }
}

impl AstNode for IfStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::If)?;
        let condition = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Then)?;
        let then_statements = SequenceOfStatements::parse(parser)?;

        let mut elsif_branches = Vec::new();
        while parser.at_keyword(KeywordKind::Elsif) {
            elsif_branches.push(ElsifBranch::parse(parser)?);
        }

        let else_statements = if parser.consume_if_keyword(KeywordKind::Else).is_some() {
            Some(SequenceOfStatements::parse(parser)?)
        } else {
            None
        };

        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::If)?;

        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };

        parser.expect(TokenKind::Semicolon)?;

        Ok(IfStatement {
            label: None,
            condition,
            then_statements,
            elsif_branches,
            else_statements,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "if ")?;
        self.condition.format(f, indent_level)?;
        writeln!(f, " then")?;
        self.then_statements.format(f, indent_level + 1)?;
        for branch in &self.elsif_branches {
            write_indent(f, indent_level)?;
            write!(f, "elsif ")?;
            branch.condition.format(f, indent_level)?;
            writeln!(f, " then")?;
            branch.statements.format(f, indent_level + 1)?;
        }
        if let Some(else_stmts) = &self.else_statements {
            write_indent(f, indent_level)?;
            writeln!(f, "else")?;
            else_stmts.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end if")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ElsifBranch {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Elsif)?;
        let condition = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Then)?;
        let statements = SequenceOfStatements::parse(parser)?;
        Ok(ElsifBranch {
            condition,
            statements,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "elsif ")?;
        self.condition.format(f, indent_level)?;
        writeln!(f, " then")?;
        self.statements.format(f, indent_level + 1)
    }
}

impl AstNode for CaseStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Case)?;
        let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Is)?;

        let mut alternatives = Vec::new();
        while parser.at_keyword(KeywordKind::When) {
            alternatives.push(CaseStatementAlternative::parse(parser)?);
        }

        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Case)?;

        // Consume matching `?` at end if present.
        if matching {
            parser.consume_if(TokenKind::QuestionMark);
        }

        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };

        parser.expect(TokenKind::Semicolon)?;

        Ok(CaseStatement {
            label: None,
            matching,
            expression,
            alternatives,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "case")?;
        if self.matching {
            write!(f, " ?")?;
        }
        write!(f, " ")?;
        self.expression.format(f, indent_level)?;
        writeln!(f, " is")?;
        for alt in &self.alternatives {
            alt.format(f, indent_level + 1)?;
        }
        write_indent(f, indent_level)?;
        write!(f, "end case")?;
        if self.matching {
            write!(f, " ?")?;
        }
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for CaseStatementAlternative {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::When)?;
        let choices = Choices::parse(parser)?;
        parser.expect(TokenKind::Arrow)?;
        let statements = SequenceOfStatements::parse(parser)?;
        Ok(CaseStatementAlternative {
            choices,
            statements,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "when ")?;
        self.choices.format(f, indent_level)?;
        writeln!(f, " =>")?;
        self.statements.format(f, indent_level + 1)
    }
}

impl AstNode for LoopStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        // Parse optional iteration scheme.
        let iteration_scheme =
            if parser.at_keyword(KeywordKind::While) || parser.at_keyword(KeywordKind::For) {
                Some(IterationScheme::parse(parser)?)
            } else {
                None
            };

        parser.expect_keyword(KeywordKind::Loop)?;
        let statements = SequenceOfStatements::parse(parser)?;
        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Loop)?;

        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };

        parser.expect(TokenKind::Semicolon)?;

        Ok(LoopStatement {
            label: None,
            iteration_scheme,
            statements,
            end_label,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        if let Some(scheme) = &self.iteration_scheme {
            scheme.format(f, indent_level)?;
            write!(f, " ")?;
        }
        writeln!(f, "loop")?;
        self.statements.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end loop")?;
        if let Some(end_label) = &self.end_label {
            write!(f, " ")?;
            end_label.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for IterationScheme {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::While).is_some() {
            let condition = Expression::parse(parser)?;
            Ok(IterationScheme::While(condition))
        } else if parser.consume_if_keyword(KeywordKind::For).is_some() {
            let spec = ParameterSpecification::parse(parser)?;
            Ok(IterationScheme::For(Box::new(spec)))
        } else {
            Err(parser.error("expected iteration scheme (while or for)"))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::While(cond) => {
                write!(f, "while ")?;
                cond.format(f, indent_level)
            }
            Self::For(spec) => {
                write!(f, "for ")?;
                spec.format(f, indent_level)
            }
        }
    }
}

impl AstNode for ParameterSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let identifier = Identifier::parse(parser)?;
        parser.expect_keyword(KeywordKind::In)?;
        let discrete_range = DiscreteRange::parse(parser)?;
        Ok(ParameterSpecification {
            identifier,
            discrete_range,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.identifier.format(f, indent_level)?;
        write!(f, " in ")?;
        self.discrete_range.format(f, indent_level)
    }
}

impl AstNode for NextStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Next)?;

        // Optional loop label (identifier that is NOT the keyword WHEN).
        let loop_label = if (parser.at(TokenKind::Identifier)
            || parser.at(TokenKind::ExtendedIdentifier))
            && !parser.at_keyword(KeywordKind::When)
        {
            Some(Label::parse(parser)?)
        } else {
            None
        };

        let condition = if parser.consume_if_keyword(KeywordKind::When).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;

        Ok(NextStatement {
            label: None,
            loop_label,
            condition,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "next")?;
        if let Some(loop_label) = &self.loop_label {
            write!(f, " ")?;
            loop_label.format(f, indent_level)?;
        }
        if let Some(condition) = &self.condition {
            write!(f, " when ")?;
            condition.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ExitStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Exit)?;

        // Optional loop label (identifier that is NOT the keyword WHEN).
        let loop_label = if (parser.at(TokenKind::Identifier)
            || parser.at(TokenKind::ExtendedIdentifier))
            && !parser.at_keyword(KeywordKind::When)
        {
            Some(Label::parse(parser)?)
        } else {
            None
        };

        let condition = if parser.consume_if_keyword(KeywordKind::When).is_some() {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;

        Ok(ExitStatement {
            label: None,
            loop_label,
            condition,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "exit")?;
        if let Some(loop_label) = &self.loop_label {
            write!(f, " ")?;
            loop_label.format(f, indent_level)?;
        }
        if let Some(condition) = &self.condition {
            write!(f, " when ")?;
            condition.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ReturnStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Return)?;

        let expression = if !parser.at(TokenKind::Semicolon) {
            Some(Expression::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Semicolon)?;

        Ok(ReturnStatement {
            label: None,
            expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        write!(f, "return")?;
        if let Some(expr) = &self.expression {
            write!(f, " ")?;
            expr.format(f, indent_level)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for NullStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Label is handled by SequentialStatement::parse.
        parser.expect_keyword(KeywordKind::Null)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(NullStatement { label: None })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(label) = &self.label {
            label.format(f, indent_level)?;
            write!(f, " : ")?;
        }
        writeln!(f, "null;")
    }
}
