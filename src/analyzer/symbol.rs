//! Semantic symbol definitions.
//!
//! These structures represent the *meaning* of VHDL declarations after
//! name resolution, as opposed to the raw AST which only captures syntax.

use crate::ast::common::Identifier;

/// A unique identifier for a symbol within the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

/// The semantic category of a declared symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// A VHDL type (enumeration, integer, array, record, etc.).
    Type(TypeInfo),
    /// A subtype declaration.
    Subtype {
        /// The base type this subtype constrains.
        base_type: String,
    },
    /// A constant object.
    Constant {
        type_name: String,
        /// True if this is a deferred constant (declared in a package
        /// specification without an initialiser).
        deferred: bool,
    },
    /// A signal object.
    Signal {
        type_name: String,
        mode: Option<PortMode>,
    },
    /// A variable object.
    Variable { type_name: String, shared: bool },
    /// A file object.
    File { type_name: String },
    /// A component declaration.
    Component {
        /// Port names declared on this component.
        ports: Vec<String>,
        /// Generic names declared on this component.
        generics: Vec<String>,
    },
    /// An entity (primary design unit).
    Entity {
        ports: Vec<String>,
        generics: Vec<String>,
    },
    /// An architecture (secondary design unit).
    Architecture { entity_name: String },
    /// A package (primary design unit).
    Package,
    /// A package body (secondary design unit).
    PackageBody,
    /// A subprogram (function or procedure).
    Subprogram(SubprogramInfo),
    /// An alias declaration.
    Alias {
        /// The name this alias refers to.
        designator: String,
    },
    /// An enumeration literal.
    EnumLiteral {
        /// The type this literal belongs to.
        type_name: String,
    },
    /// A label (for processes, blocks, generates).
    Label,
    /// An attribute declaration.
    Attribute { type_name: String },
    /// A configuration (primary design unit).
    Configuration { entity_name: String },
}

/// Port mode for signals declared on interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
    Linkage,
}

/// Information about a VHDL type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    /// `type t is (a, b, c);`
    Enumeration { literals: Vec<String> },
    /// `type t is range 0 to 255;`
    Integer,
    /// `type t is range 0.0 to 1.0;`
    Floating,
    /// `type t is range ... units ... end units;`
    Physical { primary_unit: String },
    /// `type t is array (...) of element;`
    Array {
        element_type: String,
        dimensions: usize,
    },
    /// `type t is record ... end record;`
    Record { fields: Vec<(String, String)> },
    /// `type t is access designator;`
    Access { designated_type: String },
    /// `type t is file of type_mark;`
    FileType { type_mark: String },
    /// `type t; ` (incomplete / forward declaration)
    Incomplete,
}

/// Information about a subprogram declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubprogramInfo {
    Function {
        /// Parameter names and their type names.
        parameters: Vec<(String, String)>,
        /// Return type name.
        return_type: String,
        pure: bool,
    },
    Procedure {
        parameters: Vec<(String, String)>,
    },
}

/// A resolved symbol entry stored in a scope.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    /// The declared name (lowercased for case-insensitive lookup).
    pub name: String,
    /// What kind of declaration this symbol represents.
    pub kind: SymbolKind,
    /// For error reporting: which library this symbol was declared in.
    pub library: String,
    /// For error reporting: which design unit this symbol was declared in.
    pub unit_name: String,
}

/// Extract the lowercase string from an AST [`Identifier`].
pub fn ident_to_lower(id: &Identifier) -> String {
    match id {
        Identifier::Basic(s) => s.to_lowercase(),
        Identifier::Extended(s) => s.clone(),
    }
}
