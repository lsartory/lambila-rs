//! AST node types for the VHDL parser.
//!
//! Every node carries a [`Span`] so that error messages and tooling can
//! point back to the original source location.

use crate::lexer::token::Span;

// ─── Root ────────────────────────────────────────────────────────────────

/// A complete VHDL design file (one or more design units).
#[derive(Debug, Clone)]
pub struct DesignFile {
    pub units: Vec<DesignUnit>,
}

/// A single design unit: a context clause followed by a library unit.
#[derive(Debug, Clone)]
pub struct DesignUnit {
    pub context: ContextClause,
    pub unit: LibraryUnit,
    pub span: Span,
}

// ─── Context clause ──────────────────────────────────────────────────────

/// The context clause that precedes a library unit.
#[derive(Debug, Clone)]
pub struct ContextClause {
    pub items: Vec<ContextItem>,
}

#[derive(Debug, Clone)]
pub enum ContextItem {
    Library(LibraryClause),
    Use(UseClause),
    ContextReference(ContextReference),
}

/// `LIBRARY name { , name } ;`
#[derive(Debug, Clone)]
pub struct LibraryClause {
    pub names: Vec<Identifier>,
    pub span: Span,
}

/// `USE selected_name { , selected_name } ;`
#[derive(Debug, Clone)]
pub struct UseClause {
    pub names: Vec<Name>,
    pub span: Span,
}

/// `CONTEXT selected_name { , selected_name } ;` (VHDL-2008)
#[derive(Debug, Clone)]
pub struct ContextReference {
    pub names: Vec<Name>,
    pub span: Span,
}

// ─── Library units ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum LibraryUnit {
    Entity(EntityDeclaration),
    Architecture(ArchitectureBody),
    Package(PackageDeclaration),
    PackageBody(PackageBody),
    Configuration(ConfigurationDeclaration),
    ContextDeclaration(ContextDeclarationUnit),
}

// ─── Entity ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EntityDeclaration {
    pub name: Identifier,
    pub generics: Option<InterfaceList>,
    pub ports: Option<InterfaceList>,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<ConcurrentStatement>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Architecture ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ArchitectureBody {
    pub name: Identifier,
    pub entity_name: Name,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<ConcurrentStatement>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Package ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PackageDeclaration {
    pub name: Identifier,
    pub decls: Vec<Declaration>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PackageBody {
    pub name: Identifier,
    pub decls: Vec<Declaration>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Configuration ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConfigurationDeclaration {
    pub name: Identifier,
    pub entity_name: Name,
    pub decls: Vec<Declaration>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Context declaration (VHDL-2008) ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ContextDeclarationUnit {
    pub name: Identifier,
    pub items: Vec<ContextItem>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Subprogram ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SubprogramSpec {
    Procedure {
        name: Identifier,
        params: Option<InterfaceList>,
        span: Span,
    },
    Function {
        purity: Option<Purity>,
        name: Identifier,
        params: Option<InterfaceList>,
        return_type: Name,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purity {
    Pure,
    Impure,
}

#[derive(Debug, Clone)]
pub struct SubprogramBody {
    pub spec: SubprogramSpec,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<SequentialStatement>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

// ─── Declarations ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Declaration {
    /// `TYPE identifier IS type_definition ;`
    Type(TypeDeclaration),
    /// `SUBTYPE identifier IS subtype_indication ;`
    Subtype(SubtypeDeclaration),
    /// `CONSTANT identifier_list : subtype_indication [ := expression ] ;`
    Constant(ObjectDeclaration),
    /// `SIGNAL identifier_list : subtype_indication [ signal_kind ] [ := expression ] ;`
    Signal(ObjectDeclaration),
    /// `[SHARED] VARIABLE identifier_list : subtype_indication [ := expression ] ;`
    Variable(ObjectDeclaration),
    /// `FILE identifier_list : subtype_indication [ file_open_information ] ;`
    File(ObjectDeclaration),
    /// `ALIAS ...`
    Alias(AliasDeclaration),
    /// `COMPONENT identifier [IS] ... END COMPONENT ;`
    Component(ComponentDeclaration),
    /// `ATTRIBUTE identifier : type_mark ;`
    Attribute(AttributeDeclaration),
    /// `ATTRIBUTE attr_designator OF entity_specification IS expression ;`
    AttributeSpec(AttributeSpecification),
    /// Subprogram declaration (specification + semicolon)
    SubprogramDecl(SubprogramSpec),
    /// Subprogram body
    SubprogramBody(SubprogramBody),
    /// `USE selected_name { , selected_name } ;`
    Use(UseClause),
    /// `FOR component_spec binding_indication ;`
    ConfigSpec(ConfigurationSpecification),
    /// `DISCONNECT guarded_signal_spec AFTER time_expression ;`
    Disconnection(DisconnectionSpecification),
}

// ─── Type declarations ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypeDeclaration {
    pub name: Identifier,
    pub def: Option<TypeDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Enumeration(Vec<EnumerationLiteral>),
    Integer(RangeConstraint),
    Floating(RangeConstraint),
    Physical {
        constraint: RangeConstraint,
        base_unit: Identifier,
        secondary_units: Vec<(Identifier, Expression)>,
    },
    Array(ArrayTypeDefinition),
    Record(Vec<ElementDeclaration>),
    Access(SubtypeIndication),
    File(Name),
}

#[derive(Debug, Clone)]
pub enum EnumerationLiteral {
    Identifier(Identifier),
    Character(String, Span),
}

#[derive(Debug, Clone)]
pub enum ArrayTypeDefinition {
    Unconstrained {
        index_subtypes: Vec<Name>,
        element_subtype: SubtypeIndication,
    },
    Constrained {
        index_constraint: Vec<DiscreteRange>,
        element_subtype: SubtypeIndication,
    },
}

#[derive(Debug, Clone)]
pub struct ElementDeclaration {
    pub names: Vec<Identifier>,
    pub subtype: SubtypeIndication,
    pub span: Span,
}

// ─── Subtype indication ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SubtypeDeclaration {
    pub name: Identifier,
    pub indication: SubtypeIndication,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SubtypeIndication {
    pub type_mark: Box<Name>,
    pub constraint: Option<Constraint>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Range(RangeConstraint),
    Index(Vec<DiscreteRange>),
}

#[derive(Debug, Clone)]
pub struct RangeConstraint {
    pub range: Range,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Range {
    Attribute(Name),
    Expr {
        left: Box<Expression>,
        direction: Direction,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    To,
    Downto,
}

#[derive(Debug, Clone)]
pub enum DiscreteRange {
    Subtype(Box<SubtypeIndication>),
    Range(Range),
}

// ─── Object declarations ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ObjectDeclaration {
    pub shared: bool,
    pub names: Vec<Identifier>,
    pub subtype: SubtypeIndication,
    pub default: Option<Expression>,
    pub span: Span,
}

// ─── Alias, component, attribute ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AliasDeclaration {
    pub designator: Identifier,
    pub subtype: Option<SubtypeIndication>,
    pub name: Name,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ComponentDeclaration {
    pub name: Identifier,
    pub generics: Option<InterfaceList>,
    pub ports: Option<InterfaceList>,
    pub end_name: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AttributeDeclaration {
    pub name: Identifier,
    pub type_mark: Name,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AttributeSpecification {
    pub designator: Identifier,
    pub entity_spec: EntitySpecification,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EntitySpecification {
    pub names: EntityNameList,
    pub entity_class: Identifier,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum EntityNameList {
    Names(Vec<Identifier>),
    Others,
    All,
}

#[derive(Debug, Clone)]
pub struct ConfigurationSpecification {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DisconnectionSpecification {
    pub span: Span,
}

// ─── Interface declarations (generics / ports / params) ──────────────────

#[derive(Debug, Clone)]
pub struct InterfaceList {
    pub items: Vec<InterfaceDeclaration>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InterfaceDeclaration {
    pub class: Option<InterfaceClass>,
    pub names: Vec<Identifier>,
    pub mode: Option<Mode>,
    pub subtype: SubtypeIndication,
    pub bus: bool,
    pub default: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceClass {
    Constant,
    Signal,
    Variable,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    In,
    Out,
    Inout,
    Buffer,
    Linkage,
}

// ─── Concurrent statements ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ConcurrentStatement {
    Process(ProcessStatement),
    SignalAssignment(ConcurrentSignalAssignment),
    ComponentInstantiation(ComponentInstantiation),
    Generate(GenerateStatement),
    ProcedureCall(ConcurrentProcedureCall),
    Assert(ConcurrentAssertStatement),
    Block(BlockStatement),
}

#[derive(Debug, Clone)]
pub struct ProcessStatement {
    pub label: Option<Identifier>,
    pub postponed: bool,
    pub sensitivity_list: Option<SensitivityList>,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<SequentialStatement>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SensitivityList {
    Names(Vec<Name>),
    All,
}

#[derive(Debug, Clone)]
pub struct ConcurrentSignalAssignment {
    pub label: Option<Identifier>,
    pub postponed: bool,
    pub target: Name,
    pub waveforms: Vec<WaveformEntry>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WaveformEntry {
    pub value: Expression,
    pub after: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct ComponentInstantiation {
    pub label: Identifier,
    pub unit: InstantiatedUnit,
    pub generic_map: Option<AssociationList>,
    pub port_map: Option<AssociationList>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum InstantiatedUnit {
    Component(Name),
    Entity(Name, Option<Identifier>),
    Configuration(Name),
}

#[derive(Debug, Clone)]
pub struct AssociationList {
    pub elements: Vec<AssociationElement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssociationElement {
    pub formal: Option<Name>,
    pub actual: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GenerateStatement {
    pub label: Identifier,
    pub scheme: GenerationScheme,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<ConcurrentStatement>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum GenerationScheme {
    For {
        param: Identifier,
        range: DiscreteRange,
    },
    If {
        condition: Expression,
    },
}

#[derive(Debug, Clone)]
pub struct ConcurrentProcedureCall {
    pub label: Option<Identifier>,
    pub postponed: bool,
    pub name: Name,
    pub args: Option<AssociationList>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConcurrentAssertStatement {
    pub label: Option<Identifier>,
    pub postponed: bool,
    pub condition: Expression,
    pub report: Option<Expression>,
    pub severity: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub label: Identifier,
    pub guard: Option<Expression>,
    pub decls: Vec<Declaration>,
    pub stmts: Vec<ConcurrentStatement>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

// ─── Sequential statements ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SequentialStatement {
    SignalAssignment(SeqSignalAssignment),
    VariableAssignment(VariableAssignment),
    If(IfStatement),
    Case(CaseStatement),
    Loop(LoopStatement),
    Next(NextStatement),
    Exit(ExitStatement),
    Return(ReturnStatement),
    Null(NullStatement),
    Wait(WaitStatement),
    Assert(AssertStatement),
    Report(ReportStatement),
    ProcedureCall(ProcedureCallStatement),
}

#[derive(Debug, Clone)]
pub struct SeqSignalAssignment {
    pub label: Option<Identifier>,
    pub target: Name,
    pub waveforms: Vec<WaveformEntry>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableAssignment {
    pub label: Option<Identifier>,
    pub target: Name,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub label: Option<Identifier>,
    pub condition: Expression,
    pub then_stmts: Vec<SequentialStatement>,
    pub elsif_branches: Vec<ElsifBranch>,
    pub else_stmts: Option<Vec<SequentialStatement>>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ElsifBranch {
    pub condition: Expression,
    pub stmts: Vec<SequentialStatement>,
}

#[derive(Debug, Clone)]
pub struct CaseStatement {
    pub label: Option<Identifier>,
    pub expression: Expression,
    pub alternatives: Vec<CaseAlternative>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseAlternative {
    pub choices: Vec<Choice>,
    pub stmts: Vec<SequentialStatement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Choice {
    Expression(Expression),
    DiscreteRange(DiscreteRange),
    Others,
}

#[derive(Debug, Clone)]
pub struct LoopStatement {
    pub label: Option<Identifier>,
    pub scheme: Option<IterationScheme>,
    pub stmts: Vec<SequentialStatement>,
    pub end_label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum IterationScheme {
    While(Expression),
    For {
        param: Identifier,
        range: DiscreteRange,
    },
}

#[derive(Debug, Clone)]
pub struct NextStatement {
    pub label: Option<Identifier>,
    pub loop_label: Option<Identifier>,
    pub condition: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExitStatement {
    pub label: Option<Identifier>,
    pub loop_label: Option<Identifier>,
    pub condition: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub label: Option<Identifier>,
    pub expression: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NullStatement {
    pub label: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WaitStatement {
    pub label: Option<Identifier>,
    pub sensitivity: Option<Vec<Name>>,
    pub condition: Option<Expression>,
    pub timeout: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssertStatement {
    pub label: Option<Identifier>,
    pub condition: Expression,
    pub report: Option<Expression>,
    pub severity: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReportStatement {
    pub label: Option<Identifier>,
    pub expression: Expression,
    pub severity: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ProcedureCallStatement {
    pub label: Option<Identifier>,
    pub name: Name,
    pub args: Option<AssociationList>,
    pub span: Span,
}

// ─── Expressions ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expression {
    /// A literal value (integer, real, string, character, bit-string, null).
    Literal(LiteralValue),
    /// A name (possibly selected/indexed/sliced/attributed).
    Name(Box<Name>),
    /// A binary operation: `lhs OP rhs`.
    Binary {
        lhs: Box<Expression>,
        op: BinaryOp,
        rhs: Box<Expression>,
        span: Span,
    },
    /// A unary operation: `OP operand`.
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },
    /// A parenthesized expression or aggregate: `( ... )`.
    Aggregate(Vec<ElementAssociation>, Span),
    /// `NEW subtype_indication` or `NEW qualified_expression`.
    Allocator(Box<Expression>, Span),
    /// `type_mark ' ( expression )` — qualified expression.
    Qualified {
        type_mark: Box<Name>,
        expr: Box<Expression>,
        span: Span,
    },
    /// `type_mark ( expression )` — type conversion.
    TypeConversion {
        type_mark: Box<Name>,
        expr: Box<Expression>,
        span: Span,
    },
    /// A function call: `name ( association_list )`.
    FunctionCall {
        name: Box<Name>,
        args: AssociationList,
        span: Span,
    },
    /// `OPEN` as an actual designator.
    Open(Span),
}

#[derive(Debug, Clone)]
pub struct ElementAssociation {
    pub choices: Option<Vec<Choice>>,
    pub expr: Expression,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(String, Span),
    Real(String, Span),
    Based(String, Span),
    Character(String, Span),
    String(String, Span),
    BitString(String, Span),
    Null(Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    // Shift (VHDL-93+)
    Sll,
    Srl,
    Sla,
    Sra,
    Rol,
    Ror,
    // Relational
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    // Matching relational (VHDL-2008)
    MatchEq,
    MatchNeq,
    MatchLt,
    MatchLte,
    MatchGt,
    MatchGte,
    // Adding
    Add,
    Sub,
    Concat,
    // Multiplying
    Mul,
    Div,
    Mod,
    Rem,
    // Miscellaneous
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos,
    Neg,
    Abs,
    Not,
    Condition, // ?? (VHDL-2008)
}

// ─── Names ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Name {
    /// A simple identifier.
    Simple(Identifier),
    /// `prefix . suffix`
    Selected(Box<Name>, Identifier),
    /// `prefix ( expression { , expression } )`  (also covers slicing if the
    /// single argument is a discrete range — disambiguation is left to later passes).
    Indexed(Box<Name>, Vec<Expression>, Span),
    /// `prefix ' attribute_designator [ ( expression ) ]`
    Attribute(Box<Name>, Identifier, Option<Box<Expression>>, Span),
    /// An operator symbol used as a name (e.g. `"+"`)
    Operator(String, Span),
    /// `prefix ( discrete_range )`
    Slice(Box<Name>, Box<DiscreteRange>, Span),
}

// ─── Identifier ──────────────────────────────────────────────────────────

/// A resolved identifier (basic or extended) with its source location.
#[derive(Debug, Clone)]
pub struct Identifier {
    pub text: String,
    pub span: Span,
}
