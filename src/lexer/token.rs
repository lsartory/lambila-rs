/// A byte-offset span within the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Byte offset of the first character (inclusive).
    pub start: usize,
    /// Byte offset past the last character (exclusive).
    pub end: usize,
    /// 1-based line number where this span begins.
    pub line: u32,
    /// 1-based column number where this span begins.
    pub col: u32,
}

/// A single lexical token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The kind/type of this token.
    pub kind: TokenKind,
    /// The original source text of this token.
    pub text: String,
    /// Location in the source.
    pub span: Span,
}

/// A lexer error with location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    /// Human-readable error message.
    pub message: String,
    /// Location of the error in the source.
    pub span: Span,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.span.line, self.span.col, self.message)
    }
}

impl std::error::Error for LexError {}

/// The result of lexing a VHDL source file.
#[derive(Debug, Clone)]
pub struct LexResult {
    /// The tokens produced (always ends with `Eof`).
    pub tokens: Vec<Token>,
    /// Any errors encountered during lexing.
    pub errors: Vec<LexError>,
}

/// All VHDL reserved keywords across all versions.
///
/// Version-specific keywords are only emitted when lexing with the
/// appropriate version; otherwise they are returned as `Identifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordKind {
    // ── VHDL-87 keywords (81) ───────────────────────────────────────────
    Abs,
    Access,
    After,
    Alias,
    All,
    And,
    Architecture,
    Array,
    Assert,
    Attribute,
    Begin,
    Block,
    Body,
    Buffer,
    Bus,
    Case,
    Component,
    Configuration,
    Constant,
    Disconnect,
    Downto,
    Else,
    Elsif,
    End,
    Entity,
    Exit,
    File,
    For,
    Function,
    Generate,
    Generic,
    Guarded,
    If,
    In,
    Inout,
    Is,
    Label,
    Library,
    Linkage,
    Loop,
    Map,
    Mod,
    Nand,
    New,
    Next,
    Nor,
    Not,
    Null,
    Of,
    On,
    Open,
    Or,
    Others,
    Out,
    Package,
    Port,
    Procedure,
    Process,
    Range,
    Record,
    Register,
    Rem,
    Report,
    Return,
    Select,
    Severity,
    Signal,
    Subtype,
    Then,
    To,
    Transport,
    Type,
    Units,
    Until,
    Use,
    Variable,
    Wait,
    When,
    While,
    With,
    Xor,

    // ── VHDL-93 additions (+16) ─────────────────────────────────────────
    Group,
    Impure,
    Inertial,
    Literal,
    Postponed,
    Pure,
    Reject,
    Rol,
    Ror,
    Shared,
    Sla,
    Sll,
    Sra,
    Srl,
    Unaffected,
    Xnor,

    // ── VHDL-2008 additions (+19) ───────────────────────────────────────
    Assume,
    AssumeGuarantee,
    Context,
    Cover,
    Default,
    Fairness,
    Force,
    Inherit,
    Parameter,
    Property,
    Protected,
    Release,
    Restrict,
    RestrictGuarantee,
    Sequence,
    Strong,
    Vmode,
    Vprop,
    Vunit,
}

/// All possible token types in VHDL (across all versions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ── Identifiers ─────────────────────────────────────────────────────
    /// A basic identifier: `letter { [ _ ] letter_or_digit }`
    Identifier,
    /// An extended identifier: `\ graphic_character { graphic_character } \` (VHDL-93+)
    ExtendedIdentifier,

    // ── Literals ─────────────────────────────────────────────────────────
    /// Decimal integer literal, e.g. `42`, `1_000`
    IntegerLiteral,
    /// Decimal real literal, e.g. `3.14`, `1.0e-3`
    RealLiteral,
    /// Based literal, e.g. `16#FF#`, `2#1010_1100#`
    BasedLiteral,
    /// Character literal, e.g. `'A'`, `'0'`
    CharacterLiteral,
    /// String literal, e.g. `"hello"`
    StringLiteral,
    /// Bit string literal, e.g. `B"1010"`, `X"FF"`, `12UB"0000_1111"` (2008)
    BitStringLiteral,

    // ── Delimiters ───────────────────────────────────────────────────────
    Ampersand,    // &
    Tick,         // '  (attribute tick, not character literal)
    LeftParen,    // (
    RightParen,   // )
    DoubleStar,   // **
    Star,         // *
    Plus,         // +
    Comma,        // ,
    Minus,        // -
    Dot,          // .
    Slash,        // /
    Colon,        // :
    Semicolon,    // ;
    LessThan,     // <
    Equals,       // =
    GreaterThan,  // >
    Bar,          // |
    LeftBracket,  // [
    RightBracket, // ]
    Arrow,        // =>
    VarAssign,    // :=
    NotEquals,    // /=
    GtEquals,     // >=
    LtEquals,     // <=  (signal assignment or comparison – disambiguated by parser)
    Box,          // <>

    // ── VHDL-2008 delimiters ────────────────────────────────────────────
    QuestionMark,  // ?
    Condition,     // ??
    MatchEq,       // ?=
    MatchNeq,      // ?/=
    MatchLt,       // ?<
    MatchLte,      // ?<=
    MatchGt,       // ?>
    MatchGte,      // ?>=
    DoubleLess,    // <<
    DoubleGreater, // >>

    // ── Keywords ────────────────────────────────────────────────────────
    /// A reserved keyword.
    Keyword(KeywordKind),

    // ── Special ──────────────────────────────────────────────────────────
    /// End of file.
    Eof,
    /// An erroneous / unrecognized token.
    Error,
}

impl TokenKind {
    /// Returns `true` if this token kind is a keyword.
    pub fn is_keyword(self) -> bool {
        matches!(self, TokenKind::Keyword(_))
    }
}
