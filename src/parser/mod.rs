pub mod ast;

use crate::lexer::token::{KeywordKind, Span, Token, TokenKind};
use crate::version::VhdlVersion;
use ast::*;

/// A parser error with location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.span.line, self.span.col, self.message)
    }
}

impl std::error::Error for ParseError {}

/// The result of parsing a VHDL source.
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub design_file: DesignFile,
    pub errors: Vec<ParseError>,
}

// ─── Keyword helper ──────────────────────────────────────────────────────

fn kw(k: KeywordKind) -> TokenKind {
    TokenKind::Keyword(k)
}

// ─── Parser ──────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    version: VhdlVersion,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, version: VhdlVersion) -> Self {
        Parser {
            tokens,
            pos: 0,
            version,
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> ParseResult {
        let df = self.parse_design_file();
        ParseResult {
            design_file: df,
            errors: self.errors,
        }
    }

    // ─── Token stream helpers ────────────────────────────────────────

    fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn kind(&self) -> TokenKind {
        self.current().kind
    }
    fn span(&self) -> Span {
        self.current().span
    }

    fn at_end(&self) -> bool {
        self.kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if !self.at_end() {
            self.pos += 1;
        }
        t
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.kind() == kind
    }

    fn at_kw(&self, k: KeywordKind) -> bool {
        self.kind() == kw(k)
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, k: KeywordKind) -> bool {
        self.eat(kw(k))
    }

    fn expect(&mut self, kind: TokenKind) -> Span {
        if self.at(kind) {
            self.advance().span
        } else {
            self.error_here(&format!("expected {:?}, found {:?}", kind, self.kind()));
            self.span()
        }
    }

    fn expect_kw(&mut self, k: KeywordKind) -> Span {
        self.expect(kw(k))
    }

    fn expect_semi(&mut self) {
        self.expect(TokenKind::Semicolon);
    }

    fn expect_ident(&mut self) -> Identifier {
        if self.kind() == TokenKind::Identifier || self.kind() == TokenKind::ExtendedIdentifier {
            let t = self.advance().clone();
            Identifier {
                text: t.text,
                span: t.span,
            }
        } else {
            self.error_here("expected identifier");
            Identifier {
                text: String::new(),
                span: self.span(),
            }
        }
    }

    fn try_ident(&mut self) -> Option<Identifier> {
        if self.kind() == TokenKind::Identifier || self.kind() == TokenKind::ExtendedIdentifier {
            Some(self.expect_ident())
        } else {
            None
        }
    }

    fn error_here(&mut self, msg: &str) {
        self.errors.push(ParseError {
            message: msg.to_string(),
            span: self.span(),
        });
    }

    /// Skip tokens until we find one that could start a new declaration or statement.
    fn synchronize(&mut self) {
        while !self.at_end() {
            if self.at(TokenKind::Semicolon) {
                self.advance();
                return;
            }
            if matches!(self.kind(),
                k if k == kw(KeywordKind::Entity) || k == kw(KeywordKind::Architecture)
                    || k == kw(KeywordKind::Package) || k == kw(KeywordKind::Configuration)
                    || k == kw(KeywordKind::Library) || k == kw(KeywordKind::Use)
                    || k == kw(KeywordKind::End)
            ) {
                return;
            }
            self.advance();
        }
    }

    // ─── Design file ─────────────────────────────────────────────────

    fn parse_design_file(&mut self) -> DesignFile {
        let mut units = Vec::new();
        while !self.at_end() {
            units.push(self.parse_design_unit());
        }
        DesignFile { units }
    }

    fn parse_design_unit(&mut self) -> DesignUnit {
        let start = self.span();
        let context = self.parse_context_clause();
        let unit = self.parse_library_unit();
        let end = self.span();
        DesignUnit {
            context,
            unit,
            span: merge(start, end),
        }
    }

    fn parse_context_clause(&mut self) -> ContextClause {
        let mut items = Vec::new();
        loop {
            if self.at_kw(KeywordKind::Library) {
                items.push(ContextItem::Library(self.parse_library_clause()));
            } else if self.at_kw(KeywordKind::Use) {
                items.push(ContextItem::Use(self.parse_use_clause()));
            } else if self.version >= VhdlVersion::Vhdl2008 && self.at_kw(KeywordKind::Context) {
                // Peek ahead: CONTEXT id IS ... => context declaration (library unit)
                //              CONTEXT name ; => context reference
                if self.peek_is_context_reference() {
                    items.push(ContextItem::ContextReference(
                        self.parse_context_reference(),
                    ));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        ContextClause { items }
    }

    fn peek_is_context_reference(&self) -> bool {
        // context reference: CONTEXT selected_name { , selected_name } ;
        // context declaration: CONTEXT identifier IS ...
        // Look for IS after the identifier
        let mut i = self.pos + 1;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::Semicolon => return true,
                k if k == kw(KeywordKind::Is) => return false,
                TokenKind::Eof => return false,
                _ => i += 1,
            }
        }
        false
    }

    fn parse_library_clause(&mut self) -> LibraryClause {
        let start = self.span();
        self.expect_kw(KeywordKind::Library);
        let mut names = vec![self.expect_ident()];
        while self.eat(TokenKind::Comma) {
            names.push(self.expect_ident());
        }
        self.expect_semi();
        LibraryClause {
            names,
            span: merge(start, self.span()),
        }
    }

    fn parse_use_clause(&mut self) -> UseClause {
        let start = self.span();
        self.expect_kw(KeywordKind::Use);
        let mut names = vec![self.parse_name()];
        while self.eat(TokenKind::Comma) {
            names.push(self.parse_name());
        }
        self.expect_semi();
        UseClause {
            names,
            span: merge(start, self.span()),
        }
    }

    fn parse_context_reference(&mut self) -> ContextReference {
        let start = self.span();
        self.expect_kw(KeywordKind::Context);
        let mut names = vec![self.parse_name()];
        while self.eat(TokenKind::Comma) {
            names.push(self.parse_name());
        }
        self.expect_semi();
        ContextReference {
            names,
            span: merge(start, self.span()),
        }
    }

    // ─── Library units ───────────────────────────────────────────────

    fn parse_library_unit(&mut self) -> LibraryUnit {
        match self.kind() {
            k if k == kw(KeywordKind::Entity) => {
                LibraryUnit::Entity(self.parse_entity_declaration())
            }
            k if k == kw(KeywordKind::Architecture) => {
                LibraryUnit::Architecture(self.parse_architecture_body())
            }
            k if k == kw(KeywordKind::Package) => {
                if self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(kw(KeywordKind::Body)) {
                    LibraryUnit::PackageBody(self.parse_package_body())
                } else {
                    LibraryUnit::Package(self.parse_package_declaration())
                }
            }
            k if k == kw(KeywordKind::Configuration) => {
                LibraryUnit::Configuration(self.parse_configuration_declaration())
            }
            k if k == kw(KeywordKind::Context) => {
                LibraryUnit::ContextDeclaration(self.parse_context_declaration())
            }
            _ => {
                self.error_here(
                    "expected a library unit (entity, architecture, package, or configuration)",
                );
                self.synchronize();
                LibraryUnit::Entity(EntityDeclaration {
                    name: Identifier {
                        text: String::new(),
                        span: self.span(),
                    },
                    generics: None,
                    ports: None,
                    decls: Vec::new(),
                    stmts: Vec::new(),
                    end_name: None,
                    span: self.span(),
                })
            }
        }
    }

    // ─── Entity ──────────────────────────────────────────────────────

    fn parse_entity_declaration(&mut self) -> EntityDeclaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Entity);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Is);

        let generics = if self.at_kw(KeywordKind::Generic) {
            Some(self.parse_generic_clause())
        } else {
            None
        };
        let ports = if self.at_kw(KeywordKind::Port) {
            Some(self.parse_port_clause())
        } else {
            None
        };

        let decls = self.parse_declarative_items();

        let stmts = if self.eat_kw(KeywordKind::Begin) {
            self.parse_concurrent_statements()
        } else {
            Vec::new()
        };

        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Entity);
        let end_name = self.try_ident();
        self.expect_semi();

        EntityDeclaration {
            name,
            generics,
            ports,
            decls,
            stmts,
            end_name,
            span: merge(start, self.span()),
        }
    }

    // ─── Architecture ────────────────────────────────────────────────

    fn parse_architecture_body(&mut self) -> ArchitectureBody {
        let start = self.span();
        self.expect_kw(KeywordKind::Architecture);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Of);
        let entity_name = self.parse_name();
        self.expect_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::Begin);
        let stmts = self.parse_concurrent_statements();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Architecture);
        let end_name = self.try_ident();
        self.expect_semi();
        ArchitectureBody {
            name,
            entity_name,
            decls,
            stmts,
            end_name,
            span: merge(start, self.span()),
        }
    }

    // ─── Package ─────────────────────────────────────────────────────

    fn parse_package_declaration(&mut self) -> PackageDeclaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Package);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Package);
        let end_name = self.try_ident();
        self.expect_semi();
        PackageDeclaration {
            name,
            decls,
            end_name,
            span: merge(start, self.span()),
        }
    }

    fn parse_package_body(&mut self) -> PackageBody {
        let start = self.span();
        self.expect_kw(KeywordKind::Package);
        self.expect_kw(KeywordKind::Body);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Package);
        self.eat_kw(KeywordKind::Body);
        let end_name = self.try_ident();
        self.expect_semi();
        PackageBody {
            name,
            decls,
            end_name,
            span: merge(start, self.span()),
        }
    }

    // ─── Configuration ───────────────────────────────────────────────

    fn parse_configuration_declaration(&mut self) -> ConfigurationDeclaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Configuration);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Of);
        let entity_name = self.parse_name();
        self.expect_kw(KeywordKind::Is);
        // Skip configuration body for now
        let mut depth = 1;
        while !self.at_end() && depth > 0 {
            if self.at_kw(KeywordKind::End) {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            if self.at_kw(KeywordKind::For) {
                depth += 1;
            }
            self.advance();
        }
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Configuration);
        let end_name = self.try_ident();
        self.expect_semi();
        ConfigurationDeclaration {
            name,
            entity_name,
            decls: Vec::new(),
            end_name,
            span: merge(start, self.span()),
        }
    }

    // ─── Context declaration (VHDL-2008) ─────────────────────────────

    fn parse_context_declaration(&mut self) -> ContextDeclarationUnit {
        let start = self.span();
        self.expect_kw(KeywordKind::Context);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Is);
        let ctx = self.parse_context_clause();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Context);
        let end_name = self.try_ident();
        self.expect_semi();
        ContextDeclarationUnit {
            name,
            items: ctx.items,
            end_name,
            span: merge(start, self.span()),
        }
    }

    // ─── Generic / Port clauses ──────────────────────────────────────

    fn parse_generic_clause(&mut self) -> InterfaceList {
        self.expect_kw(KeywordKind::Generic);
        let list = self.parse_interface_list();
        self.expect_semi();
        list
    }

    fn parse_port_clause(&mut self) -> InterfaceList {
        self.expect_kw(KeywordKind::Port);
        let list = self.parse_interface_list();
        self.expect_semi();
        list
    }

    fn parse_interface_list(&mut self) -> InterfaceList {
        let start = self.span();
        self.expect(TokenKind::LeftParen);
        let mut items = vec![self.parse_interface_declaration()];
        while self.eat(TokenKind::Semicolon) {
            if self.at(TokenKind::RightParen) {
                break;
            }
            items.push(self.parse_interface_declaration());
        }
        self.expect(TokenKind::RightParen);
        InterfaceList {
            items,
            span: merge(start, self.span()),
        }
    }

    fn parse_interface_declaration(&mut self) -> InterfaceDeclaration {
        let start = self.span();
        let class = match self.kind() {
            k if k == kw(KeywordKind::Constant) => {
                self.advance();
                Some(InterfaceClass::Constant)
            }
            k if k == kw(KeywordKind::Signal) => {
                self.advance();
                Some(InterfaceClass::Signal)
            }
            k if k == kw(KeywordKind::Variable) => {
                self.advance();
                Some(InterfaceClass::Variable)
            }
            k if k == kw(KeywordKind::File) => {
                self.advance();
                Some(InterfaceClass::File)
            }
            _ => None,
        };
        let mut names = vec![self.expect_ident()];
        while self.eat(TokenKind::Comma) {
            names.push(self.expect_ident());
        }
        self.expect(TokenKind::Colon);
        let mode = self.parse_optional_mode();
        let subtype = self.parse_subtype_indication();
        let bus = self.eat_kw(KeywordKind::Bus);
        let default = if self.eat(TokenKind::VarAssign) {
            Some(self.parse_expression())
        } else {
            None
        };
        InterfaceDeclaration {
            class,
            names,
            mode,
            subtype,
            bus,
            default,
            span: merge(start, self.span()),
        }
    }

    fn parse_optional_mode(&mut self) -> Option<Mode> {
        match self.kind() {
            k if k == kw(KeywordKind::In) => {
                self.advance();
                Some(Mode::In)
            }
            k if k == kw(KeywordKind::Out) => {
                self.advance();
                Some(Mode::Out)
            }
            k if k == kw(KeywordKind::Inout) => {
                self.advance();
                Some(Mode::Inout)
            }
            k if k == kw(KeywordKind::Buffer) => {
                self.advance();
                Some(Mode::Buffer)
            }
            k if k == kw(KeywordKind::Linkage) => {
                self.advance();
                Some(Mode::Linkage)
            }
            _ => None,
        }
    }

    // ─── Declarative items ───────────────────────────────────────────

    fn parse_declarative_items(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            match self.kind() {
                k if k == kw(KeywordKind::Signal) => decls.push(self.parse_signal_declaration()),
                k if k == kw(KeywordKind::Constant) => {
                    decls.push(self.parse_constant_declaration())
                }
                k if k == kw(KeywordKind::Variable) => {
                    decls.push(self.parse_variable_declaration(false))
                }
                k if k == kw(KeywordKind::Shared) => {
                    self.advance();
                    decls.push(self.parse_variable_declaration(true));
                }
                k if k == kw(KeywordKind::File) => decls.push(self.parse_file_declaration()),
                k if k == kw(KeywordKind::Type) => decls.push(self.parse_type_declaration()),
                k if k == kw(KeywordKind::Subtype) => decls.push(self.parse_subtype_declaration()),
                k if k == kw(KeywordKind::Alias) => decls.push(self.parse_alias_declaration()),
                k if k == kw(KeywordKind::Component) => {
                    decls.push(self.parse_component_declaration())
                }
                k if k == kw(KeywordKind::Attribute) => {
                    decls.push(self.parse_attribute_decl_or_spec())
                }
                k if k == kw(KeywordKind::Use) => {
                    decls.push(Declaration::Use(self.parse_use_clause()))
                }
                k if k == kw(KeywordKind::Function)
                    || k == kw(KeywordKind::Procedure)
                    || k == kw(KeywordKind::Pure)
                    || k == kw(KeywordKind::Impure) =>
                {
                    decls.push(self.parse_subprogram_decl_or_body());
                }
                k if k == kw(KeywordKind::For) => {
                    decls.push(self.parse_config_spec());
                }
                k if k == kw(KeywordKind::Disconnect) => {
                    decls.push(self.parse_disconnect_spec());
                }
                _ => break,
            }
        }
        decls
    }

    fn parse_signal_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Signal);
        let names = self.parse_identifier_list();
        self.expect(TokenKind::Colon);
        let subtype = self.parse_subtype_indication();
        let default = if self.eat(TokenKind::VarAssign) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        Declaration::Signal(ObjectDeclaration {
            shared: false,
            names,
            subtype,
            default,
            span: merge(start, self.span()),
        })
    }

    fn parse_constant_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Constant);
        let names = self.parse_identifier_list();
        self.expect(TokenKind::Colon);
        let subtype = self.parse_subtype_indication();
        let default = if self.eat(TokenKind::VarAssign) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        Declaration::Constant(ObjectDeclaration {
            shared: false,
            names,
            subtype,
            default,
            span: merge(start, self.span()),
        })
    }

    fn parse_variable_declaration(&mut self, shared: bool) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Variable);
        let names = self.parse_identifier_list();
        self.expect(TokenKind::Colon);
        let subtype = self.parse_subtype_indication();
        let default = if self.eat(TokenKind::VarAssign) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        Declaration::Variable(ObjectDeclaration {
            shared,
            names,
            subtype,
            default,
            span: merge(start, self.span()),
        })
    }

    fn parse_file_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::File);
        let names = self.parse_identifier_list();
        self.expect(TokenKind::Colon);
        let subtype = self.parse_subtype_indication();
        // skip file_open_information
        while !self.at_end() && !self.at(TokenKind::Semicolon) {
            self.advance();
        }
        self.expect_semi();
        Declaration::File(ObjectDeclaration {
            shared: false,
            names,
            subtype,
            default: None,
            span: merge(start, self.span()),
        })
    }

    fn parse_identifier_list(&mut self) -> Vec<Identifier> {
        let mut names = vec![self.expect_ident()];
        while self.eat(TokenKind::Comma) {
            names.push(self.expect_ident());
        }
        names
    }

    fn parse_type_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Type);
        let name = self.expect_ident();
        if self.eat(TokenKind::Semicolon) {
            return Declaration::Type(TypeDeclaration {
                name,
                def: None,
                span: merge(start, self.span()),
            });
        }
        self.expect_kw(KeywordKind::Is);
        let def = self.parse_type_definition();
        self.expect_semi();
        Declaration::Type(TypeDeclaration {
            name,
            def: Some(def),
            span: merge(start, self.span()),
        })
    }

    fn parse_type_definition(&mut self) -> TypeDefinition {
        if self.at(TokenKind::LeftParen) {
            return self.parse_enumeration_type();
        }
        if self.at_kw(KeywordKind::Array) {
            return self.parse_array_type();
        }
        if self.at_kw(KeywordKind::Record) {
            return self.parse_record_type();
        }
        if self.at_kw(KeywordKind::Access) {
            self.advance();
            let si = self.parse_subtype_indication();
            return TypeDefinition::Access(si);
        }
        if self.at_kw(KeywordKind::File) {
            self.advance();
            self.expect_kw(KeywordKind::Of);
            let tm = self.parse_name();
            return TypeDefinition::File(tm);
        }
        if self.at_kw(KeywordKind::Range) {
            self.advance();
            let r = self.parse_range();
            let rc = RangeConstraint {
                span: self.span(),
                range: r,
            };
            return TypeDefinition::Integer(rc);
        }
        // Fallback: try range constraint from expression
        let r = self.parse_range();
        let rc = RangeConstraint {
            span: self.span(),
            range: r,
        };
        TypeDefinition::Integer(rc)
    }

    fn parse_enumeration_type(&mut self) -> TypeDefinition {
        self.expect(TokenKind::LeftParen);
        let mut lits = Vec::new();
        loop {
            if self.kind() == TokenKind::CharacterLiteral {
                let t = self.advance().clone();
                lits.push(EnumerationLiteral::Character(t.text, t.span));
            } else {
                let id = self.expect_ident();
                lits.push(EnumerationLiteral::Identifier(id));
            }
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RightParen);
        TypeDefinition::Enumeration(lits)
    }

    fn parse_array_type(&mut self) -> TypeDefinition {
        self.expect_kw(KeywordKind::Array);
        self.expect(TokenKind::LeftParen);
        // Try to determine if unconstrained or constrained
        let first_range = self.parse_discrete_range_or_index_subtype();
        let mut ranges = vec![first_range];
        while self.eat(TokenKind::Comma) {
            ranges.push(self.parse_discrete_range_or_index_subtype());
        }
        self.expect(TokenKind::RightParen);
        self.expect_kw(KeywordKind::Of);
        let elem = self.parse_subtype_indication();

        // Check if all ranges are unconstrained index subtypes (type_mark RANGE <>)
        let all_unconstrained = ranges
            .iter()
            .all(|r| matches!(r, DiscreteRange::Range(Range::Attribute(_))));
        if all_unconstrained && !ranges.is_empty() {
            // Check if we stored Name placeholders for unconstrained
            let index_subtypes = ranges
                .into_iter()
                .map(|r| match r {
                    DiscreteRange::Range(Range::Attribute(n)) => n,
                    _ => unreachable!(),
                })
                .collect();
            TypeDefinition::Array(ArrayTypeDefinition::Unconstrained {
                index_subtypes,
                element_subtype: elem,
            })
        } else {
            TypeDefinition::Array(ArrayTypeDefinition::Constrained {
                index_constraint: ranges,
                element_subtype: elem,
            })
        }
    }

    fn parse_discrete_range_or_index_subtype(&mut self) -> DiscreteRange {
        let name = self.parse_name();
        if self.at_kw(KeywordKind::Range) {
            self.advance();
            if self.eat(TokenKind::Box) {
                // unconstrained: type_mark RANGE <>
                return DiscreteRange::Range(Range::Attribute(name));
            }
            let r = self.parse_range();
            return DiscreteRange::Range(r);
        }
        // Could be a subtype indication or expression range
        if self.at_kw(KeywordKind::To) || self.at_kw(KeywordKind::Downto) {
            let left = Box::new(Expression::Name(Box::new(name)));
            let dir = if self.eat_kw(KeywordKind::To) {
                Direction::To
            } else {
                self.advance();
                Direction::Downto
            };
            let right = Box::new(self.parse_expression());
            return DiscreteRange::Range(Range::Expr {
                left,
                direction: dir,
                right,
            });
        }
        // Subtype indication
        let constraint = if self.at(TokenKind::LeftParen) || self.at_kw(KeywordKind::Range) {
            Some(self.parse_constraint())
        } else {
            None
        };
        let span = self.span();
        DiscreteRange::Subtype(Box::new(SubtypeIndication {
            type_mark: Box::new(name),
            constraint,
            span,
        }))
    }

    fn parse_record_type(&mut self) -> TypeDefinition {
        self.expect_kw(KeywordKind::Record);
        let mut elems = Vec::new();
        while !self.at_kw(KeywordKind::End) && !self.at_end() {
            let start = self.span();
            let names = self.parse_identifier_list();
            self.expect(TokenKind::Colon);
            let subtype = self.parse_subtype_indication();
            self.expect_semi();
            elems.push(ElementDeclaration {
                names,
                subtype,
                span: merge(start, self.span()),
            });
        }
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Record);
        self.try_ident(); // optional record type simple name
        TypeDefinition::Record(elems)
    }

    fn parse_subtype_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Subtype);
        let name = self.expect_ident();
        self.expect_kw(KeywordKind::Is);
        let indication = self.parse_subtype_indication();
        self.expect_semi();
        Declaration::Subtype(SubtypeDeclaration {
            name,
            indication,
            span: merge(start, self.span()),
        })
    }

    fn parse_subtype_indication(&mut self) -> SubtypeIndication {
        let start = self.span();
        let type_mark = Box::new(self.parse_type_mark());
        let constraint = if self.at(TokenKind::LeftParen) || self.at_kw(KeywordKind::Range) {
            Some(self.parse_constraint())
        } else {
            None
        };
        SubtypeIndication {
            type_mark,
            constraint,
            span: merge(start, self.span()),
        }
    }

    /// Parse a type_mark: `name { . name }` — selected names only, no indexing or attributes.
    fn parse_type_mark(&mut self) -> Name {
        let mut name = self.parse_simple_name();
        while self.at(TokenKind::Dot) {
            self.advance();
            if self.eat_kw(KeywordKind::All) {
                let id = Identifier {
                    text: "all".into(),
                    span: self.span(),
                };
                name = Name::Selected(Box::new(name), id);
            } else {
                let suffix = self.expect_ident();
                name = Name::Selected(Box::new(name), suffix);
            }
        }
        name
    }

    fn parse_constraint(&mut self) -> Constraint {
        if self.at_kw(KeywordKind::Range) {
            self.advance();
            let r = self.parse_range();
            Constraint::Range(RangeConstraint {
                span: self.span(),
                range: r,
            })
        } else {
            self.expect(TokenKind::LeftParen);
            let mut ranges = vec![self.parse_discrete_range()];
            while self.eat(TokenKind::Comma) {
                ranges.push(self.parse_discrete_range());
            }
            self.expect(TokenKind::RightParen);
            Constraint::Index(ranges)
        }
    }

    fn parse_discrete_range(&mut self) -> DiscreteRange {
        let expr = self.parse_expression();
        if self.at_kw(KeywordKind::To) || self.at_kw(KeywordKind::Downto) {
            let dir = if self.eat_kw(KeywordKind::To) {
                Direction::To
            } else {
                self.advance();
                Direction::Downto
            };
            let right = self.parse_expression();
            DiscreteRange::Range(Range::Expr {
                left: Box::new(expr),
                direction: dir,
                right: Box::new(right),
            })
        } else {
            // It's a subtype indication or just the expression as-is
            match expr {
                Expression::Name(n) => {
                    let constraint = if self.at_kw(KeywordKind::Range) {
                        self.advance();
                        let r = self.parse_range();
                        Some(Constraint::Range(RangeConstraint {
                            span: self.span(),
                            range: r,
                        }))
                    } else {
                        None
                    };
                    DiscreteRange::Subtype(Box::new(SubtypeIndication {
                        type_mark: Box::new(*n),
                        constraint,
                        span: self.span(),
                    }))
                }
                _ => DiscreteRange::Range(Range::Expr {
                    left: Box::new(expr),
                    direction: Direction::To,
                    right: Box::new(Expression::Literal(LiteralValue::Integer(
                        "0".into(),
                        self.span(),
                    ))),
                }),
            }
        }
    }

    fn parse_range(&mut self) -> Range {
        let expr = self.parse_expression();
        if self.at_kw(KeywordKind::To) || self.at_kw(KeywordKind::Downto) {
            let dir = if self.eat_kw(KeywordKind::To) {
                Direction::To
            } else {
                self.advance();
                Direction::Downto
            };
            let right = self.parse_expression();
            Range::Expr {
                left: Box::new(expr),
                direction: dir,
                right: Box::new(right),
            }
        } else {
            match expr {
                Expression::Name(n) => Range::Attribute(*n),
                _ => Range::Attribute(Name::Simple(Identifier {
                    text: String::new(),
                    span: self.span(),
                })),
            }
        }
    }

    fn parse_alias_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Alias);
        let designator = self.expect_ident();
        let subtype = if self.eat(TokenKind::Colon) {
            Some(self.parse_subtype_indication())
        } else {
            None
        };
        self.expect_kw(KeywordKind::Is);
        let name = self.parse_name();
        self.expect_semi();
        Declaration::Alias(AliasDeclaration {
            designator,
            subtype,
            name,
            span: merge(start, self.span()),
        })
    }

    fn parse_component_declaration(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Component);
        let name = self.expect_ident();
        self.eat_kw(KeywordKind::Is);
        let generics = if self.at_kw(KeywordKind::Generic) {
            Some(self.parse_generic_clause())
        } else {
            None
        };
        let ports = if self.at_kw(KeywordKind::Port) {
            Some(self.parse_port_clause())
        } else {
            None
        };
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Component);
        let end_name = self.try_ident();
        self.expect_semi();
        Declaration::Component(ComponentDeclaration {
            name,
            generics,
            ports,
            end_name,
            span: merge(start, self.span()),
        })
    }

    fn parse_attribute_decl_or_spec(&mut self) -> Declaration {
        let start = self.span();
        self.expect_kw(KeywordKind::Attribute);
        let name = self.expect_ident();
        if self.eat(TokenKind::Colon) {
            let type_mark = self.parse_name();
            self.expect_semi();
            Declaration::Attribute(AttributeDeclaration {
                name,
                type_mark,
                span: merge(start, self.span()),
            })
        } else {
            // attribute specification: ATTRIBUTE attr OF entity_spec IS expr ;
            self.expect_kw(KeywordKind::Of);
            let entity_spec = self.parse_entity_specification();
            self.expect_kw(KeywordKind::Is);
            let value = self.parse_expression();
            self.expect_semi();
            Declaration::AttributeSpec(AttributeSpecification {
                designator: name,
                entity_spec,
                value,
                span: merge(start, self.span()),
            })
        }
    }

    fn parse_entity_specification(&mut self) -> EntitySpecification {
        let start = self.span();
        let names = if self.at_kw(KeywordKind::Others) {
            self.advance();
            EntityNameList::Others
        } else if self.at_kw(KeywordKind::All) {
            self.advance();
            EntityNameList::All
        } else {
            let mut ns = vec![self.expect_ident()];
            while self.eat(TokenKind::Comma) {
                ns.push(self.expect_ident());
            }
            EntityNameList::Names(ns)
        };
        self.expect(TokenKind::Colon);
        let entity_class = self.expect_ident();
        EntitySpecification {
            names,
            entity_class,
            span: merge(start, self.span()),
        }
    }

    fn parse_subprogram_decl_or_body(&mut self) -> Declaration {
        let spec = self.parse_subprogram_specification();
        if self.eat(TokenKind::Semicolon) {
            return Declaration::SubprogramDecl(spec);
        }
        let start = self.span();
        self.expect_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::Begin);
        let stmts = self.parse_sequential_statements();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Function);
        self.eat_kw(KeywordKind::Procedure);
        let end_name = self.try_ident();
        self.expect_semi();
        Declaration::SubprogramBody(SubprogramBody {
            spec,
            decls,
            stmts,
            end_name,
            span: merge(start, self.span()),
        })
    }

    fn parse_subprogram_specification(&mut self) -> SubprogramSpec {
        let start = self.span();
        let purity = if self.eat_kw(KeywordKind::Pure) {
            Some(Purity::Pure)
        } else if self.eat_kw(KeywordKind::Impure) {
            Some(Purity::Impure)
        } else {
            None
        };
        if self.eat_kw(KeywordKind::Procedure) {
            let name = self.expect_ident();
            let params = if self.at(TokenKind::LeftParen) {
                Some(self.parse_interface_list())
            } else {
                None
            };
            SubprogramSpec::Procedure {
                name,
                params,
                span: merge(start, self.span()),
            }
        } else {
            self.expect_kw(KeywordKind::Function);
            let name = if self.kind() == TokenKind::StringLiteral {
                let t = self.advance().clone();
                Identifier {
                    text: t.text,
                    span: t.span,
                }
            } else {
                self.expect_ident()
            };
            let params = if self.at(TokenKind::LeftParen) {
                Some(self.parse_interface_list())
            } else {
                None
            };
            self.expect_kw(KeywordKind::Return);
            let return_type = self.parse_name();
            SubprogramSpec::Function {
                purity,
                name,
                params,
                return_type,
                span: merge(start, self.span()),
            }
        }
    }

    fn parse_config_spec(&mut self) -> Declaration {
        let start = self.span();
        // Skip FOR component_spec binding_indication ;
        while !self.at_end() && !self.at(TokenKind::Semicolon) {
            self.advance();
        }
        self.expect_semi();
        Declaration::ConfigSpec(ConfigurationSpecification {
            span: merge(start, self.span()),
        })
    }

    fn parse_disconnect_spec(&mut self) -> Declaration {
        let start = self.span();
        while !self.at_end() && !self.at(TokenKind::Semicolon) {
            self.advance();
        }
        self.expect_semi();
        Declaration::Disconnection(DisconnectionSpecification {
            span: merge(start, self.span()),
        })
    }

    // ─── Concurrent statements ───────────────────────────────────────

    fn parse_concurrent_statements(&mut self) -> Vec<ConcurrentStatement> {
        let mut stmts = Vec::new();
        while !self.at_kw(KeywordKind::End) && !self.at_end() {
            stmts.push(self.parse_concurrent_statement());
        }
        stmts
    }

    fn parse_concurrent_statement(&mut self) -> ConcurrentStatement {
        // Check for labeled statement: identifier : ...
        if self.kind() == TokenKind::Identifier
            && self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            let label = self.expect_ident();
            self.expect(TokenKind::Colon);
            return self.parse_labeled_concurrent(label);
        }
        if self.at_kw(KeywordKind::Process) || self.at_kw(KeywordKind::Postponed) {
            return ConcurrentStatement::Process(self.parse_process_statement(None));
        }
        if self.at_kw(KeywordKind::Assert) {
            return self.parse_concurrent_assert(None, false);
        }
        // Default: concurrent signal assignment or procedure call
        self.parse_concurrent_signal_or_call(None, false)
    }

    fn parse_labeled_concurrent(&mut self, label: Identifier) -> ConcurrentStatement {
        if self.at_kw(KeywordKind::Process) || self.at_kw(KeywordKind::Postponed) {
            return ConcurrentStatement::Process(self.parse_process_statement(Some(label)));
        }
        if self.at_kw(KeywordKind::Block) {
            return ConcurrentStatement::Block(self.parse_block_statement(label));
        }
        if self.at_kw(KeywordKind::For) || self.at_kw(KeywordKind::If) {
            return ConcurrentStatement::Generate(self.parse_generate_statement(label));
        }
        if self.at_kw(KeywordKind::Assert) {
            return self.parse_concurrent_assert(Some(label), false);
        }
        // Component instantiation or signal assignment
        self.parse_instantiation_or_assignment(label)
    }

    fn parse_process_statement(&mut self, label: Option<Identifier>) -> ProcessStatement {
        let start = self.span();
        let postponed = self.eat_kw(KeywordKind::Postponed);
        self.expect_kw(KeywordKind::Process);
        let sensitivity_list = if self.eat(TokenKind::LeftParen) {
            if self.eat_kw(KeywordKind::All) {
                self.expect(TokenKind::RightParen);
                Some(SensitivityList::All)
            } else {
                let mut names = vec![self.parse_name()];
                while self.eat(TokenKind::Comma) {
                    names.push(self.parse_name());
                }
                self.expect(TokenKind::RightParen);
                Some(SensitivityList::Names(names))
            }
        } else {
            None
        };
        self.eat_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::Begin);
        let stmts = self.parse_sequential_statements();
        self.expect_kw(KeywordKind::End);
        self.eat_kw(KeywordKind::Postponed);
        self.expect_kw(KeywordKind::Process);
        let end_label = self.try_ident();
        self.expect_semi();
        ProcessStatement {
            label,
            postponed,
            sensitivity_list,
            decls,
            stmts,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_block_statement(&mut self, label: Identifier) -> BlockStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Block);
        let guard = if self.eat(TokenKind::LeftParen) {
            let e = self.parse_expression();
            self.expect(TokenKind::RightParen);
            Some(e)
        } else {
            None
        };
        self.eat_kw(KeywordKind::Is);
        let decls = self.parse_declarative_items();
        self.expect_kw(KeywordKind::Begin);
        let stmts = self.parse_concurrent_statements();
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Block);
        let end_label = self.try_ident();
        self.expect_semi();
        BlockStatement {
            label,
            guard,
            decls,
            stmts,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_generate_statement(&mut self, label: Identifier) -> GenerateStatement {
        let start = self.span();
        let scheme = if self.eat_kw(KeywordKind::For) {
            let param = self.expect_ident();
            self.expect_kw(KeywordKind::In);
            let range = self.parse_discrete_range();
            GenerationScheme::For { param, range }
        } else {
            self.expect_kw(KeywordKind::If);
            let condition = self.parse_expression();
            GenerationScheme::If { condition }
        };
        self.expect_kw(KeywordKind::Generate);
        let decls = self.parse_declarative_items();
        if self.eat_kw(KeywordKind::Begin) { /* optional BEGIN after declarative items */ }
        let stmts = self.parse_concurrent_statements();
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Generate);
        let end_label = self.try_ident();
        self.expect_semi();
        GenerateStatement {
            label,
            scheme,
            decls,
            stmts,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_concurrent_assert(
        &mut self,
        label: Option<Identifier>,
        postponed: bool,
    ) -> ConcurrentStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Assert);
        let condition = self.parse_expression();
        let report = if self.eat_kw(KeywordKind::Report) {
            Some(self.parse_expression())
        } else {
            None
        };
        let severity = if self.eat_kw(KeywordKind::Severity) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        ConcurrentStatement::Assert(ConcurrentAssertStatement {
            label,
            postponed,
            condition,
            report,
            severity,
            span: merge(start, self.span()),
        })
    }

    fn parse_concurrent_signal_or_call(
        &mut self,
        label: Option<Identifier>,
        postponed: bool,
    ) -> ConcurrentStatement {
        let start = self.span();
        let name = self.parse_name();
        if self.at(TokenKind::LtEquals) {
            // signal assignment
            self.advance();
            let waveforms = self.parse_waveform();
            self.expect_semi();
            let target = name;
            ConcurrentStatement::SignalAssignment(ConcurrentSignalAssignment {
                label,
                postponed,
                target,
                waveforms,
                span: merge(start, self.span()),
            })
        } else {
            // procedure call
            let args = if self.at(TokenKind::LeftParen) {
                Some(self.parse_association_list())
            } else {
                None
            };
            self.expect_semi();
            ConcurrentStatement::ProcedureCall(ConcurrentProcedureCall {
                label,
                postponed,
                name,
                args,
                span: merge(start, self.span()),
            })
        }
    }

    fn parse_instantiation_or_assignment(&mut self, label: Identifier) -> ConcurrentStatement {
        let start = self.span();
        // Check for direct entity/configuration instantiation (VHDL-93+)
        if self.at_kw(KeywordKind::Entity)
            || self.at_kw(KeywordKind::Configuration)
            || self.at_kw(KeywordKind::Component)
        {
            let unit = self.parse_instantiated_unit();
            let generic_map = if self.at_kw(KeywordKind::Generic) {
                self.advance();
                self.expect_kw(KeywordKind::Map);
                Some(self.parse_association_list())
            } else {
                None
            };
            let port_map = if self.at_kw(KeywordKind::Port) {
                self.advance();
                self.expect_kw(KeywordKind::Map);
                Some(self.parse_association_list())
            } else {
                None
            };
            self.expect_semi();
            return ConcurrentStatement::ComponentInstantiation(ComponentInstantiation {
                label,
                unit,
                generic_map,
                port_map,
                span: merge(start, self.span()),
            });
        }
        // Component instantiation by name, or signal assignment, or procedure call
        let name = self.parse_name();
        if self.at(TokenKind::LtEquals) {
            self.advance();
            let waveforms = self.parse_waveform();
            self.expect_semi();
            return ConcurrentStatement::SignalAssignment(ConcurrentSignalAssignment {
                label: Some(label),
                postponed: false,
                target: name,
                waveforms,
                span: merge(start, self.span()),
            });
        }
        // Must be component instantiation or procedure call
        let generic_map = if self.at_kw(KeywordKind::Generic) {
            self.advance();
            self.expect_kw(KeywordKind::Map);
            Some(self.parse_association_list())
        } else {
            None
        };
        let port_map = if self.at_kw(KeywordKind::Port) {
            self.advance();
            self.expect_kw(KeywordKind::Map);
            Some(self.parse_association_list())
        } else {
            None
        };
        if generic_map.is_some() || port_map.is_some() {
            self.expect_semi();
            ConcurrentStatement::ComponentInstantiation(ComponentInstantiation {
                label,
                unit: InstantiatedUnit::Component(name),
                generic_map,
                port_map,
                span: merge(start, self.span()),
            })
        } else {
            let args = if self.at(TokenKind::LeftParen) {
                Some(self.parse_association_list())
            } else {
                None
            };
            self.expect_semi();
            ConcurrentStatement::ProcedureCall(ConcurrentProcedureCall {
                label: Some(label),
                postponed: false,
                name,
                args,
                span: merge(start, self.span()),
            })
        }
    }

    fn parse_instantiated_unit(&mut self) -> InstantiatedUnit {
        if self.eat_kw(KeywordKind::Entity) {
            let name = self.parse_name();
            let arch = if self.eat(TokenKind::LeftParen) {
                let a = self.expect_ident();
                self.expect(TokenKind::RightParen);
                Some(a)
            } else {
                None
            };
            InstantiatedUnit::Entity(name, arch)
        } else if self.eat_kw(KeywordKind::Configuration) {
            InstantiatedUnit::Configuration(self.parse_name())
        } else {
            self.eat_kw(KeywordKind::Component);
            InstantiatedUnit::Component(self.parse_name())
        }
    }

    fn parse_association_list(&mut self) -> AssociationList {
        let start = self.span();
        self.expect(TokenKind::LeftParen);
        let mut elements = vec![self.parse_association_element()];
        while self.eat(TokenKind::Comma) {
            elements.push(self.parse_association_element());
        }
        self.expect(TokenKind::RightParen);
        AssociationList {
            elements,
            span: merge(start, self.span()),
        }
    }

    fn parse_association_element(&mut self) -> AssociationElement {
        let start = self.span();
        let expr = self.parse_expression();
        if self.eat(TokenKind::Arrow) {
            let actual = self.parse_expression();
            let formal = match expr {
                Expression::Name(n) => Some(*n),
                _ => None,
            };
            AssociationElement {
                formal,
                actual,
                span: merge(start, self.span()),
            }
        } else {
            AssociationElement {
                formal: None,
                actual: expr,
                span: merge(start, self.span()),
            }
        }
    }

    fn parse_waveform(&mut self) -> Vec<WaveformEntry> {
        let mut entries = Vec::new();
        let value = self.parse_expression();
        let after = if self.eat_kw(KeywordKind::After) {
            Some(self.parse_expression())
        } else {
            None
        };
        entries.push(WaveformEntry { value, after });
        while self.eat(TokenKind::Comma) {
            let value = self.parse_expression();
            let after = if self.eat_kw(KeywordKind::After) {
                Some(self.parse_expression())
            } else {
                None
            };
            entries.push(WaveformEntry { value, after });
        }
        entries
    }

    // ─── Sequential statements ───────────────────────────────────────

    fn parse_sequential_statements(&mut self) -> Vec<SequentialStatement> {
        let mut stmts = Vec::new();
        while !self.at_kw(KeywordKind::End)
            && !self.at_kw(KeywordKind::Elsif)
            && !self.at_kw(KeywordKind::Else)
            && !self.at_kw(KeywordKind::When)
            && !self.at_end()
        {
            stmts.push(self.parse_sequential_statement());
        }
        stmts
    }

    fn parse_sequential_statement(&mut self) -> SequentialStatement {
        // Check for label: identifier :
        let label = if self.kind() == TokenKind::Identifier
            && self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(TokenKind::Colon)
        {
            let l = self.expect_ident();
            self.expect(TokenKind::Colon);
            Some(l)
        } else {
            None
        };

        match self.kind() {
            k if k == kw(KeywordKind::If) => {
                SequentialStatement::If(self.parse_if_statement(label))
            }
            k if k == kw(KeywordKind::Case) => {
                SequentialStatement::Case(self.parse_case_statement(label))
            }
            k if k == kw(KeywordKind::While)
                || k == kw(KeywordKind::For)
                || k == kw(KeywordKind::Loop) =>
            {
                SequentialStatement::Loop(self.parse_loop_statement(label))
            }
            k if k == kw(KeywordKind::Next) => {
                SequentialStatement::Next(self.parse_next_statement(label))
            }
            k if k == kw(KeywordKind::Exit) => {
                SequentialStatement::Exit(self.parse_exit_statement(label))
            }
            k if k == kw(KeywordKind::Return) => {
                SequentialStatement::Return(self.parse_return_statement(label))
            }
            k if k == kw(KeywordKind::Null) => {
                SequentialStatement::Null(self.parse_null_statement(label))
            }
            k if k == kw(KeywordKind::Wait) => {
                SequentialStatement::Wait(self.parse_wait_statement(label))
            }
            k if k == kw(KeywordKind::Assert) => {
                SequentialStatement::Assert(self.parse_assert_statement(label))
            }
            k if k == kw(KeywordKind::Report) => {
                SequentialStatement::Report(self.parse_report_statement(label))
            }
            _ => self.parse_assignment_or_call(label),
        }
    }

    fn parse_if_statement(&mut self, label: Option<Identifier>) -> IfStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::If);
        let condition = self.parse_expression();
        self.expect_kw(KeywordKind::Then);
        let then_stmts = self.parse_sequential_statements();
        let mut elsif_branches = Vec::new();
        while self.eat_kw(KeywordKind::Elsif) {
            let cond = self.parse_expression();
            self.expect_kw(KeywordKind::Then);
            let stmts = self.parse_sequential_statements();
            elsif_branches.push(ElsifBranch {
                condition: cond,
                stmts,
            });
        }
        let else_stmts = if self.eat_kw(KeywordKind::Else) {
            Some(self.parse_sequential_statements())
        } else {
            None
        };
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::If);
        let end_label = self.try_ident();
        self.expect_semi();
        IfStatement {
            label,
            condition,
            then_stmts,
            elsif_branches,
            else_stmts,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_case_statement(&mut self, label: Option<Identifier>) -> CaseStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Case);
        let expression = self.parse_expression();
        self.expect_kw(KeywordKind::Is);
        let mut alternatives = Vec::new();
        while self.at_kw(KeywordKind::When) {
            let alt_start = self.span();
            self.advance();
            let choices = self.parse_choices();
            self.expect(TokenKind::Arrow);
            let stmts = self.parse_sequential_statements();
            alternatives.push(CaseAlternative {
                choices,
                stmts,
                span: merge(alt_start, self.span()),
            });
        }
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Case);
        let end_label = self.try_ident();
        self.expect_semi();
        CaseStatement {
            label,
            expression,
            alternatives,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_choices(&mut self) -> Vec<Choice> {
        let mut choices = vec![self.parse_choice()];
        while self.eat(TokenKind::Bar) {
            choices.push(self.parse_choice());
        }
        choices
    }

    fn parse_choice(&mut self) -> Choice {
        if self.eat_kw(KeywordKind::Others) {
            return Choice::Others;
        }
        let expr = self.parse_expression();
        if self.at_kw(KeywordKind::To) || self.at_kw(KeywordKind::Downto) {
            let dir = if self.eat_kw(KeywordKind::To) {
                Direction::To
            } else {
                self.advance();
                Direction::Downto
            };
            let right = self.parse_expression();
            Choice::DiscreteRange(DiscreteRange::Range(Range::Expr {
                left: Box::new(expr),
                direction: dir,
                right: Box::new(right),
            }))
        } else {
            Choice::Expression(expr)
        }
    }

    fn parse_loop_statement(&mut self, label: Option<Identifier>) -> LoopStatement {
        let start = self.span();
        let scheme = if self.eat_kw(KeywordKind::While) {
            Some(IterationScheme::While(self.parse_expression()))
        } else if self.eat_kw(KeywordKind::For) {
            let param = self.expect_ident();
            self.expect_kw(KeywordKind::In);
            let range = self.parse_discrete_range();
            Some(IterationScheme::For { param, range })
        } else {
            None
        };
        self.expect_kw(KeywordKind::Loop);
        let stmts = self.parse_sequential_statements();
        self.expect_kw(KeywordKind::End);
        self.expect_kw(KeywordKind::Loop);
        let end_label = self.try_ident();
        self.expect_semi();
        LoopStatement {
            label,
            scheme,
            stmts,
            end_label,
            span: merge(start, self.span()),
        }
    }

    fn parse_next_statement(&mut self, label: Option<Identifier>) -> NextStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Next);
        let loop_label = self.try_ident();
        let condition = if self.eat_kw(KeywordKind::When) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        NextStatement {
            label,
            loop_label,
            condition,
            span: merge(start, self.span()),
        }
    }

    fn parse_exit_statement(&mut self, label: Option<Identifier>) -> ExitStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Exit);
        let loop_label = self.try_ident();
        let condition = if self.eat_kw(KeywordKind::When) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        ExitStatement {
            label,
            loop_label,
            condition,
            span: merge(start, self.span()),
        }
    }

    fn parse_return_statement(&mut self, label: Option<Identifier>) -> ReturnStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Return);
        let expression = if !self.at(TokenKind::Semicolon) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        ReturnStatement {
            label,
            expression,
            span: merge(start, self.span()),
        }
    }

    fn parse_null_statement(&mut self, label: Option<Identifier>) -> NullStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Null);
        self.expect_semi();
        NullStatement {
            label,
            span: merge(start, self.span()),
        }
    }

    fn parse_wait_statement(&mut self, label: Option<Identifier>) -> WaitStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Wait);
        let sensitivity = if self.eat_kw(KeywordKind::On) {
            let mut names = vec![self.parse_name()];
            while self.eat(TokenKind::Comma) {
                names.push(self.parse_name());
            }
            Some(names)
        } else {
            None
        };
        let condition = if self.eat_kw(KeywordKind::Until) {
            Some(self.parse_expression())
        } else {
            None
        };
        let timeout = if self.eat_kw(KeywordKind::For) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        WaitStatement {
            label,
            sensitivity,
            condition,
            timeout,
            span: merge(start, self.span()),
        }
    }

    fn parse_assert_statement(&mut self, label: Option<Identifier>) -> AssertStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Assert);
        let condition = self.parse_expression();
        let report = if self.eat_kw(KeywordKind::Report) {
            Some(self.parse_expression())
        } else {
            None
        };
        let severity = if self.eat_kw(KeywordKind::Severity) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        AssertStatement {
            label,
            condition,
            report,
            severity,
            span: merge(start, self.span()),
        }
    }

    fn parse_report_statement(&mut self, label: Option<Identifier>) -> ReportStatement {
        let start = self.span();
        self.expect_kw(KeywordKind::Report);
        let expression = self.parse_expression();
        let severity = if self.eat_kw(KeywordKind::Severity) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        ReportStatement {
            label,
            expression,
            severity,
            span: merge(start, self.span()),
        }
    }

    fn parse_assignment_or_call(&mut self, label: Option<Identifier>) -> SequentialStatement {
        let start = self.span();
        let name = self.parse_name();
        if self.eat(TokenKind::LtEquals) {
            let waveforms = self.parse_waveform();
            self.expect_semi();
            SequentialStatement::SignalAssignment(SeqSignalAssignment {
                label,
                target: name,
                waveforms,
                span: merge(start, self.span()),
            })
        } else if self.eat(TokenKind::VarAssign) {
            let value = self.parse_expression();
            self.expect_semi();
            SequentialStatement::VariableAssignment(VariableAssignment {
                label,
                target: name,
                value,
                span: merge(start, self.span()),
            })
        } else {
            let args = if self.at(TokenKind::LeftParen) {
                Some(self.parse_association_list())
            } else {
                None
            };
            self.expect_semi();
            SequentialStatement::ProcedureCall(ProcedureCallStatement {
                label,
                name,
                args,
                span: merge(start, self.span()),
            })
        }
    }

    // ─── Expressions (precedence climbing) ───────────────────────────

    fn parse_expression(&mut self) -> Expression {
        self.parse_logical_expression()
    }

    fn parse_logical_expression(&mut self) -> Expression {
        let mut left = self.parse_relation();
        while let Some(op) = self.try_logical_op() {
            self.advance();
            let right = self.parse_relation();
            let span = self.span();
            left = Expression::Binary {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
                span,
            };
        }
        left
    }

    fn try_logical_op(&self) -> Option<BinaryOp> {
        match self.kind() {
            k if k == kw(KeywordKind::And) => Some(BinaryOp::And),
            k if k == kw(KeywordKind::Or) => Some(BinaryOp::Or),
            k if k == kw(KeywordKind::Xor) => Some(BinaryOp::Xor),
            k if k == kw(KeywordKind::Nand) => Some(BinaryOp::Nand),
            k if k == kw(KeywordKind::Nor) => Some(BinaryOp::Nor),
            k if k == kw(KeywordKind::Xnor) => Some(BinaryOp::Xnor),
            _ => None,
        }
    }

    fn parse_relation(&mut self) -> Expression {
        let left = self.parse_shift_expression();
        if let Some(op) = self.try_relational_op() {
            self.advance();
            let right = self.parse_shift_expression();
            let span = self.span();
            Expression::Binary {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
                span,
            }
        } else {
            left
        }
    }

    fn try_relational_op(&self) -> Option<BinaryOp> {
        match self.kind() {
            TokenKind::Equals => Some(BinaryOp::Eq),
            TokenKind::NotEquals => Some(BinaryOp::Neq),
            TokenKind::LessThan => Some(BinaryOp::Lt),
            TokenKind::LtEquals => Some(BinaryOp::Lte),
            TokenKind::GreaterThan => Some(BinaryOp::Gt),
            TokenKind::GtEquals => Some(BinaryOp::Gte),
            TokenKind::MatchEq => Some(BinaryOp::MatchEq),
            TokenKind::MatchNeq => Some(BinaryOp::MatchNeq),
            TokenKind::MatchLt => Some(BinaryOp::MatchLt),
            TokenKind::MatchLte => Some(BinaryOp::MatchLte),
            TokenKind::MatchGt => Some(BinaryOp::MatchGt),
            TokenKind::MatchGte => Some(BinaryOp::MatchGte),
            _ => None,
        }
    }

    fn parse_shift_expression(&mut self) -> Expression {
        let left = self.parse_simple_expression();
        if let Some(op) = self.try_shift_op() {
            self.advance();
            let right = self.parse_simple_expression();
            let span = self.span();
            Expression::Binary {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
                span,
            }
        } else {
            left
        }
    }

    fn try_shift_op(&self) -> Option<BinaryOp> {
        match self.kind() {
            k if k == kw(KeywordKind::Sll) => Some(BinaryOp::Sll),
            k if k == kw(KeywordKind::Srl) => Some(BinaryOp::Srl),
            k if k == kw(KeywordKind::Sla) => Some(BinaryOp::Sla),
            k if k == kw(KeywordKind::Sra) => Some(BinaryOp::Sra),
            k if k == kw(KeywordKind::Rol) => Some(BinaryOp::Rol),
            k if k == kw(KeywordKind::Ror) => Some(BinaryOp::Ror),
            _ => None,
        }
    }

    fn parse_simple_expression(&mut self) -> Expression {
        // optional leading sign
        let unary = match self.kind() {
            TokenKind::Plus => {
                self.advance();
                Some(UnaryOp::Pos)
            }
            TokenKind::Minus => {
                self.advance();
                Some(UnaryOp::Neg)
            }
            _ => None,
        };
        let mut expr = self.parse_term();
        if let Some(op) = unary {
            expr = Expression::Unary {
                op,
                operand: Box::new(expr),
                span: self.span(),
            };
        }
        while let Some(op) = self.try_adding_op() {
            self.advance();
            let right = self.parse_term();
            let span = self.span();
            expr = Expression::Binary {
                lhs: Box::new(expr),
                op,
                rhs: Box::new(right),
                span,
            };
        }
        expr
    }

    fn try_adding_op(&self) -> Option<BinaryOp> {
        match self.kind() {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Ampersand => Some(BinaryOp::Concat),
            _ => None,
        }
    }

    fn parse_term(&mut self) -> Expression {
        let mut left = self.parse_factor();
        while let Some(op) = self.try_multiplying_op() {
            self.advance();
            let right = self.parse_factor();
            let span = self.span();
            left = Expression::Binary {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
                span,
            };
        }
        left
    }

    fn try_multiplying_op(&self) -> Option<BinaryOp> {
        match self.kind() {
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            k if k == kw(KeywordKind::Mod) => Some(BinaryOp::Mod),
            k if k == kw(KeywordKind::Rem) => Some(BinaryOp::Rem),
            _ => None,
        }
    }

    fn parse_factor(&mut self) -> Expression {
        if self.eat_kw(KeywordKind::Abs) {
            let operand = self.parse_primary();
            return Expression::Unary {
                op: UnaryOp::Abs,
                operand: Box::new(operand),
                span: self.span(),
            };
        }
        if self.eat_kw(KeywordKind::Not) {
            let operand = self.parse_primary();
            return Expression::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                span: self.span(),
            };
        }
        let base = self.parse_primary();
        if self.eat(TokenKind::DoubleStar) {
            let exp = self.parse_primary();
            let span = self.span();
            Expression::Binary {
                lhs: Box::new(base),
                op: BinaryOp::Pow,
                rhs: Box::new(exp),
                span,
            }
        } else {
            base
        }
    }

    fn parse_primary(&mut self) -> Expression {
        match self.kind() {
            TokenKind::IntegerLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::Integer(t.text, t.span))
            }
            TokenKind::RealLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::Real(t.text, t.span))
            }
            TokenKind::BasedLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::Based(t.text, t.span))
            }
            TokenKind::CharacterLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::Character(t.text, t.span))
            }
            TokenKind::StringLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::String(t.text, t.span))
            }
            TokenKind::BitStringLiteral => {
                let t = self.advance().clone();
                Expression::Literal(LiteralValue::BitString(t.text, t.span))
            }
            k if k == kw(KeywordKind::Null) => {
                let s = self.span();
                self.advance();
                Expression::Literal(LiteralValue::Null(s))
            }
            k if k == kw(KeywordKind::Open) => {
                let s = self.span();
                self.advance();
                Expression::Open(s)
            }
            k if k == kw(KeywordKind::New) => {
                let s = self.span();
                self.advance();
                let expr = self.parse_primary();
                Expression::Allocator(Box::new(expr), s)
            }
            TokenKind::LeftParen => self.parse_paren_expression(),
            _ => {
                // Name (identifier, selected, indexed, attribute)
                let name = self.parse_name();
                Expression::Name(Box::new(name))
            }
        }
    }

    /// Parse a parenthesized expression or aggregate: `( ... )`
    fn parse_paren_expression(&mut self) -> Expression {
        let s = self.span();
        self.advance(); // (

        if self.at(TokenKind::RightParen) {
            self.advance();
            return Expression::Aggregate(Vec::new(), s);
        }

        // Check for aggregate with choices like (others => '0') or (0 => '1', 1 => '0')
        if self.at_kw(KeywordKind::Others) {
            self.advance();
            self.expect(TokenKind::Arrow);
            let expr = self.parse_expression();
            let mut elems = vec![ElementAssociation {
                choices: Some(vec![Choice::Others]),
                expr,
            }];
            while self.eat(TokenKind::Comma) {
                let ea = self.parse_element_association();
                elems.push(ea);
            }
            self.expect(TokenKind::RightParen);
            return Expression::Aggregate(elems, s);
        }

        let first = self.parse_expression();

        // arrow => this is an aggregate with choices
        if self.at(TokenKind::Arrow) {
            self.advance();
            let value = self.parse_expression();
            let choices = Some(vec![Choice::Expression(first)]);
            let mut elems = vec![ElementAssociation {
                choices,
                expr: value,
            }];
            while self.eat(TokenKind::Comma) {
                let ea = self.parse_element_association();
                elems.push(ea);
            }
            self.expect(TokenKind::RightParen);
            return Expression::Aggregate(elems, s);
        }

        if self.at(TokenKind::RightParen) {
            self.advance();
            return Expression::Aggregate(
                vec![ElementAssociation {
                    choices: None,
                    expr: first,
                }],
                s,
            );
        }
        if self.eat(TokenKind::Comma) {
            let mut elems = vec![ElementAssociation {
                choices: None,
                expr: first,
            }];
            loop {
                if self.at_kw(KeywordKind::Others) {
                    self.advance();
                    self.expect(TokenKind::Arrow);
                    let e = self.parse_expression();
                    elems.push(ElementAssociation {
                        choices: Some(vec![Choice::Others]),
                        expr: e,
                    });
                } else {
                    let e = self.parse_expression();
                    elems.push(ElementAssociation {
                        choices: None,
                        expr: e,
                    });
                }
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RightParen);
            return Expression::Aggregate(elems, s);
        }
        self.expect(TokenKind::RightParen);
        Expression::Aggregate(
            vec![ElementAssociation {
                choices: None,
                expr: first,
            }],
            s,
        )
    }

    fn parse_element_association(&mut self) -> ElementAssociation {
        if self.at_kw(KeywordKind::Others) {
            self.advance();
            self.expect(TokenKind::Arrow);
            let expr = self.parse_expression();
            return ElementAssociation {
                choices: Some(vec![Choice::Others]),
                expr,
            };
        }
        let expr = self.parse_expression();
        if self.eat(TokenKind::Arrow) {
            let value = self.parse_expression();
            ElementAssociation {
                choices: Some(vec![Choice::Expression(expr)]),
                expr: value,
            }
        } else {
            ElementAssociation {
                choices: None,
                expr,
            }
        }
    }

    // ─── Name parsing ────────────────────────────────────────────────

    fn parse_name(&mut self) -> Name {
        let mut name = self.parse_simple_name();
        loop {
            match self.kind() {
                TokenKind::Dot => {
                    self.advance();
                    if self.eat_kw(KeywordKind::All) {
                        let id = Identifier {
                            text: "all".into(),
                            span: self.span(),
                        };
                        name = Name::Selected(Box::new(name), id);
                    } else {
                        let suffix = self.expect_ident();
                        name = Name::Selected(Box::new(name), suffix);
                    }
                }
                TokenKind::Tick => {
                    self.advance();
                    if self.at(TokenKind::LeftParen) {
                        // qualified expression: name'(expr)
                        self.advance();
                        let expr = self.parse_expression();
                        self.expect(TokenKind::RightParen);
                        return Expression::Qualified {
                            type_mark: Box::new(name),
                            expr: Box::new(expr),
                            span: self.span(),
                        }
                        .into_name_fallback();
                    }
                    let attr = self.expect_ident();
                    let arg = if self.at(TokenKind::LeftParen) {
                        self.advance();
                        let e = self.parse_expression();
                        self.expect(TokenKind::RightParen);
                        Some(Box::new(e))
                    } else {
                        None
                    };
                    name = Name::Attribute(Box::new(name), attr, arg, self.span());
                }
                TokenKind::LeftParen => {
                    let span = self.span();
                    self.advance();
                    let mut args = vec![self.parse_expression()];
                    while self.eat(TokenKind::Comma) {
                        args.push(self.parse_expression());
                    }
                    self.expect(TokenKind::RightParen);
                    name = Name::Indexed(Box::new(name), args, span);
                }
                _ => break,
            }
        }
        name
    }

    fn parse_simple_name(&mut self) -> Name {
        if self.kind() == TokenKind::Identifier || self.kind() == TokenKind::ExtendedIdentifier {
            let id = self.expect_ident();
            Name::Simple(id)
        } else if self.kind() == TokenKind::StringLiteral {
            let t = self.advance().clone();
            Name::Operator(t.text, t.span)
        } else {
            self.error_here("expected a name");
            let s = self.span();
            self.advance(); // Always advance to prevent infinite loops
            Name::Simple(Identifier {
                text: String::new(),
                span: s,
            })
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn merge(start: Span, end: Span) -> Span {
    Span {
        start: start.start,
        end: end.end,
        line: start.line,
        col: start.col,
    }
}

impl Expression {
    /// Fallback: convert a qualified expression back into a name context
    /// (used when we discover name'(expr) inside a name chain).
    fn into_name_fallback(self) -> Name {
        match self {
            Expression::Qualified {
                type_mark,
                expr,
                span,
            } => Name::Attribute(
                type_mark,
                Identifier {
                    text: String::new(),
                    span,
                },
                Some(expr),
                span,
            ),
            _ => Name::Simple(Identifier {
                text: String::new(),
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    col: 0,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VhdlVersion, parse};

    #[test]
    fn test_empty_entity() {
        let r = parse("entity e is end entity e;", VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.design_file.units.len(), 1);
    }

    #[test]
    fn test_entity_with_ports() {
        let r = parse(
            "entity e is port (a : in std_logic; b : out std_logic); end e;",
            VhdlVersion::Vhdl1993,
        );
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        match &r.design_file.units[0].unit {
            crate::parser::ast::LibraryUnit::Entity(e) => {
                assert!(e.ports.is_some());
                assert_eq!(e.ports.as_ref().unwrap().items.len(), 2);
            }
            _ => panic!("expected entity"),
        }
    }

    #[test]
    fn test_architecture_with_signal() {
        let src = "architecture rtl of e is signal s : std_logic; begin end rtl;";
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_library_use_clauses() {
        let src = "library ieee; use ieee.std_logic_1164.all; entity e is end e;";
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        assert_eq!(r.design_file.units[0].context.items.len(), 2);
    }

    #[test]
    fn test_process_with_if() {
        let src = r#"
            architecture a of e is
            begin
                process (clk)
                begin
                    if rising_edge(clk) then
                        s <= '1';
                    else
                        s <= '0';
                    end if;
                end process;
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_case_statement() {
        let src = r#"
            architecture a of e is
            begin
                process (sel)
                begin
                    case sel is
                        when "00" => y <= a;
                        when "01" => y <= b;
                        when others => y <= '0';
                    end case;
                end process;
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_for_loop() {
        let src = r#"
            architecture a of e is
            begin
                process
                begin
                    for i in 0 to 7 loop
                        null;
                    end loop;
                    wait;
                end process;
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_expression_precedence() {
        let src = "entity e is end e; architecture a of e is begin s <= a + b * c; end a;";
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_package_declaration() {
        let src = r#"
            package p is
                constant C : integer := 42;
                function f(x : integer) return integer;
            end package p;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
        match &r.design_file.units[0].unit {
            crate::parser::ast::LibraryUnit::Package(p) => {
                assert_eq!(p.name.text, "p");
                assert_eq!(p.decls.len(), 2);
            }
            _ => panic!("expected package"),
        }
    }

    #[test]
    fn test_component_instantiation() {
        let src = r#"
            architecture a of e is
            begin
                u0 : entity work.sub_entity
                    port map (a => x, b => y);
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_subprogram_body() {
        let src = r#"
            architecture a of e is
                function add(x : integer; y : integer) return integer is
                begin
                    return x + y;
                end;
            begin
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_vhdl2008_context_declaration() {
        let src = r#"
            context my_ctx is
                library ieee;
                use ieee.std_logic_1164.all;
            end context my_ctx;
        "#;
        let r = parse(src, VhdlVersion::Vhdl2008);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_generate_statement() {
        let src = r#"
            architecture a of e is
            begin
                gen : for i in 0 to 3 generate
                    s(i) <= '0';
                end generate gen;
            end a;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_enumeration_type() {
        let src = r#"
            package p is
                type state_t is (idle, running, done);
            end p;
        "#;
        let r = parse(src, VhdlVersion::Vhdl1993);
        assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    }

    #[test]
    fn test_clockscaler_integration() {
        let src = std::fs::read_to_string("/workspaces/lambila-rs/test_vhdl/ClockScaler.vhd");
        if let Ok(source) = src {
            let r = parse(&source, VhdlVersion::Vhdl1993);
            assert!(
                r.errors.is_empty(),
                "ClockScaler parse errors: {:?}",
                r.errors
            );
            assert!(r.design_file.units.len() >= 2);
        }
    }
}
