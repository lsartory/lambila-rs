//! Sequential statement AST nodes.

use super::common::*;
use super::expression::*;
use super::name::Name;
use super::type_def::DiscreteRange;

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
    For(ParameterSpecification),
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
