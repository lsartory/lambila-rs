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
pub struct LexerError {
    /// Human-readable error message.
    pub message: String,
    /// Location of the error in the source.
    pub span: Span,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.span.line, self.span.col, self.message)
    }
}

impl std::error::Error for LexerError {}

/// The result of lexing a VHDL source file.
#[derive(Debug, Clone)]
pub struct LexResult {
    /// The tokens produced (always ends with `Eof`).
    pub tokens: Vec<Token>,
    /// Any errors encountered during lexing.
    pub errors: Vec<LexerError>,
}

/// All possible token types in VHDL (across all versions).
///
/// Keyword variants are prefixed with `Kw_`. Version-specific keywords
/// are only emitted when lexing with the appropriate version; otherwise
/// they are returned as `Identifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
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

    // ── Keywords (all versions) ──────────────────────────────────────
    // VHDL-87 (81 keywords)
    Kw_Abs,
    Kw_Access,
    Kw_After,
    Kw_Alias,
    Kw_All,
    Kw_And,
    Kw_Architecture,
    Kw_Array,
    Kw_Assert,
    Kw_Attribute,
    Kw_Begin,
    Kw_Block,
    Kw_Body,
    Kw_Buffer,
    Kw_Bus,
    Kw_Case,
    Kw_Component,
    Kw_Configuration,
    Kw_Constant,
    Kw_Disconnect,
    Kw_Downto,
    Kw_Else,
    Kw_Elsif,
    Kw_End,
    Kw_Entity,
    Kw_Exit,
    Kw_File,
    Kw_For,
    Kw_Function,
    Kw_Generate,
    Kw_Generic,
    Kw_Guarded,
    Kw_If,
    Kw_In,
    Kw_Inout,
    Kw_Is,
    Kw_Label,
    Kw_Library,
    Kw_Linkage,
    Kw_Loop,
    Kw_Map,
    Kw_Mod,
    Kw_Nand,
    Kw_New,
    Kw_Next,
    Kw_Nor,
    Kw_Not,
    Kw_Null,
    Kw_Of,
    Kw_On,
    Kw_Open,
    Kw_Or,
    Kw_Others,
    Kw_Out,
    Kw_Package,
    Kw_Port,
    Kw_Procedure,
    Kw_Process,
    Kw_Range,
    Kw_Record,
    Kw_Register,
    Kw_Rem,
    Kw_Report,
    Kw_Return,
    Kw_Select,
    Kw_Severity,
    Kw_Signal,
    Kw_Subtype,
    Kw_Then,
    Kw_To,
    Kw_Transport,
    Kw_Type,
    Kw_Units,
    Kw_Until,
    Kw_Use,
    Kw_Variable,
    Kw_Wait,
    Kw_When,
    Kw_While,
    Kw_With,
    Kw_Xor,

    // VHDL-93 additions (+16)
    Kw_Group,
    Kw_Impure,
    Kw_Inertial,
    Kw_Literal,
    Kw_Postponed,
    Kw_Pure,
    Kw_Reject,
    Kw_Rol,
    Kw_Ror,
    Kw_Shared,
    Kw_Sla,
    Kw_Sll,
    Kw_Sra,
    Kw_Srl,
    Kw_Unaffected,
    Kw_Xnor,

    // VHDL-2008 additions (+19)
    Kw_Assume,
    Kw_AssumeGuarantee,
    Kw_Context,
    Kw_Cover,
    Kw_Default,
    Kw_Fairness,
    Kw_Force,
    Kw_Inherit,
    Kw_Parameter,
    Kw_Property,
    Kw_Protected,
    Kw_Release,
    Kw_Restrict,
    Kw_RestrictGuarantee,
    Kw_Sequence,
    Kw_Strong,
    Kw_Vmode,
    Kw_Vprop,
    Kw_Vunit,

    // ── Special ──────────────────────────────────────────────────────────
    /// End of file.
    Eof,
    /// An erroneous / unrecognized token.
    Error,
}

impl TokenKind {
    /// Returns `true` if this token kind is a keyword.
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            TokenKind::Kw_Abs
                | TokenKind::Kw_Access
                | TokenKind::Kw_After
                | TokenKind::Kw_Alias
                | TokenKind::Kw_All
                | TokenKind::Kw_And
                | TokenKind::Kw_Architecture
                | TokenKind::Kw_Array
                | TokenKind::Kw_Assert
                | TokenKind::Kw_Attribute
                | TokenKind::Kw_Begin
                | TokenKind::Kw_Block
                | TokenKind::Kw_Body
                | TokenKind::Kw_Buffer
                | TokenKind::Kw_Bus
                | TokenKind::Kw_Case
                | TokenKind::Kw_Component
                | TokenKind::Kw_Configuration
                | TokenKind::Kw_Constant
                | TokenKind::Kw_Disconnect
                | TokenKind::Kw_Downto
                | TokenKind::Kw_Else
                | TokenKind::Kw_Elsif
                | TokenKind::Kw_End
                | TokenKind::Kw_Entity
                | TokenKind::Kw_Exit
                | TokenKind::Kw_File
                | TokenKind::Kw_For
                | TokenKind::Kw_Function
                | TokenKind::Kw_Generate
                | TokenKind::Kw_Generic
                | TokenKind::Kw_Guarded
                | TokenKind::Kw_If
                | TokenKind::Kw_In
                | TokenKind::Kw_Inout
                | TokenKind::Kw_Is
                | TokenKind::Kw_Label
                | TokenKind::Kw_Library
                | TokenKind::Kw_Linkage
                | TokenKind::Kw_Loop
                | TokenKind::Kw_Map
                | TokenKind::Kw_Mod
                | TokenKind::Kw_Nand
                | TokenKind::Kw_New
                | TokenKind::Kw_Next
                | TokenKind::Kw_Nor
                | TokenKind::Kw_Not
                | TokenKind::Kw_Null
                | TokenKind::Kw_Of
                | TokenKind::Kw_On
                | TokenKind::Kw_Open
                | TokenKind::Kw_Or
                | TokenKind::Kw_Others
                | TokenKind::Kw_Out
                | TokenKind::Kw_Package
                | TokenKind::Kw_Port
                | TokenKind::Kw_Procedure
                | TokenKind::Kw_Process
                | TokenKind::Kw_Range
                | TokenKind::Kw_Record
                | TokenKind::Kw_Register
                | TokenKind::Kw_Rem
                | TokenKind::Kw_Report
                | TokenKind::Kw_Return
                | TokenKind::Kw_Select
                | TokenKind::Kw_Severity
                | TokenKind::Kw_Signal
                | TokenKind::Kw_Subtype
                | TokenKind::Kw_Then
                | TokenKind::Kw_To
                | TokenKind::Kw_Transport
                | TokenKind::Kw_Type
                | TokenKind::Kw_Units
                | TokenKind::Kw_Until
                | TokenKind::Kw_Use
                | TokenKind::Kw_Variable
                | TokenKind::Kw_Wait
                | TokenKind::Kw_When
                | TokenKind::Kw_While
                | TokenKind::Kw_With
                | TokenKind::Kw_Xor
                | TokenKind::Kw_Group
                | TokenKind::Kw_Impure
                | TokenKind::Kw_Inertial
                | TokenKind::Kw_Literal
                | TokenKind::Kw_Postponed
                | TokenKind::Kw_Pure
                | TokenKind::Kw_Reject
                | TokenKind::Kw_Rol
                | TokenKind::Kw_Ror
                | TokenKind::Kw_Shared
                | TokenKind::Kw_Sla
                | TokenKind::Kw_Sll
                | TokenKind::Kw_Sra
                | TokenKind::Kw_Srl
                | TokenKind::Kw_Unaffected
                | TokenKind::Kw_Xnor
                | TokenKind::Kw_Assume
                | TokenKind::Kw_AssumeGuarantee
                | TokenKind::Kw_Context
                | TokenKind::Kw_Cover
                | TokenKind::Kw_Default
                | TokenKind::Kw_Fairness
                | TokenKind::Kw_Force
                | TokenKind::Kw_Inherit
                | TokenKind::Kw_Parameter
                | TokenKind::Kw_Property
                | TokenKind::Kw_Protected
                | TokenKind::Kw_Release
                | TokenKind::Kw_Restrict
                | TokenKind::Kw_RestrictGuarantee
                | TokenKind::Kw_Sequence
                | TokenKind::Kw_Strong
                | TokenKind::Kw_Vmode
                | TokenKind::Kw_Vprop
                | TokenKind::Kw_Vunit
        )
    }
}
