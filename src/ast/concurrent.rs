//! Concurrent statement AST nodes.

use super::common::*;
use super::expression::{Condition, Expression};
use super::node::{AstNode, write_indent, format_lines};
use super::sequential::*;
use crate::parser::{Parser, ParseError};

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
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for ConcurrentStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for BlockDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for ProcessStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::List(list) => list.format(f, indent_level),
        }
    }
}

impl AstNode for ProcessDeclarativePart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.items, f, indent_level)
    }
}

impl AstNode for ProcessDeclarativeItem {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_lines(&self.statements, f, indent_level)
    }
}

impl AstNode for ConcurrentAssertionStatement {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Self::Simple { label, postponed, assignment } => {
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
            Self::Conditional { label, postponed, assignment } => {
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
            Self::Selected { label, postponed, assignment } => {
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
