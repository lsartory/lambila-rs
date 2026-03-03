//! Expression-related AST nodes.

use super::common::*;
use super::literal::Literal;
use super::name::Name;
use super::type_def::{DiscreteRange, SubtypeIndication, TypeMark};

/// A VHDL expression.
///
/// EBNF (VHDL-2008): `expression ::= condition_operator primary | logical_expression`
/// EBNF (VHDL-87/93): `expression ::= relation { logical_op relation } ...`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// `condition_operator primary` (VHDL-2008: `?? primary`)
    ConditionOperator(Box<Primary>),
    /// A logical expression.
    Logical(Box<LogicalExpression>),
}

/// A logical expression — one or more relations combined by logical operators.
///
/// EBNF: `logical_expression ::= relation { AND relation } | relation { OR relation }
///       | relation { XOR relation } | relation [ NAND relation ]
///       | relation [ NOR relation ] | relation { XNOR relation }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalExpression {
    pub first: Relation,
    pub rest: Vec<(LogicalOperator, Relation)>,
}

/// A condition (aliases an expression).
///
/// EBNF (VHDL-2008): `condition ::= expression`
/// EBNF (VHDL-87/93): `condition ::= boolean_expression`
pub type Condition = Expression;

/// A relation.
///
/// EBNF (VHDL-2008): `relation ::= shift_expression [ relational_operator shift_expression ]`
/// EBNF (VHDL-87): `relation ::= simple_expression [ relational_operator simple_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub left: ShiftExpression,
    pub operator_and_right: Option<(RelationalOperator, ShiftExpression)>,
}

/// A shift expression (VHDL-93+; in VHDL-87, shift does not exist).
///
/// EBNF: `shift_expression ::= simple_expression [ shift_operator simple_expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShiftExpression {
    pub left: SimpleExpression,
    pub operator_and_right: Option<(ShiftOperator, SimpleExpression)>,
}

/// A simple expression.
///
/// EBNF: `simple_expression ::= [ sign ] term { adding_operator term }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleExpression {
    pub sign: Option<Sign>,
    pub first_term: Term,
    pub rest: Vec<(AddingOperator, Term)>,
}

/// A term.
///
/// EBNF: `term ::= factor { multiplying_operator factor }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub first: Factor,
    pub rest: Vec<(MultiplyingOperator, Factor)>,
}

/// A factor.
///
/// EBNF (VHDL-2008): `factor ::= primary [ ** primary ] | ABS primary | NOT primary
///     | logical_operator primary`
/// EBNF (VHDL-87/93): `factor ::= primary [ ** primary ] | ABS primary | NOT primary`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Factor {
    /// `primary [ ** primary ]`
    Primary {
        base: Box<Primary>,
        exponent: Option<Box<Primary>>,
    },
    /// `ABS primary`
    Abs(Box<Primary>),
    /// `NOT primary`
    Not(Box<Primary>),
    /// `logical_operator primary` (VHDL-2008)
    LogicalOp {
        operator: LogicalOperator,
        operand: Box<Primary>,
    },
}

/// A primary expression.
///
/// EBNF: `primary ::= name | literal | aggregate | function_call
///     | qualified_expression | type_conversion | allocator | ( expression )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primary {
    Name(Box<Name>),
    Literal(Literal),
    Aggregate(Aggregate),
    FunctionCall(Box<super::name::FunctionCall>),
    QualifiedExpression(Box<QualifiedExpression>),
    TypeConversion(Box<TypeConversion>),
    Allocator(Box<Allocator>),
    Parenthesized(Box<Expression>),
}

/// An aggregate.
///
/// EBNF: `aggregate ::= ( element_association { , element_association } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregate {
    pub associations: Vec<ElementAssociation>,
}

/// An element association within an aggregate.
///
/// EBNF: `element_association ::= [ choices => ] expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementAssociation {
    pub choices: Option<Choices>,
    pub expression: Box<Expression>,
}

/// A set of choices.
///
/// EBNF: `choices ::= choice { | choice }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
    pub choices: Vec<Choice>,
}

/// A single choice.
///
/// EBNF: `choice ::= simple_expression | discrete_range | element_simple_name | OTHERS`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Expression(SimpleExpression),
    DiscreteRange(DiscreteRange),
    ElementName(SimpleName),
    Others,
}

/// A qualified expression.
///
/// EBNF: `qualified_expression ::= type_mark ' ( expression ) | type_mark ' aggregate`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedExpression {
    pub type_mark: TypeMark,
    pub operand: QualifiedExpressionOperand,
}

/// The operand of a qualified expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualifiedExpressionOperand {
    Expression(Box<Expression>),
    Aggregate(Aggregate),
}

/// A type conversion.
///
/// EBNF: `type_conversion ::= type_mark ( expression )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeConversion {
    pub type_mark: TypeMark,
    pub expression: Box<Expression>,
}

/// An allocator (`new` expression).
///
/// EBNF: `allocator ::= NEW subtype_indication | NEW qualified_expression`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Allocator {
    SubtypeIndication(SubtypeIndication),
    QualifiedExpression(QualifiedExpression),
}

/// The condition operator (VHDL-2008).
///
/// EBNF: `condition_operator ::= ??`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionOperator;

/// Conditional expressions (VHDL-2008).
///
/// EBNF: `conditional_expressions ::= expression WHEN condition
///     { ELSE expression WHEN condition } [ ELSE expression ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalExpressions {
    pub alternatives: Vec<ConditionalAlternative>,
    pub else_expression: Option<Box<Expression>>,
}

/// A single conditional alternative within conditional expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalAlternative {
    pub expression: Box<Expression>,
    pub condition: Condition,
}

/// Selected expressions (VHDL-2008).
///
/// EBNF: `selected_expressions ::= { expression WHEN choices , } expression WHEN choices`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedExpressions {
    pub alternatives: Vec<SelectedExpressionAlternative>,
}

/// A single alternative within selected expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedExpressionAlternative {
    pub expression: Box<Expression>,
    pub choices: Choices,
}

// ─── Operators ──────────────────────────────────────────────────────────

/// EBNF: `adding_operator ::= + | - | &`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddingOperator {
    Plus,
    Minus,
    Concatenation,
}

/// EBNF: `multiplying_operator ::= * | / | MOD | REM`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplyingOperator {
    Multiply,
    Divide,
    Mod,
    Rem,
}

/// EBNF (VHDL-2008): `relational_operator ::= = | /= | < | <= | > | >=
///     | ?= | ?/= | ?< | ?<= | ?> | ?>=`
/// EBNF (VHDL-87/93): `relational_operator ::= = | /= | < | <= | > | >=`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalOperator {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    /// VHDL-2008 matching operators.
    MatchEq,
    MatchNeq,
    MatchLt,
    MatchLte,
    MatchGt,
    MatchGte,
}

/// EBNF: `shift_operator ::= SLL | SRL | SLA | SRA | ROL | ROR` (VHDL-93+)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftOperator {
    Sll,
    Srl,
    Sla,
    Sra,
    Rol,
    Ror,
}

/// EBNF (VHDL-2008): `logical_operator ::= AND | OR | NAND | NOR | XOR | XNOR`
/// EBNF (VHDL-87): `logical_operator ::= AND | OR | NAND | NOR | XOR`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    /// VHDL-93+.
    Xnor,
}

/// EBNF: `miscellaneous_operator ::= ** | ABS | NOT`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscellaneousOperator {
    DoubleStar,
    Abs,
    Not,
}
