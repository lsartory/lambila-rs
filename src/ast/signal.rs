//! Signal-related AST nodes.

use super::expression::Expression;
use super::name::Name;
use super::type_def::TypeMark;

/// EBNF: `signal_list ::= signal_name { , signal_name } | OTHERS | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalList {
    Names(Vec<Name>),
    Others,
    All,
}

/// EBNF: `guarded_signal_specification ::= guarded_signal_list : type_mark`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedSignalSpecification {
    pub signal_list: SignalList,
    pub type_mark: TypeMark,
}

/// EBNF: `disconnection_specification ::= DISCONNECT guarded_signal_specification
///     AFTER time_expression ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectionSpecification {
    pub guarded_signal_spec: GuardedSignalSpecification,
    pub time_expression: Expression,
}
