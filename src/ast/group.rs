//! Group declaration AST nodes (VHDL-93+).

use super::attribute::EntityClassEntryList;
use super::common::*;
use super::name::Name;

/// EBNF: `group_template_declaration ::= GROUP identifier IS ( entity_class_entry_list ) ;`
/// (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupTemplateDeclaration {
    pub identifier: Identifier,
    pub entity_class_entry_list: EntityClassEntryList,
}

/// EBNF: `group_declaration ::= GROUP identifier : group_template_name
///     ( group_constituent_list ) ;` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDeclaration {
    pub identifier: Identifier,
    pub template_name: Box<Name>,
    pub constituent_list: GroupConstituentList,
}

/// EBNF: `group_constituent ::= name | character_literal` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupConstituent {
    Name(Name),
    CharacterLiteral(String),
}

/// EBNF: `group_constituent_list ::= group_constituent { , group_constituent }` (VHDL-93+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupConstituentList {
    pub constituents: Vec<GroupConstituent>,
}
