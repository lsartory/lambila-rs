//! Concurrent statement AST nodes.

use super::common::*;
use super::expression::{Condition, Expression};
use super::node::{AstNode, format_lines, write_indent};
use super::sequential::*;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF (VHDL-2008): `concurrent_statement ::= block_statement | process_statement
///     | concurrent_procedure_call_statement | concurrent_assertion_statement
///     | concurrent_signal_assignment_statement | component_instantiation_statement
///     | generate_statement | PSL_PSL_Directive`
/// VHDL-87/93 omit PSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcurrentStatement {
    Block(Box<BlockStatement>),
    Process(Box<ProcessStatement>),
    ProcedureCall(Box<ConcurrentProcedureCallStatement>),
    Assertion(Box<ConcurrentAssertionStatement>),
    SignalAssignment(Box<ConcurrentSignalAssignmentStatement>),
    ComponentInstantiation(Box<super::component::ComponentInstantiationStatement>),
    Generate(Box<super::generate::GenerateStatement>),
}

/// EBNF (VHDL-2008): `block_statement ::= block_label : BLOCK [ ( guard_condition ) ] [ IS ]
///     block_header block_declarative_part BEGIN block_statement_part
///     END BLOCK [ block_label ] ;`
/// EBNF (VHDL-87): no `[ IS ]`, no `[ block_label ]` at end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStatement {
    pub label: Label,
    pub guard_condition: Option<Condition>,
    pub header: BlockHeader,
    pub declarative_part: BlockDeclarativePart,
    pub statement_part: BlockStatementPart,
    pub end_label: Option<Label>,
}

/// EBNF: `block_header ::= [ generic_clause [ generic_map_aspect ; ] ]
///     [ port_clause [ port_map_aspect ; ] ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub generic_clause: Option<super::interface::GenericClause>,
    pub generic_map_aspect: Option<super::interface::GenericMapAspect>,
    pub port_clause: Option<super::interface::PortClause>,
    pub port_map_aspect: Option<super::interface::PortMapAspect>,
}

/// EBNF: `block_declarative_part ::= { block_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDeclarativePart {
    pub items: Vec<BlockDeclarativeItem>,
}

/// EBNF (VHDL-2008): `block_declarative_item ::= subprogram_declaration | subprogram_body
///     | subprogram_instantiation_declaration | package_declaration | package_body
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | signal_declaration | shared_variable_declaration
///     | file_declaration | alias_declaration | component_declaration
///     | attribute_declaration | attribute_specification | configuration_specification
///     | disconnection_specification | use_clause
///     | group_template_declaration | group_declaration
///     | PSL_Property_Declaration | PSL_Sequence_Declaration | PSL_Clock_Declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDeclarativeItem {
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
    ComponentDeclaration(Box<super::component::ComponentDeclaration>),
    AttributeDeclaration(Box<super::attribute::AttributeDeclaration>),
    AttributeSpecification(Box<super::attribute::AttributeSpecification>),
    ConfigurationSpecification(Box<super::configuration::ConfigurationSpecification>),
    DisconnectionSpecification(Box<super::signal::DisconnectionSpecification>),
    UseClause(super::context::UseClause),
    /// VHDL-93+.
    GroupTemplateDeclaration(Box<super::group::GroupTemplateDeclaration>),
    /// VHDL-93+.
    GroupDeclaration(Box<super::group::GroupDeclaration>),
}

/// EBNF: `block_statement_part ::= { concurrent_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStatementPart {
    pub statements: Vec<ConcurrentStatement>,
}

/// EBNF (VHDL-2008): `process_statement ::= [ process_label : ] [ POSTPONED ] PROCESS
///     [ ( process_sensitivity_list ) ] [ IS ] process_declarative_part BEGIN
///     process_statement_part END [ POSTPONED ] PROCESS [ process_label ] ;`
/// VHDL-87: no `[ POSTPONED ]`, no `[ IS ]`, no `[ process_label ]` at end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStatement {
    pub label: Option<Label>,
    /// VHDL-93+.
    pub postponed: bool,
    pub sensitivity_list: Option<ProcessSensitivityList>,
    pub declarative_part: ProcessDeclarativePart,
    pub statement_part: ProcessStatementPart,
    pub end_label: Option<Label>,
}

/// EBNF (VHDL-2008): `process_sensitivity_list ::= ALL | sensitivity_list`
/// EBNF (VHDL-87/93): just `sensitivity_list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessSensitivityList {
    /// VHDL-2008.
    All,
    List(SensitivityList),
}

/// EBNF: `process_declarative_part ::= { process_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDeclarativePart {
    pub items: Vec<ProcessDeclarativeItem>,
}

/// EBNF (VHDL-2008): `process_declarative_item ::= subprogram_declaration | subprogram_body
///     | subprogram_instantiation_declaration | package_declaration | package_body
///     | package_instantiation_declaration | type_declaration | subtype_declaration
///     | constant_declaration | variable_declaration | file_declaration | alias_declaration
///     | attribute_declaration | attribute_specification | use_clause
///     | group_template_declaration | group_declaration`
/// Earlier versions have fewer alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessDeclarativeItem {
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

/// EBNF: `process_statement_part ::= { sequential_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStatementPart {
    pub statements: Vec<SequentialStatement>,
}

/// EBNF (VHDL-93+): `concurrent_assertion_statement ::= [ label : ] [ POSTPONED ] assertion ;`
/// EBNF (VHDL-87): `concurrent_assertion_statement ::= [ label : ] assertion ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentAssertionStatement {
    pub label: Option<Label>,
    /// VHDL-93+.
    pub postponed: bool,
    pub assertion: Assertion,
}

/// EBNF (VHDL-93+): `concurrent_procedure_call_statement ::= [ label : ] [ POSTPONED ]
///     procedure_call ;`
/// EBNF (VHDL-87): `concurrent_procedure_call_statement ::= [ label : ] procedure_call ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentProcedureCallStatement {
    pub label: Option<Label>,
    /// VHDL-93+.
    pub postponed: bool,
    pub procedure_call: ProcedureCall,
}

/// EBNF (VHDL-2008): `concurrent_signal_assignment_statement ::=
///     [ label : ] [ POSTPONED ] concurrent_simple_signal_assignment
///     | [ label : ] [ POSTPONED ] concurrent_conditional_signal_assignment
///     | [ label : ] [ POSTPONED ] concurrent_selected_signal_assignment`
/// EBNF (VHDL-87/93): `...[ label : ] [ POSTPONED ] conditional_signal_assignment
///     | [ label : ] [ POSTPONED ] selected_signal_assignment`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcurrentSignalAssignmentStatement {
    /// VHDL-2008.
    Simple {
        label: Option<Label>,
        postponed: bool,
        assignment: ConcurrentSimpleSignalAssignment,
    },
    Conditional {
        label: Option<Label>,
        postponed: bool,
        assignment: ConcurrentConditionalSignalAssignment,
    },
    Selected {
        label: Option<Label>,
        postponed: bool,
        assignment: ConcurrentSelectedSignalAssignment,
    },
}

/// EBNF: `concurrent_simple_signal_assignment ::= target <= [ GUARDED ]
///     [ delay_mechanism ] waveform ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentSimpleSignalAssignment {
    pub target: Target,
    pub guarded: bool,
    pub delay_mechanism: Option<DelayMechanism>,
    pub waveform: Waveform,
}

/// EBNF: `concurrent_conditional_signal_assignment ::= target <= [ GUARDED ]
///     [ delay_mechanism ] conditional_waveforms ;` (VHDL-2008)
///
/// EBNF (VHDL-87/93): `conditional_signal_assignment ::= target <= options
///     conditional_waveforms ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentConditionalSignalAssignment {
    pub target: Target,
    pub guarded: bool,
    pub delay_mechanism: Option<DelayMechanism>,
    pub conditional_waveforms: ConditionalWaveforms,
}

/// EBNF (VHDL-2008): `concurrent_selected_signal_assignment ::= WITH expression SELECT [ ? ]
///     target <= [ GUARDED ] [ delay_mechanism ] selected_waveforms ;`
/// EBNF (VHDL-87/93): `selected_signal_assignment ::= WITH expression SELECT
///     target <= options selected_waveforms ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrentSelectedSignalAssignment {
    pub selector: Expression,
    /// VHDL-2008: matching select (`?`).
    pub matching: bool,
    pub target: Target,
    pub guarded: bool,
    pub delay_mechanism: Option<DelayMechanism>,
    pub selected_waveforms: SelectedWaveforms,
}

/// EBNF (VHDL-87/93): `options ::= [ GUARDED ] [ delay_mechanism ]`
/// Used in VHDL-87/93 concurrent signal assignments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub guarded: bool,
    pub delay_mechanism: Option<DelayMechanism>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parse a block statement after the label and colon have already been consumed.
/// Grammar: BLOCK [ ( guard_condition ) ] [ IS ] block_header block_declarative_part
///     BEGIN block_statement_part END BLOCK [ block_label ] ;
fn parse_block_statement_with_label(
    parser: &mut Parser,
    label: Label,
) -> Result<BlockStatement, ParseError> {
    parser.expect_keyword(KeywordKind::Block)?;

    // [ ( guard_condition ) ]
    let guard_condition = if parser.consume_if(TokenKind::LeftParen).is_some() {
        let cond = Condition::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Some(cond)
    } else {
        None
    };

    // [ IS ]
    parser.consume_if_keyword(KeywordKind::Is);

    let header = BlockHeader::parse(parser)?;
    let declarative_part = BlockDeclarativePart::parse(parser)?;
    parser.expect_keyword(KeywordKind::Begin)?;
    let statement_part = BlockStatementPart::parse(parser)?;
    parser.expect_keyword(KeywordKind::End)?;
    parser.expect_keyword(KeywordKind::Block)?;
    let end_label = if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier)
    {
        Some(Label::parse(parser)?)
    } else {
        None
    };
    parser.expect(TokenKind::Semicolon)?;
    Ok(BlockStatement {
        label,
        guard_condition,
        header,
        declarative_part,
        statement_part,
        end_label,
    })
}

/// Parse a process statement. The optional label has already been consumed if present.
/// Grammar: [ POSTPONED ] PROCESS [ ( sensitivity_list | ALL ) ] [ IS ]
///     process_declarative_part BEGIN process_statement_part
///     END [ POSTPONED ] PROCESS [ label ] ;
fn parse_process_with_label(
    parser: &mut Parser,
    label: Option<Label>,
) -> Result<ProcessStatement, ParseError> {
    let postponed = parser.consume_if_keyword(KeywordKind::Postponed).is_some();
    parser.expect_keyword(KeywordKind::Process)?;

    // [ ( sensitivity_list | ALL ) ]
    let sensitivity_list = if parser.consume_if(TokenKind::LeftParen).is_some() {
        let list = ProcessSensitivityList::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Some(list)
    } else {
        None
    };

    // [ IS ]
    parser.consume_if_keyword(KeywordKind::Is);

    let declarative_part = ProcessDeclarativePart::parse(parser)?;
    parser.expect_keyword(KeywordKind::Begin)?;
    let statement_part = ProcessStatementPart::parse(parser)?;
    parser.expect_keyword(KeywordKind::End)?;
    // [ POSTPONED ]
    parser.consume_if_keyword(KeywordKind::Postponed);
    parser.expect_keyword(KeywordKind::Process)?;
    let end_label = if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier)
    {
        Some(Label::parse(parser)?)
    } else {
        None
    };
    parser.expect(TokenKind::Semicolon)?;
    Ok(ProcessStatement {
        label,
        postponed,
        sensitivity_list,
        declarative_part,
        statement_part,
        end_label,
    })
}

/// Parse a selected signal assignment.
/// Grammar: WITH expression SELECT [?] target <= [GUARDED] [delay_mechanism] selected_waveforms ;
fn parse_selected_signal_assignment(
    parser: &mut Parser,
) -> Result<ConcurrentSelectedSignalAssignment, ParseError> {
    parser.expect_keyword(KeywordKind::With)?;
    let selector = Expression::parse(parser)?;
    parser.expect_keyword(KeywordKind::Select)?;
    let matching = parser.consume_if(TokenKind::QuestionMark).is_some();
    let target = Target::parse(parser)?;
    parser.expect(TokenKind::LtEquals)?;
    let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
    let delay_mechanism = try_parse_delay_mechanism(parser);
    let selected_waveforms = SelectedWaveforms::parse(parser)?;
    parser.expect(TokenKind::Semicolon)?;
    Ok(ConcurrentSelectedSignalAssignment {
        selector,
        matching,
        target,
        guarded,
        delay_mechanism,
        selected_waveforms,
    })
}

/// Try to parse a delay mechanism (TRANSPORT or [REJECT time] INERTIAL).
fn try_parse_delay_mechanism(parser: &mut Parser) -> Option<DelayMechanism> {
    if parser.consume_if_keyword(KeywordKind::Transport).is_some() {
        Some(DelayMechanism::Transport)
    } else if parser.at_keyword(KeywordKind::Reject) {
        parser.consume();
        let time = Expression::parse(parser).ok()?;
        parser.expect_keyword(KeywordKind::Inertial).ok()?;
        Some(DelayMechanism::Inertial {
            reject_time: Some(time),
        })
    } else if parser.consume_if_keyword(KeywordKind::Inertial).is_some() {
        Some(DelayMechanism::Inertial { reject_time: None })
    } else {
        None
    }
}

/// Parse a name-based concurrent statement (signal assignment, procedure call, or component instantiation).
/// Used when no distinguishing keyword precedes the statement.
/// The optional label and postponed flag are already determined.
fn parse_name_based_concurrent_statement(
    parser: &mut Parser,
    label: Option<Label>,
    postponed: bool,
) -> Result<ConcurrentStatement, ParseError> {
    // Parse the target/name
    let target = Target::parse(parser)?;

    if parser.at(TokenKind::LtEquals) {
        // Signal assignment: target <= ...
        parser.consume(); // consume <=
        let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
        let delay_mechanism = try_parse_delay_mechanism(parser);

        // Parse waveform, then check for WHEN to distinguish simple vs conditional
        let waveform = Waveform::parse(parser)?;

        if parser.at_keyword(KeywordKind::When) {
            // Conditional signal assignment: waveform WHEN condition [ELSE waveform WHEN condition]... [ELSE waveform]
            // We already parsed the first waveform. Now parse: WHEN condition { ELSE waveform WHEN condition } [ ELSE waveform ]
            parser.consume(); // consume WHEN
            let first_condition = Condition::parse(parser)?;
            let mut alternatives = vec![ConditionalWaveformAlternative {
                waveform,
                condition: first_condition,
            }];
            let mut else_waveform = None;
            while parser.consume_if_keyword(KeywordKind::Else).is_some() {
                let next_wf = Waveform::parse(parser)?;
                if parser.at_keyword(KeywordKind::When) {
                    parser.consume(); // consume WHEN
                    let cond = Condition::parse(parser)?;
                    alternatives.push(ConditionalWaveformAlternative {
                        waveform: next_wf,
                        condition: cond,
                    });
                } else {
                    // Final else waveform (no WHEN)
                    else_waveform = Some(next_wf);
                    break;
                }
            }
            parser.expect(TokenKind::Semicolon)?;
            Ok(ConcurrentStatement::SignalAssignment(Box::new(
                ConcurrentSignalAssignmentStatement::Conditional {
                    label,
                    postponed,
                    assignment: ConcurrentConditionalSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        conditional_waveforms: ConditionalWaveforms {
                            alternatives,
                            else_waveform,
                        },
                    },
                },
            )))
        } else {
            // Simple signal assignment
            parser.expect(TokenKind::Semicolon)?;
            Ok(ConcurrentStatement::SignalAssignment(Box::new(
                ConcurrentSignalAssignmentStatement::Simple {
                    label,
                    postponed,
                    assignment: ConcurrentSimpleSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        waveform,
                    },
                },
            )))
        }
    } else {
        // Procedure call: name [ ( parameters ) ] ;
        // The target is already parsed as a name or aggregate.
        // For a procedure call, we need a Name, not a Target.
        // Since Target::parse returns Target::Name(name) for identifiers,
        // extract the name. If it's an Aggregate, that's an error.
        match target {
            Target::Name(name) => {
                // Check if there's an actual parameter part (parenthesized)
                // Note: Name::parse already handles function_call syntax name(args),
                // so the name may already include the parameter part.
                // A procedure call is just: procedure_name [ ( params ) ] ;
                // The Name parser handles name(args) as FunctionCall or IndexedName.
                // For a procedure call, we treat the entire name as the call.
                parser.expect(TokenKind::Semicolon)?;
                Ok(ConcurrentStatement::ProcedureCall(Box::new(
                    ConcurrentProcedureCallStatement {
                        label,
                        postponed,
                        procedure_call: ProcedureCall {
                            procedure_name: name,
                            parameters: None,
                        },
                    },
                )))
            }
            Target::Aggregate(_) => {
                Err(parser.error("expected procedure call or signal assignment, found aggregate"))
            }
        }
    }
}

/// Parse a labeled name-based concurrent statement.
/// After label : has been consumed, we might see a name followed by
/// <= (signal assignment), or generic/port map (component instantiation),
/// or just ( or ; (procedure call).
fn parse_labeled_name_based(
    parser: &mut Parser,
    label: Label,
) -> Result<ConcurrentStatement, ParseError> {
    // Check if this could be a component instantiation:
    // After the label, if we see name followed by GENERIC MAP or PORT MAP,
    // it's a component instantiation (VHDL-87 style without COMPONENT keyword).
    let target = Target::parse(parser)?;

    if parser.at(TokenKind::LtEquals) {
        // Signal assignment
        parser.consume(); // consume <=
        let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
        let delay_mechanism = try_parse_delay_mechanism(parser);

        let waveform = Waveform::parse(parser)?;

        if parser.at_keyword(KeywordKind::When) {
            parser.consume();
            let first_condition = Condition::parse(parser)?;
            let mut alternatives = vec![ConditionalWaveformAlternative {
                waveform,
                condition: first_condition,
            }];
            let mut else_waveform = None;
            while parser.consume_if_keyword(KeywordKind::Else).is_some() {
                let next_wf = Waveform::parse(parser)?;
                if parser.at_keyword(KeywordKind::When) {
                    parser.consume();
                    let cond = Condition::parse(parser)?;
                    alternatives.push(ConditionalWaveformAlternative {
                        waveform: next_wf,
                        condition: cond,
                    });
                } else {
                    else_waveform = Some(next_wf);
                    break;
                }
            }
            parser.expect(TokenKind::Semicolon)?;
            Ok(ConcurrentStatement::SignalAssignment(Box::new(
                ConcurrentSignalAssignmentStatement::Conditional {
                    label: Some(label),
                    postponed: false,
                    assignment: ConcurrentConditionalSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        conditional_waveforms: ConditionalWaveforms {
                            alternatives,
                            else_waveform,
                        },
                    },
                },
            )))
        } else {
            parser.expect(TokenKind::Semicolon)?;
            Ok(ConcurrentStatement::SignalAssignment(Box::new(
                ConcurrentSignalAssignmentStatement::Simple {
                    label: Some(label),
                    postponed: false,
                    assignment: ConcurrentSimpleSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        waveform,
                    },
                },
            )))
        }
    } else if parser.at_keyword(KeywordKind::Generic) || parser.at_keyword(KeywordKind::Port) {
        // Component instantiation (VHDL-87 style: label : component_name generic/port map)
        match target {
            Target::Name(name) => {
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
                parser.expect(TokenKind::Semicolon)?;
                Ok(ConcurrentStatement::ComponentInstantiation(Box::new(
                    super::component::ComponentInstantiationStatement {
                        label,
                        unit: super::component::InstantiatedUnit::Component {
                            has_component_keyword: false,
                            name: Box::new(name),
                        },
                        generic_map_aspect,
                        port_map_aspect,
                    },
                )))
            }
            Target::Aggregate(_) => Err(parser.error("expected component name for instantiation")),
        }
    } else {
        // Procedure call: label : name ;
        match target {
            Target::Name(name) => {
                parser.expect(TokenKind::Semicolon)?;
                Ok(ConcurrentStatement::ProcedureCall(Box::new(
                    ConcurrentProcedureCallStatement {
                        label: Some(label),
                        postponed: false,
                        procedure_call: ProcedureCall {
                            procedure_name: name,
                            parameters: None,
                        },
                    },
                )))
            }
            Target::Aggregate(_) => {
                Err(parser.error("expected procedure call or signal assignment"))
            }
        }
    }
}

/// Parse a generate statement after the label and colon have been consumed.
fn parse_generate_with_label(
    parser: &mut Parser,
    label: Label,
) -> Result<super::generate::GenerateStatement, ParseError> {
    if parser.at_keyword(KeywordKind::Case) {
        parser.consume();
        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let mut alternatives = Vec::new();
        while parser.at_keyword(KeywordKind::When) {
            alternatives.push(super::generate::CaseGenerateAlternative::parse(parser)?);
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
        Ok(super::generate::GenerateStatement::Case(
            super::generate::CaseGenerateStatement {
                label,
                expression,
                alternatives,
                end_label,
            },
        ))
    } else {
        // FOR or IF -- use legacy form
        let scheme = super::generate::GenerationScheme::parse(parser)?;
        parser.expect_keyword(KeywordKind::Generate)?;

        // Parse body using the same logic as LegacyGenerateStatement
        let (declarative_part, statements) = parse_generate_body(parser)?;

        parser.expect_keyword(KeywordKind::End)?;
        parser.expect_keyword(KeywordKind::Generate)?;
        let end_label =
            if parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier) {
                Some(Label::parse(parser)?)
            } else {
                None
            };
        parser.expect(TokenKind::Semicolon)?;
        Ok(super::generate::GenerateStatement::Legacy(
            super::generate::LegacyGenerateStatement {
                label,
                scheme,
                declarative_part,
                statements,
                end_label,
            },
        ))
    }
}

/// Parse the body of a generate statement: optional declarative part + concurrent statements.
/// Same logic as parse_legacy_generate_body in generate.rs but local to this module.
fn parse_generate_body(
    parser: &mut Parser,
) -> Result<(Option<BlockDeclarativePart>, Vec<ConcurrentStatement>), ParseError> {
    if parser.at_keyword(KeywordKind::Begin) {
        parser.consume();
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        return Ok((Some(BlockDeclarativePart { items: vec![] }), statements));
    }

    // Try to detect declarative region: save, try declarative items, look for BEGIN.
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
        let item_saved = parser.save();
        match BlockDeclarativeItem::parse(parser) {
            Ok(item) => items.push(item),
            Err(_) => {
                parser.restore(item_saved);
                break;
            }
        }
    }

    if found_begin {
        let declarative_part = Some(BlockDeclarativePart { items });
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        Ok((declarative_part, statements))
    } else {
        parser.restore(saved);
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        Ok((None, statements))
    }
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ConcurrentStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Determine if there is an optional label prefix: identifier ':'
        let has_label = (parser.at(TokenKind::Identifier)
            || parser.at(TokenKind::ExtendedIdentifier))
            && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon);

        if has_label {
            // Parse label and colon
            let label = Label::parse(parser)?;
            parser.expect(TokenKind::Colon)?;

            // Dispatch based on the next keyword
            if parser.at_keyword(KeywordKind::Block) {
                let block = parse_block_statement_with_label(parser, label)?;
                return Ok(ConcurrentStatement::Block(Box::new(block)));
            }

            if parser.at_keyword(KeywordKind::Postponed) {
                // POSTPONED PROCESS or POSTPONED assertion/procedure_call
                if parser.peek_nth(1).map(|t| t.kind)
                    == Some(TokenKind::Keyword(KeywordKind::Process))
                {
                    let process = parse_process_with_label(parser, Some(label))?;
                    return Ok(ConcurrentStatement::Process(Box::new(process)));
                }
                // POSTPONED ASSERT
                if parser.peek_nth(1).map(|t| t.kind)
                    == Some(TokenKind::Keyword(KeywordKind::Assert))
                {
                    parser.consume(); // consume POSTPONED
                    let assertion = Assertion::parse(parser)?;
                    parser.expect(TokenKind::Semicolon)?;
                    return Ok(ConcurrentStatement::Assertion(Box::new(
                        ConcurrentAssertionStatement {
                            label: Some(label),
                            postponed: true,
                            assertion,
                        },
                    )));
                }
                // POSTPONED procedure_call or signal assignment
                parser.consume(); // consume POSTPONED
                return parse_name_based_concurrent_statement(parser, Some(label), true);
            }

            if parser.at_keyword(KeywordKind::Process) {
                let process = parse_process_with_label(parser, Some(label))?;
                return Ok(ConcurrentStatement::Process(Box::new(process)));
            }

            if parser.at_keyword(KeywordKind::Assert) {
                let assertion = Assertion::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                return Ok(ConcurrentStatement::Assertion(Box::new(
                    ConcurrentAssertionStatement {
                        label: Some(label),
                        postponed: false,
                        assertion,
                    },
                )));
            }

            if parser.at_keyword(KeywordKind::With) {
                let assignment = parse_selected_signal_assignment(parser)?;
                return Ok(ConcurrentStatement::SignalAssignment(Box::new(
                    ConcurrentSignalAssignmentStatement::Selected {
                        label: Some(label),
                        postponed: false,
                        assignment,
                    },
                )));
            }

            if parser.at_keyword(KeywordKind::Entity)
                || parser.at_keyword(KeywordKind::Configuration)
            {
                let unit = super::component::InstantiatedUnit::parse(parser)?;
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
                parser.expect(TokenKind::Semicolon)?;
                return Ok(ConcurrentStatement::ComponentInstantiation(Box::new(
                    super::component::ComponentInstantiationStatement {
                        label,
                        unit,
                        generic_map_aspect,
                        port_map_aspect,
                    },
                )));
            }

            if parser.at_keyword(KeywordKind::Component) {
                let unit = super::component::InstantiatedUnit::parse(parser)?;
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
                parser.expect(TokenKind::Semicolon)?;
                return Ok(ConcurrentStatement::ComponentInstantiation(Box::new(
                    super::component::ComponentInstantiationStatement {
                        label,
                        unit,
                        generic_map_aspect,
                        port_map_aspect,
                    },
                )));
            }

            if parser.at_keyword(KeywordKind::For)
                || parser.at_keyword(KeywordKind::If)
                || parser.at_keyword(KeywordKind::Case)
            {
                let generate = parse_generate_with_label(parser, label)?;
                return Ok(ConcurrentStatement::Generate(Box::new(generate)));
            }

            // Otherwise: name-based (signal assignment, procedure call, or component instantiation)
            return parse_labeled_name_based(parser, label);
        }

        // No label prefix
        if parser.at_keyword(KeywordKind::Postponed) {
            if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Process))
            {
                let process = parse_process_with_label(parser, None)?;
                return Ok(ConcurrentStatement::Process(Box::new(process)));
            }
            if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Assert)) {
                parser.consume(); // consume POSTPONED
                let assertion = Assertion::parse(parser)?;
                parser.expect(TokenKind::Semicolon)?;
                return Ok(ConcurrentStatement::Assertion(Box::new(
                    ConcurrentAssertionStatement {
                        label: None,
                        postponed: true,
                        assertion,
                    },
                )));
            }
            // POSTPONED procedure call or signal assignment
            parser.consume(); // consume POSTPONED
            return parse_name_based_concurrent_statement(parser, None, true);
        }

        if parser.at_keyword(KeywordKind::Process) {
            let process = parse_process_with_label(parser, None)?;
            return Ok(ConcurrentStatement::Process(Box::new(process)));
        }

        if parser.at_keyword(KeywordKind::Assert) {
            let assertion = Assertion::parse(parser)?;
            parser.expect(TokenKind::Semicolon)?;
            return Ok(ConcurrentStatement::Assertion(Box::new(
                ConcurrentAssertionStatement {
                    label: None,
                    postponed: false,
                    assertion,
                },
            )));
        }

        if parser.at_keyword(KeywordKind::With) {
            let assignment = parse_selected_signal_assignment(parser)?;
            return Ok(ConcurrentStatement::SignalAssignment(Box::new(
                ConcurrentSignalAssignmentStatement::Selected {
                    label: None,
                    postponed: false,
                    assignment,
                },
            )));
        }

        // Name-based: signal assignment or procedure call (no label, no postponed)
        parse_name_based_concurrent_statement(parser, None, false)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Block(inner) => inner.format(f, indent_level),
            Self::Process(inner) => inner.format(f, indent_level),
            Self::ProcedureCall(inner) => inner.format(f, indent_level),
            Self::Assertion(inner) => inner.format(f, indent_level),
            Self::SignalAssignment(inner) => inner.format(f, indent_level),
            Self::ComponentInstantiation(inner) => inner.format(f, indent_level),
            Self::Generate(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for BlockStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Full standalone parse: label : BLOCK [ ( guard_condition ) ] [ IS ]
        //     block_header block_declarative_part BEGIN block_statement_part
        //     END BLOCK [ label ] ;
        let label = Label::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        parse_block_statement_with_label(parser, label)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        self.label.format(f, 0)?;
        write!(f, " : block")?;
        if let Some(ref guard) = self.guard_condition {
            write!(f, " (")?;
            guard.format(f, 0)?;
            write!(f, ")")?;
        }
        writeln!(f, " is")?;
        self.header.format(f, indent_level + 1)?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "begin")?;
        self.statement_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end block")?;
        if let Some(ref end_label) = self.end_label {
            write!(f, " ")?;
            end_label.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for BlockHeader {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ generic_clause [ generic_map_aspect ; ] ]
        // [ port_clause [ port_map_aspect ; ] ]
        let generic_clause = if parser.at_keyword(KeywordKind::Generic) {
            // Check if this is GENERIC ( (generic clause) or GENERIC MAP (generic map aspect)
            // Generic clause starts with GENERIC (
            // Generic map aspect starts with GENERIC MAP (
            if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Map)) {
                // No generic clause, but there might be a generic map aspect later
                None
            } else {
                Some(super::interface::GenericClause::parse(parser)?)
            }
        } else {
            None
        };

        let generic_map_aspect = if parser.at_keyword(KeywordKind::Generic) {
            // GENERIC MAP ( ... )
            let gma = super::interface::GenericMapAspect::parse(parser)?;
            parser.expect(TokenKind::Semicolon)?;
            Some(gma)
        } else {
            None
        };

        let port_clause = if parser.at_keyword(KeywordKind::Port) {
            if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Map)) {
                None
            } else {
                Some(super::interface::PortClause::parse(parser)?)
            }
        } else {
            None
        };

        let port_map_aspect = if parser.at_keyword(KeywordKind::Port) {
            let pma = super::interface::PortMapAspect::parse(parser)?;
            parser.expect(TokenKind::Semicolon)?;
            Some(pma)
        } else {
            None
        };

        Ok(BlockHeader {
            generic_clause,
            generic_map_aspect,
            port_clause,
            port_map_aspect,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(ref gc) = self.generic_clause {
            gc.format(f, indent_level)?;
        }
        if let Some(ref gma) = self.generic_map_aspect {
            gma.format(f, indent_level)?;
        }
        if let Some(ref pc) = self.port_clause {
            pc.format(f, indent_level)?;
        }
        if let Some(ref pma) = self.port_map_aspect {
            pma.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for BlockDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { block_declarative_item }
        // Parse until BEGIN keyword
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::Begin) && !parser.eof() {
            items.push(BlockDeclarativeItem::parse(parser)?);
        }
        Ok(BlockDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for BlockDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Discriminate by leading keyword
        match parser.peek_kind() {
            // FUNCTION / PROCEDURE / PURE / IMPURE -> subprogram declaration or body
            Some(TokenKind::Keyword(KeywordKind::Function))
            | Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                // Try subprogram body first (has IS ... BEGIN ... END),
                // fall back to subprogram declaration (just specification ;)
                let saved = parser.save();
                match super::subprogram::SubprogramBody::parse(parser) {
                    Ok(body) => Ok(BlockDeclarativeItem::SubprogramBody(Box::new(body))),
                    Err(_) => {
                        parser.restore(saved);
                        let decl = super::subprogram::SubprogramDeclaration::parse(parser)?;
                        Ok(BlockDeclarativeItem::SubprogramDeclaration(Box::new(decl)))
                    }
                }
            }
            // PACKAGE -> package declaration, package body, or package instantiation
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                // PACKAGE BODY -> PackageBody
                // PACKAGE identifier IS NEW -> PackageInstantiationDeclaration
                // PACKAGE identifier IS -> PackageDeclaration
                if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Body))
                {
                    let body = super::package::PackageBody::parse(parser)?;
                    Ok(BlockDeclarativeItem::PackageBody(Box::new(body)))
                } else {
                    // Check for instantiation: PACKAGE id IS NEW
                    let saved = parser.save();
                    match super::package::PackageInstantiationDeclaration::parse(parser) {
                        Ok(inst) => Ok(BlockDeclarativeItem::PackageInstantiationDeclaration(
                            Box::new(inst),
                        )),
                        Err(_) => {
                            parser.restore(saved);
                            let decl = super::package::PackageDeclaration::parse(parser)?;
                            Ok(BlockDeclarativeItem::PackageDeclaration(Box::new(decl)))
                        }
                    }
                }
            }
            // TYPE -> type declaration
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                let decl = super::type_def::TypeDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::TypeDeclaration(Box::new(decl)))
            }
            // SUBTYPE -> subtype declaration
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                let decl = super::type_def::SubtypeDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::SubtypeDeclaration(Box::new(decl)))
            }
            // CONSTANT -> constant declaration
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                let decl = super::object_decl::ConstantDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::ConstantDeclaration(Box::new(decl)))
            }
            // SIGNAL -> signal declaration
            Some(TokenKind::Keyword(KeywordKind::Signal)) => {
                let decl = super::object_decl::SignalDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::SignalDeclaration(Box::new(decl)))
            }
            // SHARED VARIABLE -> shared variable declaration
            Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                let decl = super::object_decl::VariableDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::SharedVariableDeclaration(Box::new(
                    decl,
                )))
            }
            // FILE -> file declaration
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                let decl = super::object_decl::FileDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::FileDeclaration(Box::new(decl)))
            }
            // ALIAS -> alias declaration
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                let decl = super::object_decl::AliasDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::AliasDeclaration(Box::new(decl)))
            }
            // COMPONENT -> component declaration
            Some(TokenKind::Keyword(KeywordKind::Component)) => {
                let decl = super::component::ComponentDeclaration::parse(parser)?;
                Ok(BlockDeclarativeItem::ComponentDeclaration(Box::new(decl)))
            }
            // ATTRIBUTE -> attribute declaration or specification
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                // ATTRIBUTE identifier : -> declaration
                // ATTRIBUTE identifier OF -> specification
                let saved = parser.save();
                match super::attribute::AttributeDeclaration::parse(parser) {
                    Ok(decl) => Ok(BlockDeclarativeItem::AttributeDeclaration(Box::new(decl))),
                    Err(_) => {
                        parser.restore(saved);
                        let spec = super::attribute::AttributeSpecification::parse(parser)?;
                        Ok(BlockDeclarativeItem::AttributeSpecification(Box::new(spec)))
                    }
                }
            }
            // FOR -> configuration specification
            Some(TokenKind::Keyword(KeywordKind::For)) => {
                let spec = super::configuration::ConfigurationSpecification::parse(parser)?;
                Ok(BlockDeclarativeItem::ConfigurationSpecification(Box::new(
                    spec,
                )))
            }
            // DISCONNECT -> disconnection specification
            Some(TokenKind::Keyword(KeywordKind::Disconnect)) => {
                let spec = super::signal::DisconnectionSpecification::parse(parser)?;
                Ok(BlockDeclarativeItem::DisconnectionSpecification(Box::new(
                    spec,
                )))
            }
            // USE -> use clause
            Some(TokenKind::Keyword(KeywordKind::Use)) => {
                let clause = super::context::UseClause::parse(parser)?;
                Ok(BlockDeclarativeItem::UseClause(clause))
            }
            // GROUP -> group template or group declaration
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                let saved = parser.save();
                match super::group::GroupTemplateDeclaration::parse(parser) {
                    Ok(decl) => Ok(BlockDeclarativeItem::GroupTemplateDeclaration(Box::new(
                        decl,
                    ))),
                    Err(_) => {
                        parser.restore(saved);
                        let decl = super::group::GroupDeclaration::parse(parser)?;
                        Ok(BlockDeclarativeItem::GroupDeclaration(Box::new(decl)))
                    }
                }
            }
            _ => Err(parser.error("expected block declarative item")),
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
            Self::ComponentDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeDeclaration(inner) => inner.format(f, indent_level),
            Self::AttributeSpecification(inner) => inner.format(f, indent_level),
            Self::ConfigurationSpecification(inner) => inner.format(f, indent_level),
            Self::DisconnectionSpecification(inner) => inner.format(f, indent_level),
            Self::UseClause(inner) => inner.format(f, indent_level),
            Self::GroupTemplateDeclaration(inner) => inner.format(f, indent_level),
            Self::GroupDeclaration(inner) => inner.format(f, indent_level),
        }
    }
}

impl AstNode for BlockStatementPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { concurrent_statement }
        // Parse until END keyword
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(ConcurrentStatement::parse(parser)?);
        }
        Ok(BlockStatementPart { statements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for ProcessStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Full standalone parse: [ label : ] [ POSTPONED ] PROCESS ...
        let mut label = None;
        if (parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier))
            && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            label = Some(Label::parse(parser)?);
            parser.expect(TokenKind::Colon)?;
        }
        parse_process_with_label(parser, label)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(ref label) = self.label {
            label.format(f, 0)?;
            write!(f, " : ")?;
        }
        if self.postponed {
            write!(f, "postponed ")?;
        }
        write!(f, "process")?;
        if let Some(ref sensitivity) = self.sensitivity_list {
            write!(f, " (")?;
            sensitivity.format(f, 0)?;
            write!(f, ")")?;
        }
        writeln!(f, " is")?;
        self.declarative_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        writeln!(f, "begin")?;
        self.statement_part.format(f, indent_level + 1)?;
        write_indent(f, indent_level)?;
        write!(f, "end")?;
        if self.postponed {
            write!(f, " postponed")?;
        }
        write!(f, " process")?;
        if let Some(ref end_label) = self.end_label {
            write!(f, " ")?;
            end_label.format(f, 0)?;
        }
        writeln!(f, ";")
    }
}

impl AstNode for ProcessSensitivityList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::All).is_some() {
            Ok(ProcessSensitivityList::All)
        } else {
            let list = SensitivityList::parse(parser)?;
            Ok(ProcessSensitivityList::List(list))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::List(list) => list.format(f, indent_level),
        }
    }
}

impl AstNode for ProcessDeclarativePart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { process_declarative_item }
        // Parse until BEGIN keyword
        let mut items = Vec::new();
        while !parser.at_keyword(KeywordKind::Begin) && !parser.eof() {
            items.push(ProcessDeclarativeItem::parse(parser)?);
        }
        Ok(ProcessDeclarativePart { items })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProcessDeclarativeItem {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            // FUNCTION / PROCEDURE / PURE / IMPURE -> subprogram declaration or body
            Some(TokenKind::Keyword(KeywordKind::Function))
            | Some(TokenKind::Keyword(KeywordKind::Procedure))
            | Some(TokenKind::Keyword(KeywordKind::Pure))
            | Some(TokenKind::Keyword(KeywordKind::Impure)) => {
                let saved = parser.save();
                match super::subprogram::SubprogramBody::parse(parser) {
                    Ok(body) => Ok(ProcessDeclarativeItem::SubprogramBody(Box::new(body))),
                    Err(_) => {
                        parser.restore(saved);
                        let decl = super::subprogram::SubprogramDeclaration::parse(parser)?;
                        Ok(ProcessDeclarativeItem::SubprogramDeclaration(Box::new(
                            decl,
                        )))
                    }
                }
            }
            // PACKAGE -> package declaration, package body, or package instantiation
            Some(TokenKind::Keyword(KeywordKind::Package)) => {
                if parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Keyword(KeywordKind::Body))
                {
                    let body = super::package::PackageBody::parse(parser)?;
                    Ok(ProcessDeclarativeItem::PackageBody(Box::new(body)))
                } else {
                    let saved = parser.save();
                    match super::package::PackageInstantiationDeclaration::parse(parser) {
                        Ok(inst) => Ok(ProcessDeclarativeItem::PackageInstantiationDeclaration(
                            Box::new(inst),
                        )),
                        Err(_) => {
                            parser.restore(saved);
                            let decl = super::package::PackageDeclaration::parse(parser)?;
                            Ok(ProcessDeclarativeItem::PackageDeclaration(Box::new(decl)))
                        }
                    }
                }
            }
            // TYPE -> type declaration
            Some(TokenKind::Keyword(KeywordKind::Type)) => {
                let decl = super::type_def::TypeDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::TypeDeclaration(Box::new(decl)))
            }
            // SUBTYPE -> subtype declaration
            Some(TokenKind::Keyword(KeywordKind::Subtype)) => {
                let decl = super::type_def::SubtypeDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::SubtypeDeclaration(Box::new(decl)))
            }
            // CONSTANT -> constant declaration
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                let decl = super::object_decl::ConstantDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::ConstantDeclaration(Box::new(decl)))
            }
            // VARIABLE or SHARED VARIABLE -> variable declaration
            Some(TokenKind::Keyword(KeywordKind::Variable))
            | Some(TokenKind::Keyword(KeywordKind::Shared)) => {
                let decl = super::object_decl::VariableDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::VariableDeclaration(Box::new(decl)))
            }
            // FILE -> file declaration
            Some(TokenKind::Keyword(KeywordKind::File)) => {
                let decl = super::object_decl::FileDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::FileDeclaration(Box::new(decl)))
            }
            // ALIAS -> alias declaration
            Some(TokenKind::Keyword(KeywordKind::Alias)) => {
                let decl = super::object_decl::AliasDeclaration::parse(parser)?;
                Ok(ProcessDeclarativeItem::AliasDeclaration(Box::new(decl)))
            }
            // ATTRIBUTE -> attribute declaration or specification
            Some(TokenKind::Keyword(KeywordKind::Attribute)) => {
                let saved = parser.save();
                match super::attribute::AttributeDeclaration::parse(parser) {
                    Ok(decl) => Ok(ProcessDeclarativeItem::AttributeDeclaration(Box::new(decl))),
                    Err(_) => {
                        parser.restore(saved);
                        let spec = super::attribute::AttributeSpecification::parse(parser)?;
                        Ok(ProcessDeclarativeItem::AttributeSpecification(Box::new(
                            spec,
                        )))
                    }
                }
            }
            // USE -> use clause
            Some(TokenKind::Keyword(KeywordKind::Use)) => {
                let clause = super::context::UseClause::parse(parser)?;
                Ok(ProcessDeclarativeItem::UseClause(clause))
            }
            // GROUP -> group template or group declaration
            Some(TokenKind::Keyword(KeywordKind::Group)) => {
                let saved = parser.save();
                match super::group::GroupTemplateDeclaration::parse(parser) {
                    Ok(decl) => Ok(ProcessDeclarativeItem::GroupTemplateDeclaration(Box::new(
                        decl,
                    ))),
                    Err(_) => {
                        parser.restore(saved);
                        let decl = super::group::GroupDeclaration::parse(parser)?;
                        Ok(ProcessDeclarativeItem::GroupDeclaration(Box::new(decl)))
                    }
                }
            }
            _ => Err(parser.error("expected process declarative item")),
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

impl AstNode for ProcessStatementPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { sequential_statement }
        // Parse until END keyword
        let mut statements = Vec::new();
        while !parser.at_keyword(KeywordKind::End) && !parser.eof() {
            statements.push(SequentialStatement::parse(parser)?);
        }
        Ok(ProcessStatementPart { statements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for ConcurrentAssertionStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ label : ] [ POSTPONED ] assertion ;
        let mut label = None;
        if (parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier))
            && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            label = Some(Label::parse(parser)?);
            parser.expect(TokenKind::Colon)?;
        }
        let postponed = parser.consume_if_keyword(KeywordKind::Postponed).is_some();
        let assertion = Assertion::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConcurrentAssertionStatement {
            label,
            postponed,
            assertion,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(ref label) = self.label {
            label.format(f, 0)?;
            write!(f, " : ")?;
        }
        if self.postponed {
            write!(f, "postponed ")?;
        }
        self.assertion.format(f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for ConcurrentProcedureCallStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ label : ] [ POSTPONED ] procedure_call ;
        let mut label = None;
        if (parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier))
            && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            label = Some(Label::parse(parser)?);
            parser.expect(TokenKind::Colon)?;
        }
        let postponed = parser.consume_if_keyword(KeywordKind::Postponed).is_some();
        let procedure_call = ProcedureCall::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConcurrentProcedureCallStatement {
            label,
            postponed,
            procedure_call,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        if let Some(ref label) = self.label {
            label.format(f, 0)?;
            write!(f, " : ")?;
        }
        if self.postponed {
            write!(f, "postponed ")?;
        }
        self.procedure_call.format(f, 0)?;
        writeln!(f, ";")
    }
}

impl AstNode for ConcurrentSignalAssignmentStatement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ label : ] [ POSTPONED ] (simple | conditional | selected)
        let mut label = None;
        if (parser.at(TokenKind::Identifier) || parser.at(TokenKind::ExtendedIdentifier))
            && parser.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            label = Some(Label::parse(parser)?);
            parser.expect(TokenKind::Colon)?;
        }
        let postponed = parser.consume_if_keyword(KeywordKind::Postponed).is_some();

        if parser.at_keyword(KeywordKind::With) {
            // Selected signal assignment
            let assignment = parse_selected_signal_assignment(parser)?;
            Ok(ConcurrentSignalAssignmentStatement::Selected {
                label,
                postponed,
                assignment,
            })
        } else {
            // Simple or conditional: target <= ...
            let target = Target::parse(parser)?;
            parser.expect(TokenKind::LtEquals)?;
            let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
            let delay_mechanism = try_parse_delay_mechanism(parser);
            let waveform = Waveform::parse(parser)?;

            if parser.at_keyword(KeywordKind::When) {
                parser.consume();
                let first_condition = Condition::parse(parser)?;
                let mut alternatives = vec![ConditionalWaveformAlternative {
                    waveform,
                    condition: first_condition,
                }];
                let mut else_waveform = None;
                while parser.consume_if_keyword(KeywordKind::Else).is_some() {
                    let next_wf = Waveform::parse(parser)?;
                    if parser.at_keyword(KeywordKind::When) {
                        parser.consume();
                        let cond = Condition::parse(parser)?;
                        alternatives.push(ConditionalWaveformAlternative {
                            waveform: next_wf,
                            condition: cond,
                        });
                    } else {
                        else_waveform = Some(next_wf);
                        break;
                    }
                }
                parser.expect(TokenKind::Semicolon)?;
                Ok(ConcurrentSignalAssignmentStatement::Conditional {
                    label,
                    postponed,
                    assignment: ConcurrentConditionalSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        conditional_waveforms: ConditionalWaveforms {
                            alternatives,
                            else_waveform,
                        },
                    },
                })
            } else {
                parser.expect(TokenKind::Semicolon)?;
                Ok(ConcurrentSignalAssignmentStatement::Simple {
                    label,
                    postponed,
                    assignment: ConcurrentSimpleSignalAssignment {
                        target,
                        guarded,
                        delay_mechanism,
                        waveform,
                    },
                })
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Simple {
                label,
                postponed,
                assignment,
            } => {
                write_indent(f, indent_level)?;
                if let Some(l) = label {
                    l.format(f, 0)?;
                    write!(f, " : ")?;
                }
                if *postponed {
                    write!(f, "postponed ")?;
                }
                assignment.format(f, 0)?;
                writeln!(f)
            }
            Self::Conditional {
                label,
                postponed,
                assignment,
            } => {
                write_indent(f, indent_level)?;
                if let Some(l) = label {
                    l.format(f, 0)?;
                    write!(f, " : ")?;
                }
                if *postponed {
                    write!(f, "postponed ")?;
                }
                assignment.format(f, 0)?;
                writeln!(f)
            }
            Self::Selected {
                label,
                postponed,
                assignment,
            } => {
                write_indent(f, indent_level)?;
                if let Some(l) = label {
                    l.format(f, 0)?;
                    write!(f, " : ")?;
                }
                if *postponed {
                    write!(f, "postponed ")?;
                }
                assignment.format(f, 0)?;
                writeln!(f)
            }
        }
    }
}

impl AstNode for ConcurrentSimpleSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // target <= [ GUARDED ] [ delay_mechanism ] waveform ;
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
        let delay_mechanism = try_parse_delay_mechanism(parser);
        let waveform = Waveform::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConcurrentSimpleSignalAssignment {
            target,
            guarded,
            delay_mechanism,
            waveform,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= ")?;
        if self.guarded {
            write!(f, "guarded ")?;
        }
        if let Some(ref delay) = self.delay_mechanism {
            delay.format(f, 0)?;
            write!(f, " ")?;
        }
        self.waveform.format(f, 0)?;
        write!(f, ";")
    }
}

impl AstNode for ConcurrentConditionalSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // target <= [ GUARDED ] [ delay_mechanism ] conditional_waveforms ;
        let target = Target::parse(parser)?;
        parser.expect(TokenKind::LtEquals)?;
        let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
        let delay_mechanism = try_parse_delay_mechanism(parser);
        let conditional_waveforms = ConditionalWaveforms::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(ConcurrentConditionalSignalAssignment {
            target,
            guarded,
            delay_mechanism,
            conditional_waveforms,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.target.format(f, indent_level)?;
        write!(f, " <= ")?;
        if self.guarded {
            write!(f, "guarded ")?;
        }
        if let Some(ref delay) = self.delay_mechanism {
            delay.format(f, 0)?;
            write!(f, " ")?;
        }
        self.conditional_waveforms.format(f, 0)?;
        write!(f, ";")
    }
}

impl AstNode for ConcurrentSelectedSignalAssignment {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // WITH expression SELECT [?] target <= [GUARDED] [delay_mechanism] selected_waveforms ;
        parse_selected_signal_assignment(parser)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "with ")?;
        self.selector.format(f, indent_level)?;
        write!(f, " select")?;
        if self.matching {
            write!(f, " ?")?;
        }
        write!(f, " ")?;
        self.target.format(f, 0)?;
        write!(f, " <= ")?;
        if self.guarded {
            write!(f, "guarded ")?;
        }
        if let Some(ref delay) = self.delay_mechanism {
            delay.format(f, 0)?;
            write!(f, " ")?;
        }
        self.selected_waveforms.format(f, 0)?;
        write!(f, ";")
    }
}

impl AstNode for Options {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // [ GUARDED ] [ delay_mechanism ]
        let guarded = parser.consume_if_keyword(KeywordKind::Guarded).is_some();
        let delay_mechanism = try_parse_delay_mechanism(parser);
        Ok(Options {
            guarded,
            delay_mechanism,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        if self.guarded {
            write!(f, "guarded")?;
        }
        if let Some(ref delay) = self.delay_mechanism {
            if self.guarded {
                write!(f, " ")?;
            }
            delay.format(f, 0)?;
        }
        Ok(())
    }
}
