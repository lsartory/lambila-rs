//! Hierarchical scope management and symbol table.
//!
//! VHDL has nested declarative regions: a process scope lives inside an
//! architecture scope, which lives inside an entity scope. The [`ScopeArena`]
//! holds all scopes and symbols, while [`ScopeId`] identifies a particular
//! scope. Lookups walk up the parent chain.

use std::collections::HashMap;

use super::symbol::{Symbol, SymbolId, SymbolKind};

/// Identifies a scope within the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

/// The kind of declarative region a scope represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    /// The implicit root scope (contains library-level declarations).
    Root,
    /// A VHDL library (e.g. `work`, `ieee`).
    Library(String),
    /// An entity declarative region.
    Entity(String),
    /// An architecture declarative region.
    Architecture { name: String, entity: String },
    /// A package declarative region.
    Package(String),
    /// A package body.
    PackageBody(String),
    /// A process declarative region.
    Process(Option<String>),
    /// A subprogram declarative region.
    Subprogram(String),
    /// A block declarative region.
    Block(String),
    /// A generate declarative region.
    Generate(String),
}

/// A single scope (declarative region).
#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    /// Symbols declared directly in this scope, keyed by lowercase name.
    /// A name may map to multiple symbols (overloaded subprograms, enum
    /// literals).
    symbols: HashMap<String, Vec<SymbolId>>,
    /// Symbols imported with `use` clauses (transitive visibility).
    imported: HashMap<String, Vec<SymbolId>>,
}

/// Arena that owns all scopes and symbols.
#[derive(Debug)]
pub struct ScopeArena {
    scopes: Vec<Scope>,
    symbols: Vec<Symbol>,
}

impl ScopeArena {
    /// Create a new arena with an empty root scope.
    pub fn new() -> Self {
        let root = Scope {
            id: ScopeId(0),
            kind: ScopeKind::Root,
            parent: None,
            symbols: HashMap::new(),
            imported: HashMap::new(),
        };
        ScopeArena {
            scopes: vec![root],
            symbols: Vec::new(),
        }
    }

    /// The root scope id.
    pub fn root(&self) -> ScopeId {
        ScopeId(0)
    }

    /// Create a child scope under `parent`.
    pub fn new_scope(&mut self, kind: ScopeKind, parent: ScopeId) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            id,
            kind,
            parent: Some(parent),
            symbols: HashMap::new(),
            imported: HashMap::new(),
        });
        id
    }

    /// Add a symbol to a scope and return its id.
    pub fn add_symbol(
        &mut self,
        scope: ScopeId,
        name: String,
        kind: SymbolKind,
        library: &str,
        unit_name: &str,
    ) -> SymbolId {
        let id = SymbolId(self.symbols.len());
        self.symbols.push(Symbol {
            id,
            name: name.clone(),
            kind,
            library: library.to_string(),
            unit_name: unit_name.to_string(),
        });
        self.scopes[scope.0]
            .symbols
            .entry(name)
            .or_default()
            .push(id);
        id
    }

    /// Import a symbol into a scope (simulating a `use` clause).
    pub fn import_symbol(&mut self, scope: ScopeId, name: String, symbol_id: SymbolId) {
        self.scopes[scope.0]
            .imported
            .entry(name)
            .or_default()
            .push(symbol_id);
    }

    /// Look up a name in the given scope, walking up the parent chain.
    /// Returns all matching symbol ids (there may be several for
    /// overloaded names).
    pub fn lookup(&self, scope: ScopeId, name: &str) -> Vec<&Symbol> {
        let mut current = Some(scope);
        while let Some(sid) = current {
            let s = &self.scopes[sid.0];
            // Direct declarations first.
            if let Some(ids) = s.symbols.get(name) {
                return ids.iter().map(|id| &self.symbols[id.0]).collect();
            }
            // Then imported symbols.
            if let Some(ids) = s.imported.get(name) {
                return ids.iter().map(|id| &self.symbols[id.0]).collect();
            }
            current = s.parent;
        }
        Vec::new()
    }

    /// Look up only in the immediate scope (no parent walk).
    pub fn lookup_local(&self, scope: ScopeId, name: &str) -> Vec<&Symbol> {
        let s = &self.scopes[scope.0];
        if let Some(ids) = s.symbols.get(name) {
            return ids.iter().map(|id| &self.symbols[id.0]).collect();
        }
        Vec::new()
    }

    /// Get a scope by id.
    pub fn scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0]
    }

    /// Get a symbol by id.
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0]
    }

    /// Return all symbols in the arena (for inspection / debug).
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Return all scopes in the arena (for inspection / debug).
    pub fn all_scopes(&self) -> &[Scope] {
        &self.scopes
    }

    /// Find a scope by kind predicate (useful for finding a library scope).
    pub fn find_scope<F: Fn(&ScopeKind) -> bool>(&self, pred: F) -> Option<ScopeId> {
        self.scopes.iter().find(|s| pred(&s.kind)).map(|s| s.id)
    }
}

impl Scope {
    /// Iterator over all direct (non-imported) symbols in this scope.
    pub fn direct_symbol_ids(&self) -> impl Iterator<Item = (String, SymbolId)> + '_ {
        self.symbols
            .iter()
            .flat_map(|(name, ids)| ids.iter().map(move |id| (name.clone(), *id)))
    }
}

impl Default for ScopeArena {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_scope_exists() {
        let arena = ScopeArena::new();
        assert_eq!(arena.root(), ScopeId(0));
        assert_eq!(arena.scope(ScopeId(0)).kind, ScopeKind::Root);
    }

    #[test]
    fn add_and_lookup_symbol() {
        let mut arena = ScopeArena::new();
        let lib = arena.new_scope(ScopeKind::Library("work".into()), arena.root());
        arena.add_symbol(
            lib,
            "my_signal".into(),
            SymbolKind::Signal {
                type_name: "std_logic".into(),
                mode: None,
            },
            "work",
            "test_entity",
        );

        let results = arena.lookup(lib, "my_signal");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "my_signal");
    }

    #[test]
    fn parent_chain_lookup() {
        let mut arena = ScopeArena::new();
        let lib = arena.new_scope(ScopeKind::Library("work".into()), arena.root());
        let entity = arena.new_scope(ScopeKind::Entity("foo".into()), lib);
        let arch = arena.new_scope(
            ScopeKind::Architecture {
                name: "rtl".into(),
                entity: "foo".into(),
            },
            entity,
        );

        // Declare a signal in the entity scope.
        arena.add_symbol(
            entity,
            "clk".into(),
            SymbolKind::Signal {
                type_name: "std_logic".into(),
                mode: Some(super::super::symbol::PortMode::In),
            },
            "work",
            "foo",
        );

        // It should be visible from the architecture scope.
        let results = arena.lookup(arch, "clk");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "clk");

        // But not locally in the architecture.
        let local = arena.lookup_local(arch, "clk");
        assert!(local.is_empty());
    }

    #[test]
    fn import_symbol() {
        let mut arena = ScopeArena::new();
        let lib = arena.new_scope(ScopeKind::Library("work".into()), arena.root());
        let pkg = arena.new_scope(ScopeKind::Package("my_pkg".into()), lib);
        let sym = arena.add_symbol(
            pkg,
            "my_const".into(),
            SymbolKind::Constant {
                type_name: "integer".into(),
                deferred: false,
            },
            "work",
            "my_pkg",
        );

        // Import into a different scope.
        let arch = arena.new_scope(
            ScopeKind::Architecture {
                name: "rtl".into(),
                entity: "foo".into(),
            },
            lib,
        );
        arena.import_symbol(arch, "my_const".into(), sym);

        let results = arena.lookup(arch, "my_const");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "my_const");
    }

    #[test]
    fn shadowing() {
        let mut arena = ScopeArena::new();
        let lib = arena.new_scope(ScopeKind::Library("work".into()), arena.root());
        let entity = arena.new_scope(ScopeKind::Entity("foo".into()), lib);
        let arch = arena.new_scope(
            ScopeKind::Architecture {
                name: "rtl".into(),
                entity: "foo".into(),
            },
            entity,
        );

        // Declare in entity.
        arena.add_symbol(
            entity,
            "x".into(),
            SymbolKind::Constant {
                type_name: "integer".into(),
                deferred: false,
            },
            "work",
            "foo",
        );
        // Shadow in architecture.
        arena.add_symbol(
            arch,
            "x".into(),
            SymbolKind::Signal {
                type_name: "std_logic".into(),
                mode: None,
            },
            "work",
            "foo",
        );

        // Architecture lookup should find the shadowed version.
        let results = arena.lookup(arch, "x");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].kind, SymbolKind::Signal { .. }));
    }
}
