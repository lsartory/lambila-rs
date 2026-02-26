use crate::parser::ast::*;
use std::io::{self, Write};

/// The VHDL exporter. Converts a parsed abstract syntax tree into formatted VHDL source code.
///
/// The exporter takes an AST (`DesignFile` or individual syntax node) and writes out
/// semantically identical VHDL tokens, enforcing standardized spacing and indentation rules.
/// It wraps any standard [`std::io::Write`] output stream.
pub struct Exporter<W: Write> {
    out: W,
    indent_level: usize,
}

impl<W: Write> Exporter<W> {
    /// Create a new exporter wrapping the provided `Write` output stream.
    pub fn new(out: W) -> Self {
        Self {
            out,
            indent_level: 0,
        }
    }

    fn write_indent(&mut self) -> io::Result<()> {
        for _ in 0..self.indent_level {
            write!(self.out, "    ")?;
        }
        Ok(())
    }

    /// Run the given closure with increased indentation level.
    fn with_indent<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self) -> io::Result<()>,
    {
        self.indent_level = self.indent_level.saturating_add(1);
        let res = f(self);
        self.indent_level = self.indent_level.saturating_sub(1);
        res
    }

    /// Helper to export a comma-separated list of items.
    fn export_list<I, T, F>(&mut self, items: I, mut f: F) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        F: FnMut(&mut Self, T) -> io::Result<()>,
    {
        let mut first = true;
        for item in items {
            if !first {
                write!(self.out, ", ")?;
            }
            first = false;
            f(self, item)?;
        }
        Ok(())
    }

    /// Export an entire `DesignFile` (the root of the AST) formatting the contents to the output stream.
    ///
    /// This method inserts 50-dash separator lines between multiple design units.
    pub fn export_design_file(&mut self, design_file: &DesignFile) -> io::Result<()> {
        for (i, unit) in design_file.units.iter().enumerate() {
            if i > 0 {
                writeln!(self.out)?;
                writeln!(
                    self.out,
                    "--------------------------------------------------"
                )?;
                writeln!(self.out)?;
            }
            self.export_design_unit(unit)?;
        }
        Ok(())
    }

    pub fn export_design_unit(&mut self, unit: &DesignUnit) -> io::Result<()> {
        if !unit.context.items.is_empty() {
            self.export_context_clause(&unit.context)?;
        }
        self.export_library_unit(&unit.unit)?;
        Ok(())
    }

    fn export_context_clause(&mut self, context_clause: &ContextClause) -> io::Result<()> {
        for item in &context_clause.items {
            match item {
                ContextItem::Library(lib) => self.export_library_clause(lib)?,
                ContextItem::Use(use_cl) => self.export_use_clause(use_cl)?,
                ContextItem::ContextReference(ctx_ref) => self.export_context_reference(ctx_ref)?,
            }
        }
        if !context_clause.items.is_empty() {
            writeln!(self.out)?;
            writeln!(
                self.out,
                "--------------------------------------------------"
            )?;
            writeln!(self.out)?;
        }
        Ok(())
    }

    fn export_library_clause(&mut self, lib: &LibraryClause) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "library ")?;
        self.export_list(&lib.names, |e, name| write!(e.out, "{}", &name.text))?;
        writeln!(self.out, ";")
    }

    fn export_use_clause(&mut self, use_cl: &UseClause) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "use ")?;
        self.export_list(&use_cl.names, |e, name| e.export_name(name))?;
        writeln!(self.out, ";")
    }

    fn export_context_reference(&mut self, ctx_ref: &ContextReference) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "context ")?;
        self.export_list(&ctx_ref.names, |e, name| e.export_name(name))?;
        writeln!(self.out, ";")
    }

    fn export_library_unit(&mut self, unit: &LibraryUnit) -> io::Result<()> {
        match unit {
            LibraryUnit::Entity(entity) => self.export_entity_declaration(entity)?,
            LibraryUnit::Architecture(arch) => self.export_architecture_body(arch)?,
            LibraryUnit::Package(pkg) => self.export_package_declaration(pkg)?,
            LibraryUnit::PackageBody(pkg_body) => self.export_package_body(pkg_body)?,
            LibraryUnit::Configuration(cfg) => self.export_configuration_declaration(cfg)?,
            LibraryUnit::ContextDeclaration(ctx) => self.export_context_declaration(ctx)?,
        }
        Ok(())
    }

    fn export_entity_declaration(&mut self, entity: &EntityDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "entity ")?;
        write!(self.out, "{}", &entity.name.text)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            if let Some(generics) = &entity.generics {
                e.write_indent()?;
                writeln!(e.out, "generic (")?;
                e.with_indent(|e2| e2.export_interface_list(generics))?;
                e.write_indent()?;
                writeln!(e.out, ");")?;
            }
            if let Some(ports) = &entity.ports {
                e.write_indent()?;
                writeln!(e.out, "port (")?;
                e.with_indent(|e2| e2.export_interface_list(ports))?;
                e.write_indent()?;
                writeln!(e.out, ");")?;
            }
            for decl in &entity.decls {
                e.export_declaration(decl)?;
            }
            Ok(())
        })?;
        if !entity.stmts.is_empty() {
            self.write_indent()?;
            writeln!(self.out, "begin")?;
            self.with_indent(|e| {
                for stmt in &entity.stmts {
                    e.export_concurrent_statement(stmt)?;
                }
                Ok(())
            })?;
        }

        self.write_indent()?;
        write!(self.out, "end ")?;
        if let Some(end_name) = &entity.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &entity.name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_architecture_body(&mut self, arch: &ArchitectureBody) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "architecture ")?;
        write!(self.out, "{}", &arch.name.text)?;
        write!(self.out, " of ")?;
        self.export_name(&arch.entity_name)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            for decl in &arch.decls {
                e.export_declaration(decl)?;
            }
            Ok(())
        })?;

        self.write_indent()?;
        writeln!(self.out, "begin")?;

        self.with_indent(|e| {
            for stmt in &arch.stmts {
                e.export_concurrent_statement(stmt)?;
            }
            Ok(())
        })?;

        self.write_indent()?;
        write!(self.out, "end ")?;
        if let Some(end_name) = &arch.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &arch.name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_package_declaration(&mut self, pkg: &PackageDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "package ")?;
        write!(self.out, "{}", &pkg.name.text)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            for decl in &pkg.decls {
                e.export_declaration(decl)?;
            }
            Ok(())
        })?;

        self.write_indent()?;
        write!(self.out, "end package ")?;
        if let Some(end_name) = &pkg.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &pkg.name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_package_body(&mut self, pkg_body: &PackageBody) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "package body ")?;
        write!(self.out, "{}", &pkg_body.name.text)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            for decl in &pkg_body.decls {
                e.export_declaration(decl)?;
            }
            Ok(())
        })?;

        self.write_indent()?;
        write!(self.out, "end package body ")?;
        if let Some(end_name) = &pkg_body.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &pkg_body.name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_configuration_declaration(
        &mut self,
        cfg: &ConfigurationDeclaration,
    ) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "configuration ")?;
        write!(self.out, "{}", &cfg.name.text)?;
        write!(self.out, " of ")?;
        self.export_name(&cfg.entity_name)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            for decl in &cfg.decls {
                e.export_declaration(decl)?;
            }
            Ok(())
        })?;

        self.write_indent()?;
        write!(self.out, "end configuration ")?;
        if let Some(end_name) = &cfg.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &cfg.name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_context_declaration(&mut self, ctx: &ContextDeclarationUnit) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "context ")?;
        write!(self.out, "{}", &ctx.name.text)?;
        writeln!(self.out, " is")?;

        self.with_indent(|e| {
            for item in &ctx.items {
                match item {
                    ContextItem::Library(lib) => e.export_library_clause(lib)?,
                    ContextItem::Use(use_cl) => e.export_use_clause(use_cl)?,
                    ContextItem::ContextReference(ctx_ref) => {
                        e.export_context_reference(ctx_ref)?
                    }
                }
            }
            Ok(())
        })?;

        self.write_indent()?;
        write!(self.out, "end context ")?;
        if let Some(end_name) = &ctx.end_name {
            write!(self.out, "{}", &end_name.text)?;
        } else {
            write!(self.out, "{}", &ctx.name.text)?;
        }
        writeln!(self.out, ";")
    }

    // A lot of names are represented differently, Name is an enum
    fn export_name(&mut self, name: &Name) -> io::Result<()> {
        match name {
            Name::Simple(id) => write!(self.out, "{}", &id.text),
            Name::Selected(prefix, suffix) => {
                self.export_name(prefix)?;
                write!(self.out, ".")?;
                write!(self.out, "{}", &suffix.text)
            }
            Name::Indexed(prefix, args, _) => {
                self.export_name(prefix)?;
                write!(self.out, "(")?;
                self.export_list(args, |e, expr| e.export_expression(expr))?;
                write!(self.out, ")")
            }
            Name::Slice(prefix, range, _) => {
                self.export_name(prefix)?;
                write!(self.out, "(")?;
                self.export_discrete_range(range)?;
                write!(self.out, ")")
            }
            Name::Attribute(prefix, attr_designator, expression, _) => {
                self.export_name(prefix)?;
                write!(self.out, "'")?;
                write!(self.out, "{}", &attr_designator.text)?;
                if let Some(expr) = expression {
                    write!(self.out, "(")?;
                    self.export_expression(expr)?;
                    write!(self.out, ")")?;
                }
                Ok(())
            }
            Name::Operator(op, _) => write!(self.out, "{}", op),
        }
    }

    fn export_declaration(&mut self, decl: &Declaration) -> io::Result<()> {
        match decl {
            Declaration::Type(t) => self.export_type_decl(t),
            Declaration::Subtype(s) => self.export_subtype_decl(s),
            Declaration::Constant(c) => self.export_object_decl("constant", c),
            Declaration::Signal(c) => self.export_object_decl("signal", c),
            Declaration::Variable(c) => self.export_object_decl("variable", c),
            Declaration::File(c) => self.export_object_decl("file", c),
            Declaration::Component(c) => self.export_component_decl(c),
            Declaration::Alias(a) => self.export_alias_decl(a),
            Declaration::Attribute(a) => self.export_attribute_decl(a),
            Declaration::AttributeSpec(a) => self.export_attribute_spec(a),
            Declaration::Use(u) => self.export_use_clause(u),
            Declaration::SubprogramDecl(s) => self.export_subprogram_decl(s),
            Declaration::SubprogramBody(b) => self.export_subprogram_body(b),
            Declaration::ConfigSpec(_) | Declaration::Disconnection(_) => {
                self.write_indent()?;
                writeln!(self.out, "-- unsupported declaration")
            }
        }
    }

    fn export_type_decl(&mut self, t: &TypeDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "type ")?;
        write!(self.out, "{}", &t.name.text)?;
        if let Some(def) = &t.def {
            write!(self.out, " is ")?;
            self.export_type_definition(def)?;
        }
        writeln!(self.out, ";")
    }

    fn export_subtype_decl(&mut self, s: &SubtypeDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "subtype ")?;
        write!(self.out, "{}", &s.name.text)?;
        write!(self.out, " is ")?;
        self.export_subtype_indication(&s.indication)?;
        writeln!(self.out, ";")
    }

    fn export_object_decl(&mut self, kind: &str, c: &ObjectDeclaration) -> io::Result<()> {
        self.write_indent()?;
        if c.shared {
            write!(self.out, "shared ")?;
        }
        write!(self.out, "{}", kind)?;
        write!(self.out, " ")?;
        self.export_list(&c.names, |e, name| write!(e.out, "{}", &name.text))?;
        write!(self.out, ": ")?;
        self.export_subtype_indication(&c.subtype)?;
        if let Some(expr) = &c.default {
            write!(self.out, " := ")?;
            self.export_expression(expr)?;
        }
        writeln!(self.out, ";")
    }

    fn export_component_decl(&mut self, c: &ComponentDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "component ")?;
        writeln!(self.out, "{}", &c.name.text)?;
        self.with_indent(|e| {
            if let Some(generics) = &c.generics {
                e.write_indent()?;
                writeln!(e.out, "generic (")?;
                e.with_indent(|e2| e2.export_interface_list(generics))?;
                e.write_indent()?;
                writeln!(e.out, ");")?;
            }
            if let Some(ports) = &c.ports {
                e.write_indent()?;
                writeln!(e.out, "port (")?;
                e.with_indent(|e2| e2.export_interface_list(ports))?;
                e.write_indent()?;
                writeln!(e.out, ");")?;
            }
            Ok(())
        })?;
        self.write_indent()?;
        write!(self.out, "end component")?;
        if let Some(end_name) = &c.end_name {
            write!(self.out, " ")?;
            write!(self.out, "{}", &end_name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_alias_decl(&mut self, a: &AliasDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "alias ")?;
        write!(self.out, "{}", &a.designator.text)?;
        if let Some(sub) = &a.subtype {
            write!(self.out, ": ")?;
            self.export_subtype_indication(sub)?;
        }
        write!(self.out, " is ")?;
        self.export_name(&a.name)?;
        writeln!(self.out, ";")
    }

    fn export_attribute_decl(&mut self, a: &AttributeDeclaration) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "attribute ")?;
        write!(self.out, "{}", &a.name.text)?;
        write!(self.out, ": ")?;
        self.export_name(&a.type_mark)?;
        writeln!(self.out, ";")
    }

    fn export_attribute_spec(&mut self, a: &AttributeSpecification) -> io::Result<()> {
        self.write_indent()?;
        write!(self.out, "attribute ")?;
        write!(self.out, "{}", &a.designator.text)?;
        write!(self.out, " of ")?;
        match &a.entity_spec.names {
            EntityNameList::Names(names) => {
                self.export_list(names, |e, name| write!(e.out, "{}", &name.text))?;
            }
            EntityNameList::Others => write!(self.out, "others")?,
            EntityNameList::All => write!(self.out, "all")?,
        }
        write!(self.out, ": ")?;
        write!(self.out, "{}", &a.entity_spec.entity_class.text)?;
        write!(self.out, " is ")?;
        self.export_expression(&a.value)?;
        writeln!(self.out, ";")
    }

    fn export_subprogram_decl(&mut self, s: &SubprogramSpec) -> io::Result<()> {
        self.write_indent()?;
        self.export_subprogram_spec(s)?;
        writeln!(self.out, ";")
    }

    fn export_subprogram_body(&mut self, b: &SubprogramBody) -> io::Result<()> {
        self.write_indent()?;
        self.export_subprogram_spec(&b.spec)?;
        writeln!(self.out, " is")?;
        self.with_indent(|e| {
            for d in &b.decls {
                e.export_declaration(d)?;
            }
            Ok(())
        })?;
        self.write_indent()?;
        writeln!(self.out, "begin")?;
        self.with_indent(|e| {
            for stmt in &b.stmts {
                e.export_sequential_statement(stmt)?;
            }
            Ok(())
        })?;
        self.write_indent()?;
        write!(self.out, "end")?;
        if let Some(end_name) = &b.end_name {
            write!(self.out, " ")?;
            write!(self.out, "{}", &end_name.text)?;
        }
        writeln!(self.out, ";")
    }

    fn export_sequential_statement(&mut self, stmt: &SequentialStatement) -> io::Result<()> {
        match stmt {
            SequentialStatement::If(i) => {
                self.write_indent()?;
                write!(self.out, "if ")?;
                self.export_expression(&i.condition)?;
                writeln!(self.out, " then")?;
                self.with_indent(|e| {
                    for s in &i.then_stmts {
                        e.export_sequential_statement(s)?;
                    }
                    Ok(())
                })?;

                for branch in &i.elsif_branches {
                    self.write_indent()?;
                    write!(self.out, "elsif ")?;
                    self.export_expression(&branch.condition)?;
                    writeln!(self.out, " then")?;
                    self.with_indent(|e| {
                        for s in &branch.stmts {
                            e.export_sequential_statement(s)?;
                        }
                        Ok(())
                    })?;
                }

                if let Some(else_stmts) = &i.else_stmts {
                    self.write_indent()?;
                    writeln!(self.out, "else")?;
                    self.with_indent(|e| {
                        for s in else_stmts {
                            e.export_sequential_statement(s)?;
                        }
                        Ok(())
                    })?;
                }

                self.write_indent()?;
                write!(self.out, "end if")?;
                if let Some(end_label) = &i.end_label {
                    write!(self.out, " ")?;
                    write!(self.out, "{}", &end_label.text)?;
                }
                writeln!(self.out, ";")?;
            }
            SequentialStatement::SignalAssignment(s) => {
                self.write_indent()?;
                self.export_name(&s.target)?;
                write!(self.out, " <= ")?;
                self.export_list(&s.waveforms, |e, wave| {
                    e.export_expression(&wave.value)?;
                    if let Some(after) = &wave.after {
                        write!(e.out, " after ")?;
                        e.export_expression(after)?;
                    }
                    Ok(())
                })?;
                writeln!(self.out, ";")?;
            }
            SequentialStatement::VariableAssignment(v) => {
                self.write_indent()?;
                self.export_name(&v.target)?;
                write!(self.out, " := ")?;
                self.export_expression(&v.value)?;
                writeln!(self.out, ";")?;
            }
            SequentialStatement::Loop(l) => {
                self.write_indent()?;
                if let Some(label) = &l.label {
                    write!(self.out, "{}", &label.text)?;
                    write!(self.out, ": ")?;
                }
                if let Some(scheme) = &l.scheme {
                    match scheme {
                        IterationScheme::While(cond) => {
                            write!(self.out, "while ")?;
                            self.export_expression(cond)?;
                            write!(self.out, " ")?;
                        }
                        IterationScheme::For { param, range } => {
                            write!(self.out, "for ")?;
                            write!(self.out, "{}", &param.text)?;
                            write!(self.out, " in ")?;
                            self.export_discrete_range(range)?;
                            write!(self.out, " ")?;
                        }
                    }
                }
                writeln!(self.out, "loop")?;
                self.with_indent(|e| {
                    for s in &l.stmts {
                        e.export_sequential_statement(s)?;
                    }
                    Ok(())
                })?;
                self.write_indent()?;
                write!(self.out, "end loop")?;
                if let Some(end_label) = &l.end_label {
                    write!(self.out, " ")?;
                    write!(self.out, "{}", &end_label.text)?;
                }
                writeln!(self.out, ";")?;
            }
            SequentialStatement::Return(r) => {
                self.write_indent()?;
                if let Some(label) = &r.label {
                    write!(self.out, "{}", &label.text)?;
                    write!(self.out, ": ")?;
                }
                write!(self.out, "return")?;
                if let Some(expr) = &r.expression {
                    write!(self.out, " ")?;
                    self.export_expression(expr)?;
                }
                writeln!(self.out, ";")?;
            }
            SequentialStatement::Case(_)
            | SequentialStatement::Next(_)
            | SequentialStatement::Exit(_)
            | SequentialStatement::Null(_)
            | SequentialStatement::Wait(_)
            | SequentialStatement::Assert(_)
            | SequentialStatement::Report(_)
            | SequentialStatement::ProcedureCall(_) => {
                self.write_indent()?;
                writeln!(self.out, "-- unimplemented sequential statement")?;
            }
        }
        Ok(())
    }

    fn export_concurrent_statement(&mut self, stmt: &ConcurrentStatement) -> io::Result<()> {
        match stmt {
            ConcurrentStatement::Process(p) => {
                writeln!(self.out)?;
                self.write_indent()?;
                if let Some(label) = &p.label {
                    write!(self.out, "{}", &label.text)?;
                    write!(self.out, ": ")?;
                }
                if p.postponed {
                    write!(self.out, "postponed ")?;
                }
                write!(self.out, "process ")?;
                if let Some(sens) = &p.sensitivity_list {
                    write!(self.out, "(")?;
                    match sens {
                        SensitivityList::All => write!(self.out, "all")?,
                        SensitivityList::Names(names) => {
                            self.export_list(names, |e, name| e.export_name(name))?;
                        }
                    }
                    write!(self.out, ")")?;
                }
                writeln!(self.out)?;
                self.with_indent(|e| {
                    for d in &p.decls {
                        e.export_declaration(d)?;
                    }
                    Ok(())
                })?;
                self.write_indent()?;
                writeln!(self.out, "begin")?;
                self.with_indent(|e| {
                    for s in &p.stmts {
                        e.export_sequential_statement(s)?;
                    }
                    Ok(())
                })?;
                self.write_indent()?;
                write!(self.out, "end ")?;
                if p.postponed {
                    write!(self.out, "postponed ")?;
                }
                write!(self.out, "process")?;
                if let Some(end_label) = &p.end_label {
                    write!(self.out, " ")?;
                    write!(self.out, "{}", &end_label.text)?;
                }
                writeln!(self.out, ";")?;
            }
            ConcurrentStatement::SignalAssignment(s) => {
                self.write_indent()?;
                if let Some(label) = &s.label {
                    write!(self.out, "{}", &label.text)?;
                    write!(self.out, ": ")?;
                }
                if s.postponed {
                    write!(self.out, "postponed ")?;
                }
                self.export_name(&s.target)?;
                write!(self.out, " <= ")?;
                self.export_list(&s.waveforms, |e, wave| {
                    e.export_expression(&wave.value)?;
                    if let Some(after) = &wave.after {
                        write!(e.out, " after ")?;
                        e.export_expression(after)?;
                    }
                    Ok(())
                })?;
                writeln!(self.out, ";")?;
            }
            ConcurrentStatement::ComponentInstantiation(c) => {
                self.write_indent()?;
                write!(self.out, "{}", &c.label.text)?;
                write!(self.out, ": ")?;
                match &c.unit {
                    InstantiatedUnit::Component(name) => self.export_name(name)?,
                    InstantiatedUnit::Entity(name, opt_arch) => {
                        write!(self.out, "entity ")?;
                        self.export_name(name)?;
                        if let Some(arch) = opt_arch {
                            write!(self.out, "(")?;
                            write!(self.out, "{}", &arch.text)?;
                            write!(self.out, ")")?;
                        }
                    }
                    InstantiatedUnit::Configuration(name) => {
                        write!(self.out, "configuration ")?;
                        self.export_name(name)?;
                    }
                }
                writeln!(self.out)?;
                self.with_indent(|e| {
                    if let Some(gmap) = &c.generic_map {
                        e.write_indent()?;
                        writeln!(e.out, "generic map (")?;
                        e.with_indent(|e2| e2.export_association_list(gmap))?;
                        e.write_indent()?;
                        writeln!(e.out, ")")?;
                    }
                    if let Some(pmap) = &c.port_map {
                        e.write_indent()?;
                        writeln!(e.out, "port map (")?;
                        e.with_indent(|e2| e2.export_association_list(pmap))?;
                        e.write_indent()?;
                        writeln!(e.out, ")")?;
                    }
                    Ok(())
                })?;
                writeln!(self.out, ";")?;
            }

            ConcurrentStatement::Generate(_)
            | ConcurrentStatement::ProcedureCall(_)
            | ConcurrentStatement::Assert(_)
            | ConcurrentStatement::Block(_) => {
                self.write_indent()?;
                writeln!(self.out, "-- unimplemented concurrent statement")?;
            }
        }
        Ok(())
    }

    fn export_association_element(&mut self, param: &AssociationElement) -> io::Result<()> {
        if let Some(formal) = &param.formal {
            self.export_name(formal)?;
            write!(self.out, " => ")?;
        }
        self.export_expression(&param.actual)?;
        Ok(())
    }

    fn export_association_list(&mut self, list: &AssociationList) -> io::Result<()> {
        self.export_list(&list.elements, |e, elem| e.export_association_element(elem))?;
        Ok(())
    }

    fn export_expression(&mut self, expr: &Expression) -> io::Result<()> {
        match expr {
            Expression::Literal(l) => match l {
                LiteralValue::Integer(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::Real(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::Based(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::Character(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::String(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::BitString(s, _) => write!(self.out, "{}", s)?,
                LiteralValue::Null(_) => write!(self.out, "null")?,
            },
            Expression::Name(n) => self.export_name(n)?,
            Expression::Binary { lhs, op, rhs, .. } => {
                self.export_expression(lhs)?;
                write!(self.out, " {} ", op)?;
                self.export_expression(rhs)?;
            }
            Expression::Unary { op, operand, .. } => {
                write!(self.out, "{}", op)?;
                self.export_expression(operand)?;
            }
            Expression::Aggregate(elems, _) => {
                write!(self.out, "(")?;
                self.export_list(elems, |e, elem| {
                    if let Some(choices) = &elem.choices {
                        for (j, choice) in choices.iter().enumerate() {
                            if j > 0 {
                                write!(e.out, " | ")?;
                            }
                            match choice {
                                Choice::Expression(exp) => e.export_expression(exp)?,
                                Choice::DiscreteRange(r) => e.export_discrete_range(r)?,
                                Choice::Others => write!(e.out, "others")?,
                            }
                        }
                        write!(e.out, " => ")?;
                    }
                    e.export_expression(&elem.expr)
                })?;
                write!(self.out, ")")?;
            }
            Expression::Allocator(expr, _) => {
                write!(self.out, "new ")?;
                self.export_expression(expr)?;
            }
            Expression::Qualified {
                type_mark, expr, ..
            } => {
                self.export_name(type_mark)?;
                write!(self.out, "'(")?;
                self.export_expression(expr)?;
                write!(self.out, ")")?;
            }
            Expression::TypeConversion {
                type_mark, expr, ..
            } => {
                self.export_name(type_mark)?;
                write!(self.out, "(")?;
                self.export_expression(expr)?;
                write!(self.out, ")")?;
            }
            Expression::FunctionCall { name, args, .. } => {
                self.export_name(name)?;
                write!(self.out, "(")?;
                self.export_association_list(args)?;
                write!(self.out, ")")?;
            }
            Expression::Open(_) => write!(self.out, "open")?,
        }
        Ok(())
    }

    fn export_discrete_range(&mut self, range: &DiscreteRange) -> io::Result<()> {
        match range {
            DiscreteRange::Subtype(sub) => self.export_subtype_indication(sub)?,
            DiscreteRange::Range(r) => self.export_range(r)?,
        }
        Ok(())
    }

    fn export_range(&mut self, range: &Range) -> io::Result<()> {
        match range {
            Range::Attribute(name) => self.export_name(name)?,
            Range::Expr {
                left,
                direction,
                right,
            } => {
                self.export_expression(left)?;
                match direction {
                    Direction::To => write!(self.out, " to ")?,
                    Direction::Downto => write!(self.out, " downto ")?,
                }
                self.export_expression(right)?;
            }
        }
        Ok(())
    }

    fn export_type_definition(&mut self, def: &TypeDefinition) -> io::Result<()> {
        match def {
            TypeDefinition::Enumeration(lits) => {
                write!(self.out, "(")?;
                self.export_list(lits, |e, lit| match lit {
                    EnumerationLiteral::Identifier(id) => write!(e.out, "{}", &id.text),
                    EnumerationLiteral::Character(ch, _) => {
                        write!(e.out, "'")?;
                        write!(e.out, "{}", ch)?;
                        write!(e.out, "'")
                    }
                })?;
                write!(self.out, ")")?;
            }
            TypeDefinition::Integer(c) | TypeDefinition::Floating(c) => {
                write!(self.out, "range ")?;
                self.export_range(&c.range)?;
            }
            TypeDefinition::Physical {
                constraint,
                base_unit,
                secondary_units,
            } => {
                write!(self.out, "range ")?;
                self.export_range(&constraint.range)?;
                writeln!(self.out, " units")?;
                self.with_indent(|e| {
                    e.write_indent()?;
                    write!(e.out, "{}", &base_unit.text)?;
                    writeln!(e.out, ";")?;
                    for (id, expr) in secondary_units {
                        e.write_indent()?;
                        write!(e.out, "{}", &id.text)?;
                        write!(e.out, " = ")?;
                        e.export_expression(expr)?;
                        writeln!(e.out, ";")?;
                    }
                    Ok(())
                })?;
                self.write_indent()?;
                write!(self.out, "end units")?;
            }
            TypeDefinition::Array(a) => {
                write!(self.out, "array (")?;
                match a {
                    ArrayTypeDefinition::Unconstrained { index_subtypes, .. } => {
                        self.export_list(index_subtypes, |e, typ| {
                            e.export_name(typ)?;
                            write!(e.out, " range <>")
                        })?;
                    }
                    ArrayTypeDefinition::Constrained {
                        index_constraint, ..
                    } => {
                        self.export_list(index_constraint, |e, range| {
                            e.export_discrete_range(range)
                        })?;
                    }
                }
                write!(self.out, ") of ")?;
                let subtype = match a {
                    ArrayTypeDefinition::Unconstrained {
                        element_subtype, ..
                    } => element_subtype,
                    ArrayTypeDefinition::Constrained {
                        element_subtype, ..
                    } => element_subtype,
                };
                self.export_subtype_indication(subtype)?;
            }
            TypeDefinition::Record(elems) => {
                writeln!(self.out, "record")?;
                self.with_indent(|e| {
                    for elem in elems {
                        e.write_indent()?;
                        e.export_list(&elem.names, |e2, name| write!(e2.out, "{}", &name.text))?;
                        write!(e.out, ": ")?;
                        e.export_subtype_indication(&elem.subtype)?;
                        writeln!(e.out, ";")?;
                    }
                    Ok(())
                })?;
                self.write_indent()?;
                write!(self.out, "end record")?;
            }
            TypeDefinition::Access(sub) => {
                write!(self.out, "access ")?;
                self.export_subtype_indication(sub)?;
            }
            TypeDefinition::File(name) => {
                write!(self.out, "file of ")?;
                self.export_name(name)?;
            }
        }
        Ok(())
    }

    fn export_subtype_indication(&mut self, sub: &SubtypeIndication) -> io::Result<()> {
        self.export_name(&sub.type_mark)?;
        if let Some(constraint) = &sub.constraint {
            match constraint {
                Constraint::Range(c) => {
                    write!(self.out, " range ")?;
                    self.export_range(&c.range)?;
                }
                Constraint::Index(ranges) => {
                    write!(self.out, "(")?;
                    self.export_list(ranges, |e, range| e.export_discrete_range(range))?;
                    write!(self.out, ")")?;
                }
            }
        }
        Ok(())
    }

    fn export_interface_list(&mut self, list: &InterfaceList) -> io::Result<()> {
        for (i, decl) in list.items.iter().enumerate() {
            if i > 0 {
                writeln!(self.out, ";")?;
            }
            self.write_indent()?;
            if let Some(class) = &decl.class {
                let class_str = match class {
                    InterfaceClass::Constant => "constant ",
                    InterfaceClass::Signal => "signal ",
                    InterfaceClass::Variable => "variable ",
                    InterfaceClass::File => "file ",
                };
                write!(self.out, "{}", class_str)?;
            }
            self.export_list(&decl.names, |e, name| write!(e.out, "{}", &name.text))?;
            write!(self.out, ": ")?;
            if let Some(mode) = &decl.mode {
                let mode_str = match mode {
                    Mode::In => "in ",
                    Mode::Out => "out ",
                    Mode::Inout => "inout ",
                    Mode::Buffer => "buffer ",
                    Mode::Linkage => "linkage ",
                };
                write!(self.out, "{}", mode_str)?;
            }
            self.export_subtype_indication(&decl.subtype)?;
            if decl.bus {
                write!(self.out, " bus")?;
            }
            if let Some(default) = &decl.default {
                write!(self.out, " := ")?;
                self.export_expression(default)?;
            }
        }
        writeln!(self.out)?;
        Ok(())
    }

    fn export_subprogram_spec(&mut self, spec: &SubprogramSpec) -> io::Result<()> {
        match spec {
            SubprogramSpec::Procedure { name, params, .. } => {
                write!(self.out, "procedure ")?;
                write!(self.out, "{}", &name.text)?;
                if let Some(params) = params {
                    writeln!(self.out, " (")?;
                    self.with_indent(|e| e.export_interface_list(params))?;
                    self.write_indent()?;
                    write!(self.out, ")")?;
                }
            }
            SubprogramSpec::Function {
                purity,
                name,
                params,
                return_type,
                ..
            } => {
                if let Some(p) = purity {
                    match p {
                        Purity::Pure => write!(self.out, "pure ")?,
                        Purity::Impure => write!(self.out, "impure ")?,
                    }
                }
                write!(self.out, "function ")?;
                write!(self.out, "{}", &name.text)?;
                if let Some(params) = params {
                    writeln!(self.out, " (")?;
                    self.with_indent(|e| e.export_interface_list(params))?;
                    self.write_indent()?;
                    write!(self.out, ") return ")?;
                } else {
                    write!(self.out, " return ")?;
                }
                self.export_name(return_type)?;
            }
        }
        Ok(())
    }
}
