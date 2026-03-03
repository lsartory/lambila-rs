//! Architecture body AST nodes.

use super::common::*;

/// EBNF (VHDL-2008): `architecture_body ::= ARCHITECTURE identifier OF entity_name IS
///     architecture_declarative_part BEGIN architecture_statement_part
///     END [ ARCHITECTURE ] [ architecture_simple_name ] ;`
/// EBNF (VHDL-87): `...END [ architecture_simple_name ] ;` (no optional ARCHITECTURE keyword).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureBody {
    pub identifier: Identifier,
    pub entity_name: SimpleName,
    pub declarative_part: ArchitectureDeclarativePart,
    pub statement_part: ArchitectureStatementPart,
    pub end_name: Option<SimpleName>,
}

/// EBNF: `architecture_declarative_part ::= { block_declarative_item }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureDeclarativePart {
    pub items: Vec<super::concurrent::BlockDeclarativeItem>,
}

/// EBNF: `architecture_statement_part ::= { concurrent_statement }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureStatementPart {
    pub statements: Vec<super::concurrent::ConcurrentStatement>,
}
