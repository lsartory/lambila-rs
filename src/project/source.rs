//! Source file management.
//!
//! Provides a central registry for loaded VHDL source files, assigning each a
//! unique [`SourceId`] and retaining its path and raw text for the lifetime of
//! the compilation session.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// An opaque handle that uniquely identifies a loaded source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(pub(crate) usize);

/// Metadata for a single loaded source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Unique identifier.
    pub id: SourceId,
    /// Canonical filesystem path.
    pub path: PathBuf,
    /// Raw text content of the file.
    pub content: String,
}

/// Central registry that manages all loaded VHDL source files.
///
/// Files are loaded at most once (keyed by canonical path). Subsequent
/// requests for the same path return the previously assigned [`SourceId`].
#[derive(Debug, Default)]
pub struct SourceManager {
    /// All known source files, indexed by their [`SourceId`].
    files: Vec<SourceFile>,
    /// Maps canonical paths to their [`SourceId`] to avoid duplicates.
    path_index: HashMap<PathBuf, SourceId>,
}

impl SourceManager {
    /// Create an empty source manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a source file from disk.
    ///
    /// If the canonical path has already been loaded, the existing
    /// [`SourceId`] is returned without re-reading the file.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if the path cannot be canonicalized or
    /// the file cannot be read.
    pub fn load(&mut self, path: &Path) -> std::io::Result<SourceId> {
        let canonical = std::fs::canonicalize(path)?;

        if let Some(&id) = self.path_index.get(&canonical) {
            return Ok(id);
        }

        let content = std::fs::read_to_string(&canonical)?;
        let id = SourceId(self.files.len());
        self.files.push(SourceFile {
            id,
            path: canonical.clone(),
            content,
        });
        self.path_index.insert(canonical, id);
        Ok(id)
    }

    /// Register a source file from an in-memory string (useful for tests).
    ///
    /// The `logical_path` is stored as-is (no canonicalization) and must be
    /// unique across all calls.
    pub fn add_in_memory(&mut self, logical_path: PathBuf, content: String) -> SourceId {
        if let Some(&id) = self.path_index.get(&logical_path) {
            return id;
        }
        let id = SourceId(self.files.len());
        self.files.push(SourceFile {
            id,
            path: logical_path.clone(),
            content,
        });
        self.path_index.insert(logical_path, id);
        id
    }

    /// Get a source file by its [`SourceId`].
    ///
    /// # Panics
    ///
    /// Panics if the id is out of range (should never happen with ids obtained
    /// from this manager).
    pub fn get(&self, id: SourceId) -> &SourceFile {
        &self.files[id.0]
    }

    /// Iterate over all loaded source files.
    pub fn iter(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter()
    }

    /// Total number of loaded source files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` if no files have been loaded.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_in_memory_assigns_unique_ids() {
        let mut mgr = SourceManager::new();
        let id_a = mgr.add_in_memory(PathBuf::from("a.vhd"), "-- file a".into());
        let id_b = mgr.add_in_memory(PathBuf::from("b.vhd"), "-- file b".into());
        assert_ne!(id_a, id_b);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn duplicate_in_memory_returns_same_id() {
        let mut mgr = SourceManager::new();
        let id1 = mgr.add_in_memory(PathBuf::from("x.vhd"), "-- x".into());
        let id2 = mgr.add_in_memory(PathBuf::from("x.vhd"), "-- x".into());
        assert_eq!(id1, id2);
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn get_returns_correct_content() {
        let mut mgr = SourceManager::new();
        let id = mgr.add_in_memory(PathBuf::from("t.vhd"), "entity foo is end;".into());
        let file = mgr.get(id);
        assert_eq!(file.content, "entity foo is end;");
        assert_eq!(file.path, PathBuf::from("t.vhd"));
    }
}
