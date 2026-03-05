use lambila::project::dependency::DependencyGraph;
use lambila::project::library::DesignUnitKind;
use lambila::project::workspace::Workspace;
use lambila::{VhdlVersion, lex_file, parse};
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "lex" | "parse" | "format" => {
            // Single-file commands: require <file> [version]
            if args.len() < 3 {
                eprintln!("Error: '{}' requires a file argument.", command);
                print_usage(&args[0]);
                process::exit(1);
            }
            let path = &args[2];
            let version = parse_version(args.get(3));
            match command.as_str() {
                "lex" => cmd_lex(path, version),
                "parse" => cmd_parse(path, version),
                "format" => cmd_format(path, version),
                _ => unreachable!(),
            }
        }
        "project" => {
            // Multi-file project command:
            //   project [-r] [--version VER] <path...>
            cmd_project(&args[2..]);
        }
        other => {
            eprintln!(
                "Unknown command '{}'. Use lex, parse, format, or project.",
                other
            );
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {} lex     <vhdl-file> [version]", program);
    eprintln!("  {} parse   <vhdl-file> [version]", program);
    eprintln!("  {} format  <vhdl-file> [version]", program);
    eprintln!("  {} project [-r] [--version VER] <path>...", program);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  lex      Lex a single file and print tokens");
    eprintln!("  parse    Lex and parse a single file, then print the AST");
    eprintln!("  format   Lex and parse a single file, re-export formatted VHDL");
    eprintln!("  project  Load a VHDL project (multiple files or -r for recursive");
    eprintln!("           directory scan) and print the design hierarchy");
    eprintln!();
    eprintln!("Versions: 1987, 1993 (default), 2008");
}

fn parse_version(arg: Option<&String>) -> VhdlVersion {
    match arg.map(|s| s.as_str()) {
        Some("1987") | Some("87") => VhdlVersion::Vhdl1987,
        Some("1993") | Some("93") => VhdlVersion::Vhdl1993,
        Some("2008") | Some("08") => VhdlVersion::Vhdl2008,
        Some(other) => {
            eprintln!("Unknown VHDL version '{}'. Use 1987, 1993, or 2008.", other);
            process::exit(1);
        }
        None => VhdlVersion::Vhdl1993,
    }
}

// ---------------------------------------------------------------------------
// Single-file commands
// ---------------------------------------------------------------------------

fn lex_or_exit(path: &str, version: VhdlVersion) -> lambila::LexResult {
    match lex_file(path, version) {
        Ok(r) => {
            if !r.errors.is_empty() {
                eprintln!("--- {} lexer error(s) ---", r.errors.len());
                for err in &r.errors {
                    eprintln!("  {}", err);
                }
            }
            r
        }
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn cmd_lex(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

    for token in &result.tokens {
        println!(
            "{:>4}:{:<3}  {:20}  {}",
            token.span.line,
            token.span.col,
            format!("{:?}", token.kind),
            token.text
        );
    }

    if !result.errors.is_empty() {
        process::exit(2);
    }
}

fn cmd_parse(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

    match parse(&result.tokens) {
        Ok(design_file) => {
            println!("{:#?}", design_file);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(2);
        }
    }
}

fn cmd_format(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

    match parse(&result.tokens) {
        Ok(design_file) => {
            print!("{}", design_file);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(2);
        }
    }
}

// ---------------------------------------------------------------------------
// Project command
// ---------------------------------------------------------------------------

fn cmd_project(args: &[String]) {
    let mut recursive = false;
    let mut version = VhdlVersion::Vhdl1993;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--recursive" => recursive = true,
            "--version" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --version requires a value.");
                    process::exit(1);
                }
                version = parse_version(Some(&args[i]));
            }
            other => paths.push(other.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("Error: 'project' requires at least one file or directory.");
        process::exit(1);
    }

    // Collect all VHDL files.
    let vhdl_files = collect_vhdl_files(&paths, recursive);

    if vhdl_files.is_empty() {
        eprintln!("No VHDL files found.");
        process::exit(1);
    }

    eprintln!(
        "Loading {} VHDL file(s) as {}...",
        vhdl_files.len(),
        version
    );

    // Build workspace.
    let mut workspace = Workspace::new();
    let mut errors = 0;

    for file_path in &vhdl_files {
        match workspace.load_file(Path::new(file_path), version, None) {
            Ok(loaded) => {
                eprintln!("  ✓ {} ({} unit(s))", file_path, loaded.len());
            }
            Err(e) => {
                eprintln!("  ✗ {}: {}", file_path, e);
                errors += 1;
            }
        }
    }

    if errors > 0 {
        eprintln!("\n{} file(s) failed to load.", errors);
    }

    // Build dependency graph and sort.
    let graph = DependencyGraph::build(&workspace);
    match graph.topological_sort() {
        Ok(order) => {
            eprintln!(
                "\nCompilation order determined ({} design unit(s)).\n",
                order.len()
            );
            print_entity_tree(&workspace);
        }
        Err(e) => {
            eprintln!("\nDependency error: {}", e);
            process::exit(2);
        }
    }
}

/// Collect all `.vhd` / `.vhdl` files from the given list of paths.
/// If `recursive` is true, directories are walked recursively.
fn collect_vhdl_files(paths: &[String], recursive: bool) -> Vec<String> {
    let mut result = Vec::new();
    for p in paths {
        let path = Path::new(p);
        if path.is_file() {
            if is_vhdl_file(path) {
                result.push(p.clone());
            } else {
                eprintln!("Warning: '{}' is not a VHDL file, skipping.", p);
            }
        } else if path.is_dir() {
            if recursive {
                collect_dir_recursive(path, &mut result);
            } else {
                // Non-recursive: just list immediate children.
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let ep = entry.path();
                        if ep.is_file() && is_vhdl_file(&ep) {
                            result.push(ep.to_string_lossy().to_string());
                        }
                    }
                }
            }
        } else {
            eprintln!("Warning: '{}' does not exist, skipping.", p);
        }
    }
    result.sort();
    result
}

fn collect_dir_recursive(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_dir_recursive(&path, out);
            } else if path.is_file() && is_vhdl_file(&path) {
                out.push(path.to_string_lossy().to_string());
            }
        }
    }
}

fn is_vhdl_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("vhd") | Some("vhdl") | Some("VHD") | Some("VHDL")
    )
}

// ---------------------------------------------------------------------------
// Entity hierarchy tree
// ---------------------------------------------------------------------------

use lambila::ast::common::Identifier;
use lambila::ast::component::InstantiatedUnit;
use lambila::ast::concurrent::ConcurrentStatement;
use lambila::ast::design_unit::{LibraryUnit, SecondaryUnit};
use lambila::ast::generate::GenerateStatement;
use lambila::ast::name::{Name, Suffix};
use lambila::project::library::AnalyzedUnit;
use std::collections::HashSet;

/// Information about a single component/entity instantiation found in an
/// architecture body.
#[derive(Debug, Clone)]
struct InstanceInfo {
    /// The label used in the instantiation statement.
    label: String,
    /// The resolved entity/component name (lowercased).
    entity_name: String,
    /// How it was instantiated.
    style: InstantiationStyle,
}

#[derive(Debug, Clone)]
enum InstantiationStyle {
    /// `entity lib.entity_name(arch)`
    DirectEntity { arch: Option<String> },
    /// `component comp_name` or just `comp_name`
    Component,
}

/// Print the workspace as a hierarchical entity tree that resolves
/// sub-instances within architectures.
fn print_entity_tree(workspace: &Workspace) {
    // Build a lookup: entity_name -> list of architecture AnalyzedUnits.
    // We look across all libraries.
    let mut entity_archs: std::collections::HashMap<String, Vec<&AnalyzedUnit>> =
        std::collections::HashMap::new();

    for lib in workspace.libraries() {
        for unit in lib.iter() {
            if let DesignUnitKind::Architecture { entity_name } = &unit.kind {
                entity_archs
                    .entry(entity_name.clone())
                    .or_default()
                    .push(unit);
            }
        }
    }

    // Find top-level entities: entities that are NOT instantiated by any
    // architecture in the workspace.
    let all_instantiated = collect_all_instantiated_names(workspace);

    for lib in workspace.libraries() {
        println!("Library: {}", lib.name);

        let entities: Vec<&AnalyzedUnit> = lib
            .iter()
            .filter(|u| u.kind == DesignUnitKind::Entity)
            .collect();

        let packages: Vec<&AnalyzedUnit> = lib
            .iter()
            .filter(|u| u.kind == DesignUnitKind::Package)
            .collect();

        // Separate top-level entities from sub-entities.
        let top_entities: Vec<&&AnalyzedUnit> = entities
            .iter()
            .filter(|e| !all_instantiated.contains(&e.name))
            .collect();
        let sub_entities: Vec<&&AnalyzedUnit> = entities
            .iter()
            .filter(|e| all_instantiated.contains(&e.name))
            .collect();

        // Print top-level entities with hierarchy.
        if !top_entities.is_empty() {
            println!("  Top-level entities:");
            for (i, entity) in top_entities.iter().enumerate() {
                let is_last =
                    i == top_entities.len() - 1 && packages.is_empty() && sub_entities.is_empty();
                let connector = if is_last { "└" } else { "├" };
                println!("  {}── Entity: {}", connector, entity.name);
                let branch = if is_last { " " } else { "│" };
                print_entity_hierarchy(
                    &entity_archs,
                    &entity.name,
                    &format!("  {}   ", branch),
                    &mut HashSet::new(),
                );
            }
        }

        // Print sub-entities (instantiated by others) — just list them.
        if !sub_entities.is_empty() {
            println!("  Sub-entities:");
            for (i, entity) in sub_entities.iter().enumerate() {
                let is_last = i == sub_entities.len() - 1 && packages.is_empty();
                let connector = if is_last { "└" } else { "├" };
                println!("  {}── Entity: {}", connector, entity.name);
            }
        }

        // Print packages.
        if !packages.is_empty() {
            println!("  Packages:");
            for (i, pkg) in packages.iter().enumerate() {
                let has_body = lib
                    .iter()
                    .any(|u| u.kind == DesignUnitKind::PackageBody && u.name == pkg.name);
                let is_last = i == packages.len() - 1;
                let connector = if is_last { "└" } else { "├" };
                if has_body {
                    println!("  {}── Package: {} (+ body)", connector, pkg.name);
                } else {
                    println!("  {}── Package: {}", connector, pkg.name);
                }
            }
        }

        println!();
    }
}

/// Print the hierarchy under a given entity, recursing into architectures.
fn print_entity_hierarchy(
    entity_archs: &std::collections::HashMap<String, Vec<&AnalyzedUnit>>,
    entity_name: &str,
    prefix: &str,
    visited: &mut HashSet<String>,
) {
    // Prevent infinite recursion.
    if visited.contains(entity_name) {
        println!("{}└── (recursive: {})", prefix, entity_name);
        return;
    }
    visited.insert(entity_name.to_string());

    if let Some(archs) = entity_archs.get(entity_name) {
        for (arch_idx, arch_unit) in archs.iter().enumerate() {
            let is_last_arch = arch_idx == archs.len() - 1;
            let arch_connector = if is_last_arch { "└" } else { "├" };
            println!(
                "{}{}── Architecture: {}",
                prefix, arch_connector, arch_unit.name
            );

            // Extract sub-instances from this architecture's AST.
            let instances = extract_instances_from_unit(arch_unit);

            let child_prefix = if is_last_arch {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            for (inst_idx, inst) in instances.iter().enumerate() {
                let is_last_inst = inst_idx == instances.len() - 1;
                let inst_connector = if is_last_inst { "└" } else { "├" };

                let style_label = match &inst.style {
                    InstantiationStyle::DirectEntity { arch } => {
                        if let Some(a) = arch {
                            format!("entity {}({})", inst.entity_name, a)
                        } else {
                            format!("entity {}", inst.entity_name)
                        }
                    }
                    InstantiationStyle::Component => {
                        format!("component {}", inst.entity_name)
                    }
                };

                println!(
                    "{}{}── {}: {} [{}]",
                    child_prefix, inst_connector, inst.label, inst.entity_name, style_label
                );

                // Recurse into the instantiated entity.
                let recurse_prefix = if is_last_inst {
                    format!("{}    ", child_prefix)
                } else {
                    format!("{}│   ", child_prefix)
                };

                print_entity_hierarchy(entity_archs, &inst.entity_name, &recurse_prefix, visited);
            }
        }
    }

    visited.remove(entity_name);
}

/// Collect the names of all entities/components that are instantiated
/// anywhere in the workspace.
fn collect_all_instantiated_names(workspace: &Workspace) -> HashSet<String> {
    let mut names = HashSet::new();
    for lib in workspace.libraries() {
        for unit in lib.iter() {
            let instances = extract_instances_from_unit(unit);
            for inst in instances {
                names.insert(inst.entity_name);
            }
        }
    }
    names
}

/// Extract all component/entity instantiations from a design unit.
fn extract_instances_from_unit(unit: &AnalyzedUnit) -> Vec<InstanceInfo> {
    match &unit.design_unit.library_unit {
        LibraryUnit::Secondary(SecondaryUnit::Architecture(arch)) => {
            let mut instances = Vec::new();
            for stmt in &arch.statement_part.statements {
                collect_instances_from_statement(stmt, &mut instances);
            }
            instances
        }
        _ => Vec::new(),
    }
}

/// Recursively collect instantiations from a concurrent statement
/// (handles generate statements which nest concurrent statements).
fn collect_instances_from_statement(stmt: &ConcurrentStatement, out: &mut Vec<InstanceInfo>) {
    match stmt {
        ConcurrentStatement::ComponentInstantiation(inst) => {
            let label = ident_to_lower(&inst.label.identifier);
            let (entity_name, style) = resolve_instantiated_unit(&inst.unit);
            out.push(InstanceInfo {
                label,
                entity_name,
                style,
            });
        }
        ConcurrentStatement::Generate(generate) => {
            collect_instances_from_generate(generate, out);
        }
        ConcurrentStatement::Block(block) => {
            for s in &block.statement_part.statements {
                collect_instances_from_statement(s, out);
            }
        }
        // Process, signal assignment, procedure call, assertion — no sub-instances.
        _ => {}
    }
}

/// Collect instances from generate statement bodies.
fn collect_instances_from_generate(generate_stmt: &GenerateStatement, out: &mut Vec<InstanceInfo>) {
    match generate_stmt {
        GenerateStatement::For(fg) => {
            for s in &fg.body.statements {
                collect_instances_from_statement(s, out);
            }
        }
        GenerateStatement::If(ig) => {
            for s in &ig.if_branch.body.statements {
                collect_instances_from_statement(s, out);
            }
            for branch in &ig.elsif_branches {
                for s in &branch.body.statements {
                    collect_instances_from_statement(s, out);
                }
            }
            if let Some(ref else_branch) = ig.else_branch {
                for s in &else_branch.body.statements {
                    collect_instances_from_statement(s, out);
                }
            }
        }
        GenerateStatement::Case(cg) => {
            for alt in &cg.alternatives {
                for s in &alt.body.statements {
                    collect_instances_from_statement(s, out);
                }
            }
        }
        GenerateStatement::Legacy(lg) => {
            for s in &lg.statements {
                collect_instances_from_statement(s, out);
            }
        }
    }
}

/// Extract the entity/component name from an InstantiatedUnit.
fn resolve_instantiated_unit(unit: &InstantiatedUnit) -> (String, InstantiationStyle) {
    match unit {
        InstantiatedUnit::Entity { name, architecture } => {
            let entity_name = extract_entity_name_from_name(name);
            let arch = architecture.as_ref().map(ident_to_lower);
            (entity_name, InstantiationStyle::DirectEntity { arch })
        }
        InstantiatedUnit::Component { name, .. } => {
            let component_name = name_to_lower(name);
            (component_name, InstantiationStyle::Component)
        }
        InstantiatedUnit::Configuration(name) => {
            let config_name = name_to_lower(name);
            (config_name, InstantiationStyle::Component)
        }
    }
}

/// For `entity work.foo` (a selected name), extract `foo`.
/// For a simple name, just return it lowercased.
fn extract_entity_name_from_name(name: &Name) -> String {
    match name {
        Name::Selected(sel) => suffix_to_lower(&sel.suffix),
        Name::Simple(s) => ident_to_lower(&s.identifier),
        _ => name_to_lower(name),
    }
}

/// Convert a Name to its lowercased string (best-effort for simple/selected).
fn name_to_lower(name: &Name) -> String {
    match name {
        Name::Simple(s) => ident_to_lower(&s.identifier),
        Name::Selected(sel) => {
            // For selected names like `lib.entity`, use the suffix (entity name).
            suffix_to_lower(&sel.suffix)
        }
        _ => "<unknown>".to_string(),
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

fn ident_to_lower(id: &Identifier) -> String {
    match id {
        Identifier::Basic(s) => s.to_lowercase(),
        Identifier::Extended(s) => s.clone(),
    }
}
