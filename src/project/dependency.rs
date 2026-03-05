//! Dependency graph construction and topological ordering.
//!
//! Given a populated [`Workspace`], this module builds a directed acyclic
//! graph (DAG) of design-unit dependencies by inspecting `library` and
//! `use` context clauses.  A topological sort then yields the correct
//! compilation order, or reports a cycle if one exists.

use std::collections::HashMap;

use super::library::{AnalyzedUnit, DesignUnitKind};
use super::workspace::Workspace;

use crate::ast::common::Identifier;
use crate::ast::context::ContextItem;
use crate::ast::name::{Name, Prefix, SelectedName, Suffix};

/// Globally unique reference to a design unit across all libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalUnitRef {
    /// Library name (lowercased).
    pub library: String,
    /// Unit index within that library.
    pub index: usize,
}

/// An edge in the dependency graph.
#[derive(Debug, Clone)]
pub struct DepEdge {
    /// The unit that is depended upon.
    pub dependency: GlobalUnitRef,
    /// Human-readable reason (e.g. `"use ieee.std_logic_1164.all"`).
    pub reason: String,
}

/// Dependency analysis error.
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// A circular dependency was detected.
    Cycle(Vec<GlobalUnitRef>),
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::Cycle(path) => {
                write!(f, "circular dependency detected: ")?;
                for (i, r) in path.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{}.#{}", r.library, r.index)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// The full dependency graph for a workspace.
#[derive(Debug)]
pub struct DependencyGraph {
    /// All units in the graph, in discovery order.
    pub units: Vec<GlobalUnitRef>,
    /// Adjacency list: for each unit, the list of units it depends on.
    pub edges: HashMap<GlobalUnitRef, Vec<DepEdge>>,
}

impl DependencyGraph {
    /// Build a dependency graph from an entire workspace.
    ///
    /// The graph captures:
    /// - `use lib.pkg.xxx` → dependency on `pkg` in library `lib`.
    /// - Architecture / PackageBody → implicit dependency on the corresponding
    ///   primary unit (Entity / Package) in the same library.
    pub fn build(workspace: &Workspace) -> Self {
        let mut units = Vec::new();
        let mut edges: HashMap<GlobalUnitRef, Vec<DepEdge>> = HashMap::new();

        // Build an index: (library, unit_name, kind_tag) -> GlobalUnitRef
        // to resolve symbolic references.
        let mut name_index: HashMap<(String, String, &str), GlobalUnitRef> = HashMap::new();

        for lib in workspace.libraries() {
            for unit in lib.iter() {
                let gref = GlobalUnitRef {
                    library: lib.name.clone(),
                    index: unit.id.0,
                };
                units.push(gref.clone());

                let tag = kind_tag(&unit.kind);
                name_index.insert((lib.name.clone(), unit.name.clone(), tag), gref.clone());

                edges.entry(gref).or_default();
            }
        }

        // Second pass: resolve dependencies.
        for lib in workspace.libraries() {
            for unit in lib.iter() {
                let gref = GlobalUnitRef {
                    library: lib.name.clone(),
                    index: unit.id.0,
                };
                let mut deps = Vec::new();

                // 1. Implicit structural dependencies.
                add_implicit_deps(&gref, unit, &name_index, &mut deps);

                // 2. Context clause dependencies.
                add_context_clause_deps(unit, &lib.name, &name_index, &mut deps);

                if !deps.is_empty() {
                    edges.entry(gref).or_default().extend(deps);
                }
            }
        }

        DependencyGraph { units, edges }
    }

    /// Perform a topological sort (Kahn's algorithm) and return the units in
    /// compilation order (dependencies first).
    ///
    /// # Errors
    ///
    /// Returns a [`DependencyError::Cycle`] if the graph contains a cycle.
    pub fn topological_sort(&self) -> Result<Vec<GlobalUnitRef>, DependencyError> {
        // Build in-degree counts.
        let mut in_degree: HashMap<GlobalUnitRef, usize> = HashMap::new();
        for u in &self.units {
            in_degree.entry(u.clone()).or_insert(0);
        }
        for deps in self.edges.values() {
            for dep in deps {
                // Only count edges whose target is actually in the graph.
                if in_degree.contains_key(&dep.dependency) {
                    *in_degree.entry(dep.dependency.clone()).or_insert(0) += 0;
                }
            }
        }

        // Build a forward adjacency list (dependee -> dependants).
        let mut forward: HashMap<GlobalUnitRef, Vec<GlobalUnitRef>> = HashMap::new();
        for (unit, deps) in &self.edges {
            for dep in deps {
                if in_degree.contains_key(&dep.dependency) {
                    forward
                        .entry(dep.dependency.clone())
                        .or_default()
                        .push(unit.clone());
                    *in_degree.entry(unit.clone()).or_insert(0) += 1;
                }
            }
        }

        // Seed the queue with zero in-degree units.
        let mut queue: std::collections::VecDeque<GlobalUnitRef> = in_degree
            .iter()
            .filter_map(|(u, &deg)| if deg == 0 { Some(u.clone()) } else { None })
            .collect();

        let mut sorted = Vec::with_capacity(self.units.len());

        while let Some(u) = queue.pop_front() {
            sorted.push(u.clone());
            if let Some(dependants) = forward.get(&u) {
                for dep in dependants {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        if sorted.len() != self.units.len() {
            // There is at least one cycle — collect the remaining nodes.
            let remaining: Vec<GlobalUnitRef> = self
                .units
                .iter()
                .filter(|u| !sorted.contains(u))
                .cloned()
                .collect();
            return Err(DependencyError::Cycle(remaining));
        }

        Ok(sorted)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Return a tag string that discriminates between primary/secondary unit
/// kinds so that we can look up the correct unit by name.
fn kind_tag(kind: &DesignUnitKind) -> &'static str {
    match kind {
        DesignUnitKind::Entity => "entity",
        DesignUnitKind::Architecture { .. } => "architecture",
        DesignUnitKind::Package => "package",
        DesignUnitKind::PackageBody => "package_body",
        DesignUnitKind::Configuration { .. } => "configuration",
        DesignUnitKind::PackageInstantiation => "package_instantiation",
        DesignUnitKind::Context => "context",
    }
}

/// Add implicit structural dependencies.
///
/// - An Architecture implicitly depends on its Entity.
/// - A PackageBody implicitly depends on its Package declaration.
/// - A Configuration implicitly depends on its Entity.
fn add_implicit_deps(
    _gref: &GlobalUnitRef,
    unit: &AnalyzedUnit,
    index: &HashMap<(String, String, &str), GlobalUnitRef>,
    deps: &mut Vec<DepEdge>,
) {
    let lib = &_gref.library;
    match &unit.kind {
        DesignUnitKind::Architecture { entity_name } => {
            if let Some(target) = index.get(&(lib.clone(), entity_name.clone(), "entity")) {
                deps.push(DepEdge {
                    dependency: target.clone(),
                    reason: format!("architecture of entity '{}'", entity_name),
                });
            }
        }
        DesignUnitKind::PackageBody => {
            if let Some(target) = index.get(&(lib.clone(), unit.name.clone(), "package")) {
                deps.push(DepEdge {
                    dependency: target.clone(),
                    reason: format!("body of package '{}'", unit.name),
                });
            }
        }
        DesignUnitKind::Configuration { entity_name } => {
            if let Some(target) = index.get(&(lib.clone(), entity_name.clone(), "entity")) {
                deps.push(DepEdge {
                    dependency: target.clone(),
                    reason: format!("configuration of entity '{}'", entity_name),
                });
            }
        }
        _ => {}
    }
}

/// Walk the context clause (`library …; use …;`) and add edges for any
/// referenced packages that exist in the workspace.
fn add_context_clause_deps(
    unit: &AnalyzedUnit,
    current_lib: &str,
    index: &HashMap<(String, String, &str), GlobalUnitRef>,
    deps: &mut Vec<DepEdge>,
) {
    // Collect which library names are declared via `library` clauses.
    // If the user writes `library ieee;` that makes `ieee` a visible library
    // name.  The implicit library `work` is always visible.
    let mut visible_libs: HashMap<String, String> = HashMap::new();
    visible_libs.insert("work".into(), current_lib.into());

    for item in &unit.context_clause.items {
        match item {
            ContextItem::Library(lib_clause) => {
                for logical_name in &lib_clause.logical_names.names {
                    let name = ident_lower(logical_name);
                    visible_libs.insert(name.clone(), name);
                }
            }
            ContextItem::Use(use_clause) => {
                for selected_name in &use_clause.names {
                    // Selected names typically look like `lib.pkg.all` or
                    // `lib.pkg.symbol`.  We extract the library and package
                    // components.
                    if let Some((lib_ref, pkg_name)) =
                        extract_use_target(selected_name, &visible_libs)
                    {
                        // Try to find this package in the index.
                        if let Some(target) =
                            index.get(&(lib_ref.clone(), pkg_name.clone(), "package"))
                        {
                            deps.push(DepEdge {
                                dependency: target.clone(),
                                reason: format!("use {}.{}", lib_ref, pkg_name),
                            });
                        }
                    }
                }
            }
            ContextItem::ContextReference(_) => {
                // Context references (VHDL-2008) reference context
                // declarations, not packages. We could add edges for
                // them but that is a refinement for later.
            }
        }
    }
}

/// Given a `SelectedName` from a `use` clause, try to extract the resolved
/// library name and the package name.
///
/// Typical patterns:
///   `use ieee.std_logic_1164.all;`   → ("ieee", "std_logic_1164")
///   `use work.my_pkg.my_type;`       → ("work" / actual lib, "my_pkg")
///
/// The function returns `None` for names it cannot resolve (e.g. deeply
/// nested or unexpected shapes).
fn extract_use_target(
    sel: &SelectedName,
    visible_libs: &HashMap<String, String>,
) -> Option<(String, String)> {
    // A use-clause selected name has the shape:  lib . pkg [ . suffix ]
    // The outermost SelectedName is (prefix=lib.pkg, suffix=all/symbol).
    // We need to walk down to get the library and package names.

    // sel.prefix is either:
    //   Prefix::Name(Name::Selected(inner))   for lib.pkg.xxx
    //   Prefix::Name(Name::Simple(..))         for pkg.xxx (implicit work)

    match &sel.prefix {
        Prefix::Name(name) => match name.as_ref() {
            Name::Selected(inner) => {
                // inner is another SelectedName: lib . pkg
                // inner.prefix should be a simple name (the library)
                // inner.suffix should be a simple name (the package)
                let lib_name = prefix_to_ident(&inner.prefix)?;
                let pkg_name = suffix_to_ident(&inner.suffix)?;

                let resolved_lib = visible_libs.get(&lib_name)?;
                Some((resolved_lib.clone(), pkg_name))
            }
            Name::Simple(simple) => {
                // Just `pkg.something` → implicit "work"
                let pkg_name = ident_lower(&simple.identifier);
                let resolved_lib = visible_libs.get("work")?;
                Some((resolved_lib.clone(), pkg_name))
            }
            _ => None,
        },
        _ => None,
    }
}

fn prefix_to_ident(prefix: &Prefix) -> Option<String> {
    match prefix {
        Prefix::Name(name) => match name.as_ref() {
            Name::Simple(s) => Some(ident_lower(&s.identifier)),
            _ => None,
        },
        _ => None,
    }
}

fn suffix_to_ident(suffix: &Suffix) -> Option<String> {
    match suffix {
        Suffix::SimpleName(s) => Some(ident_lower(&s.identifier)),
        _ => None,
    }
}

fn ident_lower(id: &Identifier) -> String {
    match id {
        Identifier::Basic(s) => s.to_lowercase(),
        Identifier::Extended(s) => s.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VhdlVersion;

    /// Create a workspace with multiple design units and verify topological
    /// ordering.
    #[test]
    fn basic_topological_sort() {
        let mut ws = Workspace::new();

        // Package (no dependencies).
        ws.load_str(
            "pkg.vhd",
            "package my_pkg is end package my_pkg;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();

        // Entity that uses my_pkg.
        ws.load_str(
            "ent.vhd",
            "library work;\nuse work.my_pkg.all;\n\
             entity foo is end entity foo;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();

        // Architecture of foo.
        ws.load_str(
            "arch.vhd",
            "architecture rtl of foo is begin end architecture rtl;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();

        let graph = DependencyGraph::build(&ws);
        let order = graph.topological_sort().unwrap();

        // my_pkg must come before foo, foo must come before rtl.
        let pos = |lib: &str, idx: usize| {
            order
                .iter()
                .position(|r| r.library == lib && r.index == idx)
                .unwrap()
        };

        let lib = ws.library("work").unwrap();
        let pkg_idx = lib.iter().find(|u| u.name == "my_pkg").unwrap().id.0;
        let ent_idx = lib.iter().find(|u| u.name == "foo").unwrap().id.0;
        let arch_idx = lib.iter().find(|u| u.name == "rtl").unwrap().id.0;

        assert!(pos("work", pkg_idx) < pos("work", ent_idx));
        assert!(pos("work", ent_idx) < pos("work", arch_idx));
    }

    #[test]
    fn independent_units_sort_without_error() {
        let mut ws = Workspace::new();
        ws.load_str(
            "a.vhd",
            "entity a is end entity a;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();
        ws.load_str(
            "b.vhd",
            "entity b is end entity b;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();

        let graph = DependencyGraph::build(&ws);
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn package_body_depends_on_package() {
        let mut ws = Workspace::new();
        ws.load_str(
            "pkg.vhd",
            "package my_pkg is end package my_pkg;\n\
             package body my_pkg is end package body my_pkg;",
            VhdlVersion::Vhdl1993,
            None,
        )
        .unwrap();

        let graph = DependencyGraph::build(&ws);
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 2);

        let lib = ws.library("work").unwrap();
        let pkg_idx = lib
            .iter()
            .find(|u| {
                u.name == "my_pkg" && u.kind == crate::project::library::DesignUnitKind::Package
            })
            .unwrap()
            .id
            .0;
        let body_idx = lib
            .iter()
            .find(|u| u.kind == crate::project::library::DesignUnitKind::PackageBody)
            .unwrap()
            .id
            .0;

        let pos_pkg = order.iter().position(|r| r.index == pkg_idx).unwrap();
        let pos_body = order.iter().position(|r| r.index == body_idx).unwrap();
        assert!(pos_pkg < pos_body);
    }
}
