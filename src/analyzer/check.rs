//! AST-walking semantic checker.
//!
//! This module traverses the parsed design units in topological order,
//! populating the scope arena with symbol declarations. It also performs
//! basic validation such as duplicate declaration detection and ensures
//! that ports and generics defined on entities are correctly recorded.

use crate::ast::architecture::ArchitectureBody;
use crate::ast::common::Identifier;
use crate::ast::concurrent::BlockDeclarativeItem;
use crate::ast::context::UseClause;
use crate::ast::design_unit::{LibraryUnit, PrimaryUnit, SecondaryUnit};
use crate::ast::entity::EntityDeclaration;
use crate::ast::interface::{
    GenericClause, InterfaceDeclaration, InterfaceObjectDeclaration,
    InterfaceSubprogramSpecification, PortClause,
};
use crate::ast::name::{Name, Suffix};
use crate::ast::object_decl::{ConstantDeclaration, SignalDeclaration};
use crate::ast::package::{PackageBody, PackageDeclaration};
use crate::ast::type_def::{SubtypeDeclaration, TypeDeclaration, TypeDefinition, TypeMark};
use crate::project::library::{AnalyzedUnit, DesignUnitKind};
use crate::project::workspace::Workspace;

use super::SemanticError;
use super::scope::{ScopeArena, ScopeId, ScopeKind};
use super::symbol::{PortMode, SymbolKind, TypeInfo};

/// Walk all design units in a workspace and populate the scope arena.
pub fn analyze_workspace(
    workspace: &Workspace,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    for lib in workspace.libraries() {
        let lib_scope = arena.new_scope(ScopeKind::Library(lib.name.clone()), arena.root());

        for unit in lib.iter() {
            analyze_unit(unit, &lib.name, lib_scope, arena, errors);
        }
    }
}

/// Analyze a single design unit.
fn analyze_unit(
    unit: &AnalyzedUnit,
    lib_name: &str,
    lib_scope: ScopeId,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    // Process use clauses in the context to import symbols.
    for ctx_item in &unit.design_unit.context_clause.items {
        if let crate::ast::context::ContextItem::Use(use_clause) = ctx_item {
            process_use_clause(use_clause, lib_scope, arena);
        }
    }

    match &unit.design_unit.library_unit {
        LibraryUnit::Primary(primary) => match primary {
            PrimaryUnit::Entity(entity) => {
                analyze_entity(entity, lib_name, &unit.name, lib_scope, arena, errors);
            }
            PrimaryUnit::Package(pkg) => {
                analyze_package(pkg, lib_name, &unit.name, lib_scope, arena, errors);
            }
            PrimaryUnit::Configuration(_) => {
                if let DesignUnitKind::Configuration { entity_name } = &unit.kind {
                    arena.add_symbol(
                        lib_scope,
                        unit.name.clone(),
                        SymbolKind::Configuration {
                            entity_name: entity_name.clone(),
                        },
                        lib_name,
                        &unit.name,
                    );
                }
            }
            PrimaryUnit::Context(_) => {}
            PrimaryUnit::PackageInstantiation(_) => {
                arena.add_symbol(
                    lib_scope,
                    unit.name.clone(),
                    SymbolKind::Package,
                    lib_name,
                    &unit.name,
                );
            }
        },
        LibraryUnit::Secondary(secondary) => match secondary {
            SecondaryUnit::Architecture(arch) => {
                analyze_architecture(arch, lib_name, &unit.name, lib_scope, arena, errors);
            }
            SecondaryUnit::PackageBody(body) => {
                analyze_package_body(body, lib_name, &unit.name, lib_scope, arena, errors);
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Entity analysis
// ---------------------------------------------------------------------------

fn analyze_entity(
    entity: &EntityDeclaration,
    lib_name: &str,
    unit_name: &str,
    lib_scope: ScopeId,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    let entity_scope = arena.new_scope(ScopeKind::Entity(unit_name.to_string()), lib_scope);

    let mut port_names = Vec::new();
    let mut generic_names = Vec::new();

    // Generics.
    if let Some(ref gc) = entity.header.generic_clause {
        register_generics(
            gc,
            entity_scope,
            lib_name,
            unit_name,
            arena,
            &mut generic_names,
        );
    }

    // Ports.
    if let Some(ref pc) = entity.header.port_clause {
        register_ports(
            pc,
            entity_scope,
            lib_name,
            unit_name,
            arena,
            &mut port_names,
        );
    }

    // Entity declarative items.
    for item in &entity.declarative_part.items {
        analyze_entity_decl_item(item, entity_scope, lib_name, unit_name, arena, errors);
    }

    // Register the entity symbol in the library scope.
    arena.add_symbol(
        lib_scope,
        unit_name.to_string(),
        SymbolKind::Entity {
            ports: port_names,
            generics: generic_names,
        },
        lib_name,
        unit_name,
    );
}

fn analyze_entity_decl_item(
    item: &crate::ast::entity::EntityDeclarativeItem,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    use crate::ast::entity::EntityDeclarativeItem;
    match item {
        EntityDeclarativeItem::SignalDeclaration(sig) => {
            register_signal(sig, scope, lib_name, unit_name, arena);
        }
        EntityDeclarativeItem::ConstantDeclaration(c) => {
            register_constant(c, scope, lib_name, unit_name, arena);
        }
        EntityDeclarativeItem::TypeDeclaration(td) => {
            register_type_decl(td, scope, lib_name, unit_name, arena, errors);
        }
        EntityDeclarativeItem::SubtypeDeclaration(sd) => {
            register_subtype_decl(sd, scope, lib_name, unit_name, arena);
        }
        EntityDeclarativeItem::UseClause(uc) => {
            process_use_clause(uc, scope, arena);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Architecture analysis
// ---------------------------------------------------------------------------

fn analyze_architecture(
    arch: &ArchitectureBody,
    lib_name: &str,
    unit_name: &str,
    lib_scope: ScopeId,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    let entity_name = ident_to_lower(&arch.entity_name.identifier);

    let parent = arena
        .find_scope(|k| matches!(k, ScopeKind::Entity(n) if n == &entity_name))
        .unwrap_or(lib_scope);

    let arch_scope = arena.new_scope(
        ScopeKind::Architecture {
            name: unit_name.to_string(),
            entity: entity_name.clone(),
        },
        parent,
    );

    for item in &arch.declarative_part.items {
        analyze_block_decl_item(item, arch_scope, lib_name, unit_name, arena, errors);
    }

    arena.add_symbol(
        lib_scope,
        unit_name.to_string(),
        SymbolKind::Architecture { entity_name },
        lib_name,
        unit_name,
    );
}

fn analyze_block_decl_item(
    item: &BlockDeclarativeItem,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    match item {
        BlockDeclarativeItem::SignalDeclaration(sig) => {
            register_signal(sig, scope, lib_name, unit_name, arena);
        }
        BlockDeclarativeItem::ConstantDeclaration(c) => {
            register_constant(c, scope, lib_name, unit_name, arena);
        }
        BlockDeclarativeItem::TypeDeclaration(td) => {
            register_type_decl(td, scope, lib_name, unit_name, arena, errors);
        }
        BlockDeclarativeItem::SubtypeDeclaration(sd) => {
            register_subtype_decl(sd, scope, lib_name, unit_name, arena);
        }
        BlockDeclarativeItem::ComponentDeclaration(comp) => {
            let comp_name = ident_to_lower(&comp.identifier);
            let ports = comp
                .port_clause
                .as_ref()
                .map(extract_port_names)
                .unwrap_or_default();
            let generics = comp
                .generic_clause
                .as_ref()
                .map(extract_generic_names)
                .unwrap_or_default();
            arena.add_symbol(
                scope,
                comp_name,
                SymbolKind::Component { ports, generics },
                lib_name,
                unit_name,
            );
        }
        BlockDeclarativeItem::FileDeclaration(fd) => {
            for ident in &fd.identifiers.identifiers {
                let name = ident_to_lower(ident);
                let type_name = subtype_indication_name(&fd.subtype_indication);
                arena.add_symbol(
                    scope,
                    name,
                    SymbolKind::File { type_name },
                    lib_name,
                    unit_name,
                );
            }
        }
        BlockDeclarativeItem::AliasDeclaration(alias) => {
            let designator = match &alias.designator {
                crate::ast::object_decl::AliasDesignator::Identifier(id) => ident_to_lower(id),
                crate::ast::object_decl::AliasDesignator::CharacterLiteral(c) => c.clone(),
                crate::ast::object_decl::AliasDesignator::OperatorSymbol(op) => {
                    op.text.to_lowercase()
                }
            };
            arena.add_symbol(
                scope,
                designator.clone(),
                SymbolKind::Alias { designator },
                lib_name,
                unit_name,
            );
        }
        BlockDeclarativeItem::SharedVariableDeclaration(var) => {
            for ident in &var.identifiers.identifiers {
                let name = ident_to_lower(ident);
                let type_name = subtype_indication_name(&var.subtype_indication);
                arena.add_symbol(
                    scope,
                    name,
                    SymbolKind::Variable {
                        type_name,
                        shared: true,
                    },
                    lib_name,
                    unit_name,
                );
            }
        }
        BlockDeclarativeItem::AttributeDeclaration(attr) => {
            let name = ident_to_lower(&attr.identifier);
            let type_name = type_mark_name(&attr.type_mark);
            arena.add_symbol(
                scope,
                name,
                SymbolKind::Attribute { type_name },
                lib_name,
                unit_name,
            );
        }
        BlockDeclarativeItem::UseClause(uc) => {
            process_use_clause(uc, scope, arena);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Package analysis
// ---------------------------------------------------------------------------

fn analyze_package(
    pkg: &PackageDeclaration,
    lib_name: &str,
    unit_name: &str,
    lib_scope: ScopeId,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    let pkg_scope = arena.new_scope(ScopeKind::Package(unit_name.to_string()), lib_scope);

    for item in &pkg.declarative_part.items {
        analyze_pkg_decl_item(item, pkg_scope, lib_name, unit_name, arena, errors);
    }

    arena.add_symbol(
        lib_scope,
        unit_name.to_string(),
        SymbolKind::Package,
        lib_name,
        unit_name,
    );
}

fn analyze_pkg_decl_item(
    item: &crate::ast::package::PackageDeclarativeItem,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    use crate::ast::package::PackageDeclarativeItem;
    match item {
        PackageDeclarativeItem::SignalDeclaration(sig) => {
            register_signal(sig, scope, lib_name, unit_name, arena);
        }
        PackageDeclarativeItem::ConstantDeclaration(c) => {
            register_constant(c, scope, lib_name, unit_name, arena);
        }
        PackageDeclarativeItem::TypeDeclaration(td) => {
            register_type_decl(td, scope, lib_name, unit_name, arena, errors);
        }
        PackageDeclarativeItem::SubtypeDeclaration(sd) => {
            register_subtype_decl(sd, scope, lib_name, unit_name, arena);
        }
        PackageDeclarativeItem::ComponentDeclaration(comp) => {
            let comp_name = ident_to_lower(&comp.identifier);
            let ports = comp
                .port_clause
                .as_ref()
                .map(extract_port_names)
                .unwrap_or_default();
            let generics = comp
                .generic_clause
                .as_ref()
                .map(extract_generic_names)
                .unwrap_or_default();
            arena.add_symbol(
                scope,
                comp_name,
                SymbolKind::Component { ports, generics },
                lib_name,
                unit_name,
            );
        }
        PackageDeclarativeItem::UseClause(uc) => {
            process_use_clause(uc, scope, arena);
        }
        _ => {}
    }
}

fn analyze_package_body(
    body: &PackageBody,
    lib_name: &str,
    unit_name: &str,
    lib_scope: ScopeId,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    let parent = arena
        .find_scope(|k| matches!(k, ScopeKind::Package(n) if n == unit_name))
        .unwrap_or(lib_scope);

    let body_scope = arena.new_scope(ScopeKind::PackageBody(unit_name.to_string()), parent);

    for item in &body.declarative_part.items {
        analyze_pkg_body_decl_item(item, body_scope, lib_name, unit_name, arena, errors);
    }

    arena.add_symbol(
        lib_scope,
        unit_name.to_string(),
        SymbolKind::PackageBody,
        lib_name,
        unit_name,
    );
}

fn analyze_pkg_body_decl_item(
    item: &crate::ast::package::PackageBodyDeclarativeItem,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    errors: &mut Vec<SemanticError>,
) {
    use crate::ast::package::PackageBodyDeclarativeItem;
    match item {
        PackageBodyDeclarativeItem::ConstantDeclaration(c) => {
            register_constant(c, scope, lib_name, unit_name, arena);
        }
        PackageBodyDeclarativeItem::TypeDeclaration(td) => {
            register_type_decl(td, scope, lib_name, unit_name, arena, errors);
        }
        PackageBodyDeclarativeItem::SubtypeDeclaration(sd) => {
            register_subtype_decl(sd, scope, lib_name, unit_name, arena);
        }
        PackageBodyDeclarativeItem::SharedVariableDeclaration(var) => {
            for ident in &var.identifiers.identifiers {
                let name = ident_to_lower(ident);
                let type_name = subtype_indication_name(&var.subtype_indication);
                arena.add_symbol(
                    scope,
                    name,
                    SymbolKind::Variable {
                        type_name,
                        shared: true,
                    },
                    lib_name,
                    unit_name,
                );
            }
        }
        PackageBodyDeclarativeItem::FileDeclaration(fd) => {
            for ident in &fd.identifiers.identifiers {
                let name = ident_to_lower(ident);
                let type_name = subtype_indication_name(&fd.subtype_indication);
                arena.add_symbol(
                    scope,
                    name,
                    SymbolKind::File { type_name },
                    lib_name,
                    unit_name,
                );
            }
        }
        PackageBodyDeclarativeItem::UseClause(uc) => {
            process_use_clause(uc, scope, arena);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Use clause handling
// ---------------------------------------------------------------------------

fn process_use_clause(uc: &UseClause, scope: ScopeId, arena: &mut ScopeArena) {
    for selected_name in &uc.names {
        let suffix = suffix_to_lower(&selected_name.suffix);
        if suffix == "all" {
            // `use lib.pkg.all;`
            if let Some(pkg_name) = extract_prefix_simple_name(&selected_name.prefix) {
                let pkg_lower = pkg_name.to_lowercase();
                if let Some(pkg_scope_id) =
                    arena.find_scope(|k| matches!(k, ScopeKind::Package(n) if n == &pkg_lower))
                {
                    let sym_ids: Vec<_> = arena.scope(pkg_scope_id).direct_symbol_ids().collect();
                    for (name, id) in sym_ids {
                        arena.import_symbol(scope, name, id);
                    }
                }
            }
        } else {
            // `use lib.pkg.specific_item;`
            let item_lower = suffix;
            if let Some(pkg_name) = extract_prefix_simple_name(&selected_name.prefix) {
                let pkg_lower = pkg_name.to_lowercase();
                if let Some(pkg_scope_id) =
                    arena.find_scope(|k| matches!(k, ScopeKind::Package(n) if n == &pkg_lower))
                {
                    let syms = arena.lookup_local(pkg_scope_id, &item_lower);
                    let ids: Vec<_> = syms.iter().map(|s| s.id).collect();
                    for id in ids {
                        arena.import_symbol(scope, item_lower.clone(), id);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Declaration helpers
// ---------------------------------------------------------------------------

fn register_signal(
    sig: &SignalDeclaration,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
) {
    let type_name = subtype_indication_name(&sig.subtype_indication);
    for ident in &sig.identifiers.identifiers {
        let name = ident_to_lower(ident);
        arena.add_symbol(
            scope,
            name,
            SymbolKind::Signal {
                type_name: type_name.clone(),
                mode: None,
            },
            lib_name,
            unit_name,
        );
    }
}

fn register_constant(
    c: &ConstantDeclaration,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
) {
    let type_name = subtype_indication_name(&c.subtype_indication);
    let deferred = c.default_expression.is_none();
    for ident in &c.identifiers.identifiers {
        let name = ident_to_lower(ident);
        arena.add_symbol(
            scope,
            name,
            SymbolKind::Constant {
                type_name: type_name.clone(),
                deferred,
            },
            lib_name,
            unit_name,
        );
    }
}

fn register_type_decl(
    td: &TypeDeclaration,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    _errors: &mut Vec<SemanticError>,
) {
    match td {
        TypeDeclaration::Full(full) => {
            let type_name = ident_to_lower(&full.identifier);
            let info = type_def_to_info(&full.type_definition);

            // If it's an enumeration, also register the literals.
            if let TypeInfo::Enumeration { ref literals } = info {
                for lit in literals {
                    arena.add_symbol(
                        scope,
                        lit.clone(),
                        SymbolKind::EnumLiteral {
                            type_name: type_name.clone(),
                        },
                        lib_name,
                        unit_name,
                    );
                }
            }

            arena.add_symbol(
                scope,
                type_name,
                SymbolKind::Type(info),
                lib_name,
                unit_name,
            );
        }
        TypeDeclaration::Incomplete(inc) => {
            let type_name = ident_to_lower(&inc.identifier);
            arena.add_symbol(
                scope,
                type_name,
                SymbolKind::Type(TypeInfo::Incomplete),
                lib_name,
                unit_name,
            );
        }
    }
}

fn register_subtype_decl(
    sd: &SubtypeDeclaration,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
) {
    let name = ident_to_lower(&sd.identifier);
    let base = subtype_indication_name(&sd.subtype_indication);
    arena.add_symbol(
        scope,
        name,
        SymbolKind::Subtype { base_type: base },
        lib_name,
        unit_name,
    );
}

fn register_generics(
    gc: &GenericClause,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    generic_names: &mut Vec<String>,
) {
    for decl in &gc.generic_list.elements {
        let names = interface_decl_names(decl);
        for name in &names {
            arena.add_symbol(
                scope,
                name.clone(),
                SymbolKind::Constant {
                    type_name: interface_decl_type_name(decl),
                    deferred: false,
                },
                lib_name,
                unit_name,
            );
        }
        generic_names.extend(names);
    }
}

fn register_ports(
    pc: &PortClause,
    scope: ScopeId,
    lib_name: &str,
    unit_name: &str,
    arena: &mut ScopeArena,
    port_names: &mut Vec<String>,
) {
    for decl in &pc.port_list.elements {
        let mode = interface_decl_mode(decl);
        let type_name = interface_decl_type_name(decl);
        let names = interface_decl_names(decl);
        for n in &names {
            arena.add_symbol(
                scope,
                n.clone(),
                SymbolKind::Signal {
                    type_name: type_name.clone(),
                    mode: Some(mode),
                },
                lib_name,
                unit_name,
            );
        }
        port_names.extend(names);
    }
}

fn extract_port_names(pc: &PortClause) -> Vec<String> {
    pc.port_list
        .elements
        .iter()
        .flat_map(interface_decl_names)
        .collect()
}

fn extract_generic_names(gc: &GenericClause) -> Vec<String> {
    gc.generic_list
        .elements
        .iter()
        .flat_map(interface_decl_names)
        .collect()
}

// ---------------------------------------------------------------------------
// AST helper functions
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

fn extract_prefix_simple_name(prefix: &crate::ast::name::Prefix) -> Option<String> {
    match prefix {
        crate::ast::name::Prefix::Name(name) => match name.as_ref() {
            Name::Simple(s) => Some(ident_to_lower(&s.identifier)),
            Name::Selected(sel) => Some(suffix_to_lower(&sel.suffix)),
            _ => None,
        },
        _ => None,
    }
}

fn interface_decl_names(decl: &InterfaceDeclaration) -> Vec<String> {
    match decl {
        InterfaceDeclaration::Object(obj) => match obj {
            InterfaceObjectDeclaration::Constant(c) => c
                .identifiers
                .identifiers
                .iter()
                .map(ident_to_lower)
                .collect(),
            InterfaceObjectDeclaration::Signal(s) => s
                .identifiers
                .identifiers
                .iter()
                .map(ident_to_lower)
                .collect(),
            InterfaceObjectDeclaration::Variable(v) => v
                .identifiers
                .identifiers
                .iter()
                .map(ident_to_lower)
                .collect(),
            InterfaceObjectDeclaration::File(f) => f
                .identifiers
                .identifiers
                .iter()
                .map(ident_to_lower)
                .collect(),
        },
        InterfaceDeclaration::Type(td) => vec![ident_to_lower(&td.identifier)],
        InterfaceDeclaration::Subprogram(sd) => {
            vec![interface_subprogram_spec_name(&sd.specification)]
        }
        InterfaceDeclaration::Package(pd) => vec![ident_to_lower(&pd.identifier)],
    }
}

fn interface_decl_mode(decl: &InterfaceDeclaration) -> PortMode {
    match decl {
        InterfaceDeclaration::Object(obj) => match obj {
            InterfaceObjectDeclaration::Signal(s) => match s.mode {
                Some(crate::ast::common::Mode::In) | None => PortMode::In,
                Some(crate::ast::common::Mode::Out) => PortMode::Out,
                Some(crate::ast::common::Mode::InOut) => PortMode::InOut,
                Some(crate::ast::common::Mode::Buffer) => PortMode::Buffer,
                Some(crate::ast::common::Mode::Linkage) => PortMode::Linkage,
            },
            InterfaceObjectDeclaration::Variable(v) => match v.mode {
                Some(crate::ast::common::Mode::In) | None => PortMode::In,
                Some(crate::ast::common::Mode::Out) => PortMode::Out,
                Some(crate::ast::common::Mode::InOut) => PortMode::InOut,
                Some(crate::ast::common::Mode::Buffer) => PortMode::Buffer,
                Some(crate::ast::common::Mode::Linkage) => PortMode::Linkage,
            },
            _ => PortMode::In,
        },
        _ => PortMode::In,
    }
}

fn interface_decl_type_name(decl: &InterfaceDeclaration) -> String {
    match decl {
        InterfaceDeclaration::Object(obj) => match obj {
            InterfaceObjectDeclaration::Constant(c) => {
                subtype_indication_name(&c.subtype_indication)
            }
            InterfaceObjectDeclaration::Signal(s) => subtype_indication_name(&s.subtype_indication),
            InterfaceObjectDeclaration::Variable(v) => {
                subtype_indication_name(&v.subtype_indication)
            }
            InterfaceObjectDeclaration::File(f) => subtype_indication_name(&f.subtype_indication),
        },
        InterfaceDeclaration::Type(td) => ident_to_lower(&td.identifier),
        InterfaceDeclaration::Subprogram(sd) => interface_subprogram_spec_name(&sd.specification),
        InterfaceDeclaration::Package(pd) => ident_to_lower(&pd.identifier),
    }
}

fn interface_subprogram_spec_name(spec: &InterfaceSubprogramSpecification) -> String {
    match spec {
        InterfaceSubprogramSpecification::Procedure(p) => designator_to_lower(&p.designator),
        InterfaceSubprogramSpecification::Function(f) => designator_to_lower(&f.designator),
    }
}

fn subtype_indication_name(si: &crate::ast::type_def::SubtypeIndication) -> String {
    type_mark_name(&si.type_mark)
}

fn type_mark_name(tm: &TypeMark) -> String {
    match tm {
        TypeMark::TypeName(name) | TypeMark::SubtypeName(name) => name_to_lower(name),
    }
}

fn name_to_lower(name: &Name) -> String {
    match name {
        Name::Simple(s) => ident_to_lower(&s.identifier),
        Name::Selected(sel) => suffix_to_lower(&sel.suffix),
        _ => "<complex>".to_string(),
    }
}

fn designator_to_lower(d: &crate::ast::common::Designator) -> String {
    match d {
        crate::ast::common::Designator::Identifier(id) => ident_to_lower(id),
        crate::ast::common::Designator::OperatorSymbol(op) => op.text.to_lowercase(),
    }
}

fn type_def_to_info(td: &TypeDefinition) -> TypeInfo {
    match td {
        TypeDefinition::Scalar(scalar) => match scalar {
            crate::ast::type_def::ScalarTypeDefinition::Enumeration(e) => {
                let literals: Vec<String> = e
                    .literals
                    .iter()
                    .map(|lit| match lit {
                        crate::ast::literal::EnumerationLiteral::Identifier(id) => {
                            ident_to_lower(id)
                        }
                        crate::ast::literal::EnumerationLiteral::CharacterLiteral(c) => c.clone(),
                    })
                    .collect();
                TypeInfo::Enumeration { literals }
            }
            crate::ast::type_def::ScalarTypeDefinition::Integer(_) => TypeInfo::Integer,
            crate::ast::type_def::ScalarTypeDefinition::Floating(_) => TypeInfo::Floating,
            crate::ast::type_def::ScalarTypeDefinition::Physical(p) => TypeInfo::Physical {
                primary_unit: ident_to_lower(&p.primary_unit.identifier),
            },
        },
        TypeDefinition::Composite(comp) => match comp {
            crate::ast::type_def::CompositeTypeDefinition::Array(arr) => {
                let (element_type, dims) = match arr {
                    crate::ast::type_def::ArrayTypeDefinition::Unbounded(u) => (
                        subtype_indication_name(&u.element_subtype),
                        u.index_subtypes.len(),
                    ),
                    crate::ast::type_def::ArrayTypeDefinition::Constrained(c) => (
                        subtype_indication_name(&c.element_subtype),
                        c.index_constraint.ranges.len(),
                    ),
                };
                TypeInfo::Array {
                    element_type,
                    dimensions: dims,
                }
            }
            crate::ast::type_def::CompositeTypeDefinition::Record(r) => {
                let fields: Vec<(String, String)> = r
                    .elements
                    .iter()
                    .flat_map(|elem| {
                        let field_type = subtype_indication_name(&elem.subtype.subtype_indication);
                        elem.identifiers
                            .identifiers
                            .iter()
                            .map(|id| (ident_to_lower(id), field_type.clone()))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                TypeInfo::Record { fields }
            }
        },
        TypeDefinition::Access(a) => TypeInfo::Access {
            designated_type: subtype_indication_name(&a.subtype_indication),
        },
        TypeDefinition::File(f) => TypeInfo::FileType {
            type_mark: type_mark_name(&f.type_mark),
        },
        TypeDefinition::Protected(_) => TypeInfo::Incomplete,
    }
}
