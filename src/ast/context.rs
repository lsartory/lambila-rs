//! Context and library AST nodes.

use super::common::*;
use super::name::SelectedName;

/// EBNF: `context_clause ::= { context_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextClause {
    pub items: Vec<ContextItem>,
}

/// EBNF (VHDL-2008): `context_item ::= library_clause | use_clause | context_reference`
/// EBNF (VHDL-87/93): `context_item ::= library_clause | use_clause`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextItem {
    Library(LibraryClause),
    Use(UseClause),
    /// VHDL-2008.
    ContextReference(ContextReference),
}

/// EBNF: `context_declaration ::= CONTEXT identifier IS context_clause
///     END [ CONTEXT ] [ context_simple_name ] ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDeclaration {
    pub identifier: Identifier,
    pub context_clause: ContextClause,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `context_reference ::= CONTEXT selected_name { , selected_name } ;` (VHDL-2008)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextReference {
    pub names: Vec<SelectedName>,
}

/// EBNF: `library_clause ::= LIBRARY logical_name_list ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryClause {
    pub logical_names: LogicalNameList,
}

/// EBNF: `logical_name ::= identifier`
pub type LogicalName = Identifier;

/// EBNF: `logical_name_list ::= logical_name { , logical_name }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalNameList {
    pub names: Vec<LogicalName>,
}

/// EBNF: `use_clause ::= USE selected_name { , selected_name } ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseClause {
    pub names: Vec<SelectedName>,
}
