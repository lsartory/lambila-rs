//! VHDL Library abstraction.
//!
//! A library groups parsed design units that were analyzed into the same
//! logical name (e.g. `work`, `ieee`). Each design unit carries enough
//! metadata to reconstruct its identity for dependency resolution.

use crate::VhdlVersion;
use crate::ast::context::ContextClause;
use crate::ast::design_unit::{DesignUnit, LibraryUnit, PrimaryUnit, SecondaryUnit};

use super::source::SourceId;

/// An opaque handle that uniquely identifies a design unit within a
/// [`Library`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DesignUnitId(pub(crate) usize);

/// The kind of a design unit, stripped to the minimum needed for dependency
/// resolution (we don't need the full AST here, just the category and name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignUnitKind {
    /// `entity foo is … end;`
    Entity,
    /// `architecture bar of foo is … end;`
    Architecture {
        /// The entity this architecture belongs to.
        entity_name: String,
    },
    /// `package pkg is … end;`
    Package,
    /// `package body pkg is … end;`
    PackageBody,
    /// `configuration cfg of entity is … end;`
    Configuration {
        /// The entity this configuration targets.
        entity_name: String,
    },
    /// `package name is new …` (VHDL-2008)
    PackageInstantiation,
    /// `context ctx is … end;` (VHDL-2008)
    Context,
}

/// An analyzed design unit stored inside a library.
#[derive(Debug, Clone)]
pub struct AnalyzedUnit {
    /// Unique local identifier within the library.
    pub id: DesignUnitId,
    /// The primary name of this unit (e.g. entity name, package name, …).
    pub name: String,
    /// What kind of unit this is.
    pub kind: DesignUnitKind,
    /// The VHDL version used to lex/parse this unit.
    pub version: VhdlVersion,
    /// Which source file this unit came from.
    pub source_id: SourceId,
    /// The parsed context clause (library/use/context-reference).
    pub context_clause: ContextClause,
    /// The full parsed design unit AST.
    pub design_unit: DesignUnit,
}

/// A VHDL library — a named collection of analyzed design units.
#[derive(Debug, Clone)]
pub struct Library {
    /// Logical library name (e.g. `"work"`, `"ieee"`).
    pub name: String,
    /// All design units belonging to this library.
    units: Vec<AnalyzedUnit>,
}

impl Library {
    /// Create a new empty library with the given logical name.
    pub fn new(name: impl Into<String>) -> Self {
        Library {
            name: name.into(),
            units: Vec::new(),
        }
    }

    /// Add a parsed [`DesignUnit`] to this library.
    ///
    /// The unit is inspected to determine its name and kind.
    pub fn add_unit(
        &mut self,
        design_unit: DesignUnit,
        source_id: SourceId,
        version: VhdlVersion,
    ) -> DesignUnitId {
        let (name, kind) = extract_unit_identity(&design_unit.library_unit);
        let id = DesignUnitId(self.units.len());
        self.units.push(AnalyzedUnit {
            id,
            name,
            kind,
            version,
            source_id,
            context_clause: design_unit.context_clause.clone(),
            design_unit,
        });
        id
    }

    /// Look up a design unit by its local id.
    pub fn get(&self, id: DesignUnitId) -> &AnalyzedUnit {
        &self.units[id.0]
    }

    /// Iterate over all units in the library.
    pub fn iter(&self) -> impl Iterator<Item = &AnalyzedUnit> {
        self.units.iter()
    }

    /// Number of design units in this library.
    pub fn len(&self) -> usize {
        self.units.len()
    }

    /// Returns `true` if the library contains no design units.
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the primary name and kind from a `LibraryUnit`.
fn extract_unit_identity(unit: &LibraryUnit) -> (String, DesignUnitKind) {
    match unit {
        LibraryUnit::Primary(primary) => match primary {
            PrimaryUnit::Entity(e) => (ident_to_string(&e.identifier), DesignUnitKind::Entity),
            PrimaryUnit::Package(p) => (ident_to_string(&p.identifier), DesignUnitKind::Package),
            PrimaryUnit::Configuration(c) => (
                ident_to_string(&c.identifier),
                DesignUnitKind::Configuration {
                    entity_name: name_to_string(&c.entity_name),
                },
            ),
            PrimaryUnit::PackageInstantiation(pi) => (
                ident_to_string(&pi.identifier),
                DesignUnitKind::PackageInstantiation,
            ),
            PrimaryUnit::Context(ctx) => {
                (ident_to_string(&ctx.identifier), DesignUnitKind::Context)
            }
        },
        LibraryUnit::Secondary(secondary) => match secondary {
            SecondaryUnit::Architecture(a) => (
                ident_to_string(&a.identifier),
                DesignUnitKind::Architecture {
                    entity_name: ident_to_string(&a.entity_name.identifier),
                },
            ),
            SecondaryUnit::PackageBody(pb) => (
                ident_to_string(&pb.name.identifier),
                DesignUnitKind::PackageBody,
            ),
        },
    }
}

/// Convert an `Identifier` to a lowercase `String`.
fn ident_to_string(id: &crate::ast::common::Identifier) -> String {
    match id {
        crate::ast::common::Identifier::Basic(s) => s.to_lowercase(),
        crate::ast::common::Identifier::Extended(s) => s.clone(),
    }
}

/// Convert a `SimpleName` to its lowercase string representation.
fn name_to_string(name: &crate::ast::common::SimpleName) -> String {
    ident_to_string(&name.identifier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{VhdlVersion, parse_str};

    #[test]
    fn add_entity_unit() {
        let design_file =
            parse_str("entity foo is end entity foo;", VhdlVersion::Vhdl1993).unwrap();
        let unit = design_file.design_units.into_iter().next().unwrap();
        let mut lib = Library::new("work");
        let id = lib.add_unit(unit, SourceId(0), VhdlVersion::Vhdl1993);
        let analyzed = lib.get(id);
        assert_eq!(analyzed.name, "foo");
        assert_eq!(analyzed.kind, DesignUnitKind::Entity);
    }

    #[test]
    fn add_architecture_unit() {
        let src = "architecture rtl of foo is begin end architecture rtl;";
        let design_file = parse_str(src, VhdlVersion::Vhdl1993).unwrap();
        let unit = design_file.design_units.into_iter().next().unwrap();
        let mut lib = Library::new("work");
        let id = lib.add_unit(unit, SourceId(0), VhdlVersion::Vhdl1993);
        let analyzed = lib.get(id);
        assert_eq!(analyzed.name, "rtl");
        assert_eq!(
            analyzed.kind,
            DesignUnitKind::Architecture {
                entity_name: "foo".into()
            }
        );
    }

    #[test]
    fn add_package_and_body() {
        let src = "package my_pkg is end package my_pkg;\n\
                   package body my_pkg is end package body my_pkg;";
        let design_file = parse_str(src, VhdlVersion::Vhdl1993).unwrap();
        let mut lib = Library::new("work");
        for unit in design_file.design_units {
            lib.add_unit(unit, SourceId(0), VhdlVersion::Vhdl1993);
        }
        assert_eq!(lib.len(), 2);
        let kinds: Vec<_> = lib.iter().map(|u| &u.kind).collect();
        assert_eq!(kinds[0], &DesignUnitKind::Package);
        assert_eq!(kinds[1], &DesignUnitKind::PackageBody);
    }
}
