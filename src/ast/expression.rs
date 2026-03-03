//! Expression-related AST nodes.

use super::common::*;
use super::literal::Literal;
use super::name::Name;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{DiscreteRange, SubtypeIndication, TypeMark};
use crate::parser::{Parser, ParseError};

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

// ─── AstNode implementations ───────────────────────────────────────────

impl AstNode for Expression {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Expression::ConditionOperator(primary) => {
                write!(f, "?? ")?;
                primary.format(f, indent_level)
            }
            Expression::Logical(logical_expr) => logical_expr.format(f, indent_level),
        }
    }
}

impl AstNode for LogicalExpression {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.first.format(f, indent_level)?;
        for (op, relation) in &self.rest {
            write!(f, " ")?;
            op.format(f, indent_level)?;
            write!(f, " ")?;
            relation.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for Relation {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.left.format(f, indent_level)?;
        if let Some((op, right)) = &self.operator_and_right {
            write!(f, " ")?;
            op.format(f, indent_level)?;
            write!(f, " ")?;
            right.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ShiftExpression {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.left.format(f, indent_level)?;
        if let Some((op, right)) = &self.operator_and_right {
            write!(f, " ")?;
            op.format(f, indent_level)?;
            write!(f, " ")?;
            right.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for SimpleExpression {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(sign) = &self.sign {
            match sign {
                Sign::Plus => write!(f, "+")?,
                Sign::Minus => write!(f, "-")?,
            }
        }
        self.first_term.format(f, indent_level)?;
        for (op, term) in &self.rest {
            write!(f, " ")?;
            op.format(f, indent_level)?;
            write!(f, " ")?;
            term.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for Term {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.first.format(f, indent_level)?;
        for (op, factor) in &self.rest {
            write!(f, " ")?;
            op.format(f, indent_level)?;
            write!(f, " ")?;
            factor.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for Factor {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Factor::Primary { base, exponent } => {
                base.format(f, indent_level)?;
                if let Some(exp) = exponent {
                    write!(f, " ** ")?;
                    exp.format(f, indent_level)?;
                }
                Ok(())
            }
            Factor::Abs(primary) => {
                write!(f, "abs ")?;
                primary.format(f, indent_level)
            }
            Factor::Not(primary) => {
                write!(f, "not ")?;
                primary.format(f, indent_level)
            }
            Factor::LogicalOp { operator, operand } => {
                operator.format(f, indent_level)?;
                write!(f, " ")?;
                operand.format(f, indent_level)
            }
        }
    }
}

impl AstNode for Primary {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Primary::Name(name) => name.format(f, indent_level),
            Primary::Literal(lit) => lit.format(f, indent_level),
            Primary::Aggregate(agg) => agg.format(f, indent_level),
            Primary::FunctionCall(fc) => fc.format(f, indent_level),
            Primary::QualifiedExpression(qe) => qe.format(f, indent_level),
            Primary::TypeConversion(tc) => tc.format(f, indent_level),
            Primary::Allocator(alloc) => alloc.format(f, indent_level),
            Primary::Parenthesized(expr) => {
                write!(f, "(")?;
                expr.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}

impl AstNode for Aggregate {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.associations, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for ElementAssociation {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(choices) = &self.choices {
            choices.format(f, indent_level)?;
            write!(f, " => ")?;
        }
        self.expression.format(f, indent_level)
    }
}

impl AstNode for Choices {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, choice) in self.choices.iter().enumerate() {
            if i > 0 {
                write!(f, " | ")?;
            }
            choice.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for Choice {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Choice::Expression(simple_expr) => simple_expr.format(f, indent_level),
            Choice::DiscreteRange(dr) => dr.format(f, indent_level),
            Choice::ElementName(name) => name.format(f, indent_level),
            Choice::Others => write!(f, "others"),
        }
    }
}

impl AstNode for QualifiedExpression {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.type_mark.format(f, indent_level)?;
        write!(f, "'")?;
        match &self.operand {
            QualifiedExpressionOperand::Expression(expr) => {
                write!(f, "(")?;
                expr.format(f, indent_level)?;
                write!(f, ")")
            }
            QualifiedExpressionOperand::Aggregate(agg) => agg.format(f, indent_level),
        }
    }
}

impl AstNode for QualifiedExpressionOperand {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            QualifiedExpressionOperand::Expression(expr) => expr.format(f, indent_level),
            QualifiedExpressionOperand::Aggregate(agg) => agg.format(f, indent_level),
        }
    }
}

impl AstNode for TypeConversion {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.type_mark.format(f, indent_level)?;
        write!(f, "(")?;
        self.expression.format(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for Allocator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Allocator::SubtypeIndication(si) => {
                write!(f, "new ")?;
                si.format(f, indent_level)
            }
            Allocator::QualifiedExpression(qe) => {
                write!(f, "new ")?;
                qe.format(f, indent_level)
            }
        }
    }
}

impl AstNode for ConditionOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "??")
    }
}

impl AstNode for ConditionalExpressions {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, alt) in self.alternatives.iter().enumerate() {
            if i > 0 {
                write!(f, " else ")?;
            }
            alt.format(f, indent_level)?;
        }
        if let Some(else_expr) = &self.else_expression {
            write!(f, " else ")?;
            else_expr.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for ConditionalAlternative {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)?;
        write!(f, " when ")?;
        self.condition.format(f, indent_level)
    }
}

impl AstNode for SelectedExpressions {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for (i, alt) in self.alternatives.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            alt.format(f, indent_level)?;
        }
        Ok(())
    }
}

impl AstNode for SelectedExpressionAlternative {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)?;
        write!(f, " when ")?;
        self.choices.format(f, indent_level)
    }
}

impl AstNode for AddingOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            AddingOperator::Plus => write!(f, "+"),
            AddingOperator::Minus => write!(f, "-"),
            AddingOperator::Concatenation => write!(f, "&"),
        }
    }
}

impl AstNode for MultiplyingOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            MultiplyingOperator::Multiply => write!(f, "*"),
            MultiplyingOperator::Divide => write!(f, "/"),
            MultiplyingOperator::Mod => write!(f, "mod"),
            MultiplyingOperator::Rem => write!(f, "rem"),
        }
    }
}

impl AstNode for RelationalOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            RelationalOperator::Eq => write!(f, "="),
            RelationalOperator::Neq => write!(f, "/="),
            RelationalOperator::Lt => write!(f, "<"),
            RelationalOperator::Lte => write!(f, "<="),
            RelationalOperator::Gt => write!(f, ">"),
            RelationalOperator::Gte => write!(f, ">="),
            RelationalOperator::MatchEq => write!(f, "?="),
            RelationalOperator::MatchNeq => write!(f, "?/="),
            RelationalOperator::MatchLt => write!(f, "?<"),
            RelationalOperator::MatchLte => write!(f, "?<="),
            RelationalOperator::MatchGt => write!(f, "?>"),
            RelationalOperator::MatchGte => write!(f, "?>="),
        }
    }
}

impl AstNode for ShiftOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            ShiftOperator::Sll => write!(f, "sll"),
            ShiftOperator::Srl => write!(f, "srl"),
            ShiftOperator::Sla => write!(f, "sla"),
            ShiftOperator::Sra => write!(f, "sra"),
            ShiftOperator::Rol => write!(f, "rol"),
            ShiftOperator::Ror => write!(f, "ror"),
        }
    }
}

impl AstNode for LogicalOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "and"),
            LogicalOperator::Or => write!(f, "or"),
            LogicalOperator::Nand => write!(f, "nand"),
            LogicalOperator::Nor => write!(f, "nor"),
            LogicalOperator::Xor => write!(f, "xor"),
            LogicalOperator::Xnor => write!(f, "xnor"),
        }
    }
}

impl AstNode for MiscellaneousOperator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            MiscellaneousOperator::DoubleStar => write!(f, "**"),
            MiscellaneousOperator::Abs => write!(f, "abs"),
            MiscellaneousOperator::Not => write!(f, "not"),
        }
    }
}
