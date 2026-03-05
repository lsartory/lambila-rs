//! Semantic analysis for VHDL design units.
//!
//! This module validates the parsed AST by building symbol tables,
//! resolving names, and performing type checking. It operates on a
//! [`Workspace`] whose design units have already been topologically sorted.

pub mod check;
pub mod resolve;
pub mod scope;
pub mod symbol;

use crate::project::workspace::Workspace;

/// Errors reported during semantic analysis.
#[derive(Debug, Clone)]
pub struct SemanticError {
    /// Human-readable description of the error.
    pub message: String,
    /// Library containing the offending design unit.
    pub library: String,
    /// Name of the design unit where the error was detected.
    pub unit_name: String,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}.{}] {}", self.library, self.unit_name, self.message)
    }
}

/// Result of running the semantic analyzer over a workspace.
#[derive(Debug)]
pub struct AnalysisResult {
    /// The global scope containing all resolved symbols.
    pub global_scope: scope::ScopeArena,
    /// Errors collected during analysis (non-fatal; analysis continues).
    pub errors: Vec<SemanticError>,
}

/// Run semantic analysis on a workspace.
///
/// Design units are processed in topological order so that dependencies
/// (entities before architectures, packages before package bodies, etc.)
/// are always available when needed.
pub fn analyze(workspace: &Workspace) -> AnalysisResult {
    let mut arena = scope::ScopeArena::new();
    let mut errors = Vec::new();

    check::analyze_workspace(workspace, &mut arena, &mut errors);

    AnalysisResult {
        global_scope: arena,
        errors,
    }
}
