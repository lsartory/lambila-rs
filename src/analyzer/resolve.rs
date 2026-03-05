//! Name resolution utilities.
//!
//! Given a scope and an AST [`Name`], this module resolves the name to a set of candidate symbols.
//! Overload resolution for subprograms is handled by filtering candidates based on the number and types of actual parameters.

use crate::ast::common::Identifier;
use crate::ast::name::{Name, Suffix};

use super::scope::{ScopeArena, ScopeId};
use super::symbol::{Symbol, SymbolKind};

/// Result of resolving a name.
#[derive(Debug)]
pub enum ResolveResult<'a> {
    /// Exactly one symbol matched.
    Resolved(&'a Symbol),
    /// Several equally valid candidates (e.g. overloaded subprograms
    /// when we don't yet have type information).
    Ambiguous(Vec<&'a Symbol>),
    /// No matching declaration was found.
    Unresolved(String),
}

/// Resolve a simple name (case-insensitive) in the given scope.
pub fn resolve_simple<'a>(arena: &'a ScopeArena, scope: ScopeId, name: &str) -> ResolveResult<'a> {
    let lower = name.to_lowercase();
    let candidates = arena.lookup(scope, &lower);
    match candidates.len() {
        0 => ResolveResult::Unresolved(lower),
        1 => ResolveResult::Resolved(candidates[0]),
        _ => ResolveResult::Ambiguous(candidates),
    }
}

/// Resolve an AST [`Name`] node by extracting its text and looking it up.
pub fn resolve_name<'a>(arena: &'a ScopeArena, scope: ScopeId, name: &Name) -> ResolveResult<'a> {
    match name {
        Name::Simple(s) => {
            let text = ident_to_lower(&s.identifier);
            resolve_simple(arena, scope, &text)
        }
        Name::Selected(sel) => {
            // For selected names like `pkg.sym`, first resolve the prefix,
            // then look up the suffix in the prefix's scope.
            let suffix_text = suffix_to_lower(&sel.suffix);
            // If the prefix resolved to a package, look in that package's
            // scope. For now, fall back to a flat lookup.
            resolve_simple(arena, scope, &suffix_text)
        }
        // Indexed, slice, attribute names — fall back to best-effort.
        _ => {
            let text = name_to_lower(name);
            resolve_simple(arena, scope, &text)
        }
    }
}

/// Check whether a resolved symbol is of a type-compatible kind for a
/// given expected category. This is a simplified compatibility check.
pub fn is_type_compatible(actual: &SymbolKind, expected: &str) -> bool {
    match actual {
        SymbolKind::Signal { type_name, .. }
        | SymbolKind::Variable { type_name, .. }
        | SymbolKind::Constant { type_name, .. }
        | SymbolKind::File { type_name } => type_name == expected,
        SymbolKind::EnumLiteral { type_name } => type_name == expected,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ident_to_lower(id: &Identifier) -> String {
    match id {
        Identifier::Basic(s) => s.to_lowercase(),
        Identifier::Extended(s) => s.clone(),
    }
}

fn suffix_to_lower(suffix: &Suffix) -> String {
    match suffix {
        Suffix::SimpleName(s) => ident_to_lower(&s.identifier),
        Suffix::All => "all".to_string(),
        Suffix::OperatorSymbol(op) => op.text.to_lowercase(),
        Suffix::CharacterLiteral(c) => c.clone(),
    }
}

fn name_to_lower(name: &Name) -> String {
    match name {
        Name::Simple(s) => ident_to_lower(&s.identifier),
        Name::Selected(sel) => suffix_to_lower(&sel.suffix),
        _ => "<unknown>".to_string(),
    }
}
