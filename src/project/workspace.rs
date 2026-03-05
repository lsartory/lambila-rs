//! Workspace — the top-level project container.
//!
//! A [`Workspace`] owns a [`SourceManager`] and a set of logical VHDL
//! libraries.  It provides the entry-point for loading VHDL source files,
//! parsing them, and assigning their design units to the appropriate library.

use std::collections::HashMap;
use std::path::Path;

use crate::VhdlVersion;
use crate::ast::design_unit::DesignFile;
use crate::ast::node::AstNode;
use crate::parser::Parser;

use super::library::{DesignUnitId, Library};
use super::source::{SourceId, SourceManager};

/// The default library name for user code.
pub const DEFAULT_LIBRARY: &str = "work";

/// A VHDL project workspace.
///
/// Holds all source files and libraries for a compilation session.
#[derive(Debug)]
pub struct Workspace {
    /// Source file registry.
    pub sources: SourceManager,
    /// Mapping from logical library names to [`Library`] instances.
    libraries: HashMap<String, Library>,
}

/// An error produced while loading a source file into the workspace.
#[derive(Debug)]
pub enum WorkspaceError {
    /// I/O error (file not found, permission denied, …).
    Io(std::io::Error),
    /// Lexer errors encountered during tokenization.
    Lex(Vec<String>),
    /// Parser error.
    Parse(crate::parser::ParseError),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::Io(e) => write!(f, "I/O error: {}", e),
            WorkspaceError::Lex(errors) => {
                write!(f, "lexer errors: ")?;
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", e)?;
                }
                Ok(())
            }
            WorkspaceError::Parse(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for WorkspaceError {}

impl From<std::io::Error> for WorkspaceError {
    fn from(e: std::io::Error) -> Self {
        WorkspaceError::Io(e)
    }
}

impl From<crate::parser::ParseError> for WorkspaceError {
    fn from(e: crate::parser::ParseError) -> Self {
        WorkspaceError::Parse(e)
    }
}

/// Descriptor for every design unit that was added during a load operation.
#[derive(Debug, Clone)]
pub struct LoadedUnit {
    /// Library the unit was placed in.
    pub library_name: String,
    /// Id within that library.
    pub unit_id: DesignUnitId,
    /// Source file the unit came from.
    pub source_id: SourceId,
}

impl Workspace {
    /// Create a new, empty workspace.
    pub fn new() -> Self {
        Workspace {
            sources: SourceManager::new(),
            libraries: HashMap::new(),
        }
    }

    /// Get or create a library by name.
    pub fn library_mut(&mut self, name: &str) -> &mut Library {
        self.libraries
            .entry(name.to_lowercase())
            .or_insert_with(|| Library::new(name.to_lowercase()))
    }

    /// Get a library by name (read-only).
    pub fn library(&self, name: &str) -> Option<&Library> {
        self.libraries.get(&name.to_lowercase())
    }

    /// Iterate over all libraries.
    pub fn libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries.values()
    }

    /// Load a VHDL file from disk, parse it, and add all design units to the
    /// given library (defaults to `"work"`).
    ///
    /// Returns the list of units that were added.
    pub fn load_file(
        &mut self,
        path: &Path,
        version: VhdlVersion,
        library_name: Option<&str>,
    ) -> Result<Vec<LoadedUnit>, WorkspaceError> {
        let source_id = self.sources.load(path)?;
        let lib_name = library_name.unwrap_or(DEFAULT_LIBRARY);

        let content = self.sources.get(source_id).content.clone();
        let design_file = parse_source(&content, version)?;

        let mut loaded = Vec::new();
        let lib = self.library_mut(lib_name);
        for unit in design_file.design_units {
            let unit_id = lib.add_unit(unit, source_id, version);
            loaded.push(LoadedUnit {
                library_name: lib_name.to_string(),
                unit_id,
                source_id,
            });
        }
        Ok(loaded)
    }

    /// Load a VHDL source from an in-memory string. Useful for tests and
    /// embedded standard libraries.
    pub fn load_str(
        &mut self,
        logical_path: &str,
        source: &str,
        version: VhdlVersion,
        library_name: Option<&str>,
    ) -> Result<Vec<LoadedUnit>, WorkspaceError> {
        let source_id = self
            .sources
            .add_in_memory(logical_path.into(), source.to_string());
        let lib_name = library_name.unwrap_or(DEFAULT_LIBRARY);

        let design_file = parse_source(source, version)?;

        let mut loaded = Vec::new();
        let lib = self.library_mut(lib_name);
        for unit in design_file.design_units {
            let unit_id = lib.add_unit(unit, source_id, version);
            loaded.push(LoadedUnit {
                library_name: lib_name.to_string(),
                unit_id,
                source_id,
            });
        }
        Ok(loaded)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Lex + parse a VHDL source string into a `DesignFile`.
fn parse_source(source: &str, version: VhdlVersion) -> Result<DesignFile, WorkspaceError> {
    let lex_result = crate::lex(source, version);
    if !lex_result.errors.is_empty() {
        return Err(WorkspaceError::Lex(
            lex_result.errors.iter().map(|e| e.to_string()).collect(),
        ));
    }
    let mut parser = Parser::new(&lex_result.tokens);
    let design_file = DesignFile::parse(&mut parser)?;
    Ok(design_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_single_entity_to_work() {
        let mut ws = Workspace::new();
        let loaded = ws
            .load_str(
                "test.vhd",
                "entity foo is end entity foo;",
                VhdlVersion::Vhdl1993,
                None,
            )
            .unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].library_name, "work");
        let lib = ws.library("work").unwrap();
        assert_eq!(lib.len(), 1);
    }

    #[test]
    fn load_multiple_units_from_one_file() {
        let mut ws = Workspace::new();
        let src = "\
            entity bar is end entity bar;\n\
            architecture rtl of bar is begin end architecture rtl;\n";
        let loaded = ws
            .load_str("bar.vhd", src, VhdlVersion::Vhdl1993, None)
            .unwrap();
        assert_eq!(loaded.len(), 2);
        let lib = ws.library("work").unwrap();
        assert_eq!(lib.len(), 2);
    }

    #[test]
    fn load_into_custom_library() {
        let mut ws = Workspace::new();
        ws.load_str(
            "my_lib.vhd",
            "package my_pkg is end package my_pkg;",
            VhdlVersion::Vhdl1993,
            Some("my_lib"),
        )
        .unwrap();
        assert!(ws.library("work").is_none());
        let lib = ws.library("my_lib").unwrap();
        assert_eq!(lib.len(), 1);
    }
}
