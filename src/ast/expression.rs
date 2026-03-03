//! Expression-related AST nodes.

use super::common::*;
use super::literal::Literal;
use super::name::Name;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{DiscreteRange, SubtypeIndication, TypeMark};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

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

// ─── Helper functions ───────────────────────────────────────────────────

/// Try to parse a logical operator from the current token.
fn try_parse_logical_operator(parser: &mut Parser) -> Option<LogicalOperator> {
    match parser.peek_kind() {
        Some(TokenKind::Keyword(KeywordKind::And)) => {
            parser.consume();
            Some(LogicalOperator::And)
        }
        Some(TokenKind::Keyword(KeywordKind::Or)) => {
            parser.consume();
            Some(LogicalOperator::Or)
        }
        Some(TokenKind::Keyword(KeywordKind::Nand)) => {
            parser.consume();
            Some(LogicalOperator::Nand)
        }
        Some(TokenKind::Keyword(KeywordKind::Nor)) => {
            parser.consume();
            Some(LogicalOperator::Nor)
        }
        Some(TokenKind::Keyword(KeywordKind::Xor)) => {
            parser.consume();
            Some(LogicalOperator::Xor)
        }
        Some(TokenKind::Keyword(KeywordKind::Xnor)) => {
            parser.consume();
            Some(LogicalOperator::Xnor)
        }
        _ => None,
    }
}

/// Try to parse a relational operator from the current token.
fn try_parse_relational_operator(parser: &mut Parser) -> Option<RelationalOperator> {
    match parser.peek_kind() {
        Some(TokenKind::Equals) => {
            parser.consume();
            Some(RelationalOperator::Eq)
        }
        Some(TokenKind::NotEquals) => {
            parser.consume();
            Some(RelationalOperator::Neq)
        }
        Some(TokenKind::LessThan) => {
            parser.consume();
            Some(RelationalOperator::Lt)
        }
        Some(TokenKind::LtEquals) => {
            parser.consume();
            Some(RelationalOperator::Lte)
        }
        Some(TokenKind::GreaterThan) => {
            parser.consume();
            Some(RelationalOperator::Gt)
        }
        Some(TokenKind::GtEquals) => {
            parser.consume();
            Some(RelationalOperator::Gte)
        }
        Some(TokenKind::MatchEq) => {
            parser.consume();
            Some(RelationalOperator::MatchEq)
        }
        Some(TokenKind::MatchNeq) => {
            parser.consume();
            Some(RelationalOperator::MatchNeq)
        }
        Some(TokenKind::MatchLt) => {
            parser.consume();
            Some(RelationalOperator::MatchLt)
        }
        Some(TokenKind::MatchLte) => {
            parser.consume();
            Some(RelationalOperator::MatchLte)
        }
        Some(TokenKind::MatchGt) => {
            parser.consume();
            Some(RelationalOperator::MatchGt)
        }
        Some(TokenKind::MatchGte) => {
            parser.consume();
            Some(RelationalOperator::MatchGte)
        }
        _ => None,
    }
}

/// Try to parse a shift operator from the current token.
fn try_parse_shift_operator(parser: &mut Parser) -> Option<ShiftOperator> {
    match parser.peek_kind() {
        Some(TokenKind::Keyword(KeywordKind::Sll)) => {
            parser.consume();
            Some(ShiftOperator::Sll)
        }
        Some(TokenKind::Keyword(KeywordKind::Srl)) => {
            parser.consume();
            Some(ShiftOperator::Srl)
        }
        Some(TokenKind::Keyword(KeywordKind::Sla)) => {
            parser.consume();
            Some(ShiftOperator::Sla)
        }
        Some(TokenKind::Keyword(KeywordKind::Sra)) => {
            parser.consume();
            Some(ShiftOperator::Sra)
        }
        Some(TokenKind::Keyword(KeywordKind::Rol)) => {
            parser.consume();
            Some(ShiftOperator::Rol)
        }
        Some(TokenKind::Keyword(KeywordKind::Ror)) => {
            parser.consume();
            Some(ShiftOperator::Ror)
        }
        _ => None,
    }
}

/// Try to parse an adding operator from the current token.
fn try_parse_adding_operator(parser: &mut Parser) -> Option<AddingOperator> {
    match parser.peek_kind() {
        Some(TokenKind::Plus) => {
            parser.consume();
            Some(AddingOperator::Plus)
        }
        Some(TokenKind::Minus) => {
            parser.consume();
            Some(AddingOperator::Minus)
        }
        Some(TokenKind::Ampersand) => {
            parser.consume();
            Some(AddingOperator::Concatenation)
        }
        _ => None,
    }
}

/// Try to parse a multiplying operator from the current token.
fn try_parse_multiplying_operator(parser: &mut Parser) -> Option<MultiplyingOperator> {
    match parser.peek_kind() {
        Some(TokenKind::Star) => {
            parser.consume();
            Some(MultiplyingOperator::Multiply)
        }
        Some(TokenKind::Slash) => {
            parser.consume();
            Some(MultiplyingOperator::Divide)
        }
        Some(TokenKind::Keyword(KeywordKind::Mod)) => {
            parser.consume();
            Some(MultiplyingOperator::Mod)
        }
        Some(TokenKind::Keyword(KeywordKind::Rem)) => {
            parser.consume();
            Some(MultiplyingOperator::Rem)
        }
        _ => None,
    }
}

/// Check (without consuming) whether the current token is a logical operator keyword.
fn peek_is_logical_operator(parser: &Parser) -> bool {
    matches!(
        parser.peek_kind(),
        Some(TokenKind::Keyword(KeywordKind::And))
            | Some(TokenKind::Keyword(KeywordKind::Or))
            | Some(TokenKind::Keyword(KeywordKind::Nand))
            | Some(TokenKind::Keyword(KeywordKind::Nor))
            | Some(TokenKind::Keyword(KeywordKind::Xor))
            | Some(TokenKind::Keyword(KeywordKind::Xnor))
    )
}

// ─── AstNode implementations ───────────────────────────────────────────

impl AstNode for Expression {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // VHDL-2008: condition_operator primary => ?? primary
        if parser.at(TokenKind::Condition) {
            parser.consume();
            let primary = Primary::parse(parser)?;
            return Ok(Expression::ConditionOperator(Box::new(primary)));
        }
        let logical = LogicalExpression::parse(parser)?;
        Ok(Expression::Logical(Box::new(logical)))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let first = Relation::parse(parser)?;
        let mut rest = Vec::new();
        while peek_is_logical_operator(parser) {
            let op = try_parse_logical_operator(parser).unwrap();
            let relation = Relation::parse(parser)?;
            rest.push((op, relation));
        }
        Ok(LogicalExpression { first, rest })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let left = ShiftExpression::parse(parser)?;
        let save = parser.save();
        let operator_and_right = if let Some(op) = try_parse_relational_operator(parser) {
            match ShiftExpression::parse(parser) {
                Ok(right) => Some((op, right)),
                Err(_) => {
                    parser.restore(save);
                    None
                }
            }
        } else {
            None
        };
        Ok(Relation {
            left,
            operator_and_right,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let left = SimpleExpression::parse(parser)?;
        let save = parser.save();
        let operator_and_right = if let Some(op) = try_parse_shift_operator(parser) {
            match SimpleExpression::parse(parser) {
                Ok(right) => Some((op, right)),
                Err(_) => {
                    parser.restore(save);
                    None
                }
            }
        } else {
            None
        };
        Ok(ShiftExpression {
            left,
            operator_and_right,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Optional leading sign
        let sign = match parser.peek_kind() {
            Some(TokenKind::Plus) => {
                parser.consume();
                Some(Sign::Plus)
            }
            Some(TokenKind::Minus) => {
                parser.consume();
                Some(Sign::Minus)
            }
            _ => None,
        };
        let first_term = Term::parse(parser)?;
        let mut rest = Vec::new();
        while let Some(op) = try_parse_adding_operator(parser) {
            let term = Term::parse(parser)?;
            rest.push((op, term));
        }
        Ok(SimpleExpression {
            sign,
            first_term,
            rest,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let first = Factor::parse(parser)?;
        let mut rest = Vec::new();
        while let Some(op) = try_parse_multiplying_operator(parser) {
            let factor = Factor::parse(parser)?;
            rest.push((op, factor));
        }
        Ok(Term { first, rest })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Abs)) => {
                parser.consume();
                let primary = Primary::parse(parser)?;
                Ok(Factor::Abs(Box::new(primary)))
            }
            Some(TokenKind::Keyword(KeywordKind::Not)) => {
                parser.consume();
                let primary = Primary::parse(parser)?;
                Ok(Factor::Not(Box::new(primary)))
            }
            // VHDL-2008: logical_operator primary (reduction operators at factor level)
            Some(TokenKind::Keyword(KeywordKind::And))
            | Some(TokenKind::Keyword(KeywordKind::Or))
            | Some(TokenKind::Keyword(KeywordKind::Nand))
            | Some(TokenKind::Keyword(KeywordKind::Nor))
            | Some(TokenKind::Keyword(KeywordKind::Xor))
            | Some(TokenKind::Keyword(KeywordKind::Xnor)) => {
                let op = try_parse_logical_operator(parser).unwrap();
                let primary = Primary::parse(parser)?;
                Ok(Factor::LogicalOp {
                    operator: op,
                    operand: Box::new(primary),
                })
            }
            _ => {
                let base = Primary::parse(parser)?;
                let exponent = if parser.at(TokenKind::DoubleStar) {
                    parser.consume();
                    let exp = Primary::parse(parser)?;
                    Some(Box::new(exp))
                } else {
                    None
                };
                Ok(Factor::Primary {
                    base: Box::new(base),
                    exponent,
                })
            }
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            // Parenthesized expression or aggregate: ( ... )
            Some(TokenKind::LeftParen) => {
                // Parse as aggregate (which covers single-expression parenthesized case too)
                let agg = Aggregate::parse(parser)?;
                // If it's a single element with no choices, treat as parenthesized expression
                if agg.associations.len() == 1 && agg.associations[0].choices.is_none() {
                    Ok(Primary::Parenthesized(
                        agg.associations.into_iter().next().unwrap().expression,
                    ))
                } else {
                    Ok(Primary::Aggregate(agg))
                }
            }
            // NEW keyword -> Allocator
            Some(TokenKind::Keyword(KeywordKind::New)) => {
                let alloc = Allocator::parse(parser)?;
                Ok(Primary::Allocator(Box::new(alloc)))
            }
            // Literal tokens
            Some(TokenKind::IntegerLiteral)
            | Some(TokenKind::RealLiteral)
            | Some(TokenKind::BasedLiteral)
            | Some(TokenKind::BitStringLiteral) => {
                let lit = Literal::parse(parser)?;
                Ok(Primary::Literal(lit))
            }
            Some(TokenKind::StringLiteral) => {
                // Could be operator_symbol (part of a name) or string literal.
                // Parse as name (which handles operator_symbol) and let the
                // name parser take care of it.
                let name = Name::parse(parser)?;
                Ok(Primary::Name(Box::new(name)))
            }
            Some(TokenKind::CharacterLiteral) => {
                // Character literal: could be part of a name (Name::CharacterLiteral)
                // or an enumeration literal. Parse as Name.
                let name = Name::parse(parser)?;
                Ok(Primary::Name(Box::new(name)))
            }
            Some(TokenKind::Keyword(KeywordKind::Null)) => {
                parser.consume();
                Ok(Primary::Literal(Literal::Null))
            }
            // Identifier or ExtendedIdentifier -> parse as Name, then check for
            // qualified expression (name'(...)) or type conversion (name(...))
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let name = Name::parse(parser)?;
                // After parsing the name, check if it's followed by tick+paren for
                // qualified expression. The Name parser handles tick for attributes,
                // but qualified expression has tick followed by paren which the Name
                // parser won't consume (since attribute designator expects an identifier).
                if parser.at(TokenKind::Tick) {
                    // Check if the token after tick is LeftParen
                    if let Some(next) = parser.peek_nth(1)
                        && next.kind == TokenKind::LeftParen
                    {
                        // This is a qualified expression: type_mark ' ( expression )
                        // or type_mark ' aggregate
                        parser.consume(); // consume tick
                        let type_mark = TypeMark::TypeName(Box::new(name));
                        let operand = QualifiedExpressionOperand::parse(parser)?;
                        return Ok(Primary::QualifiedExpression(Box::new(
                            QualifiedExpression { type_mark, operand },
                        )));
                    }
                }
                Ok(Primary::Name(Box::new(name)))
            }
            // External name <<...>>
            Some(TokenKind::DoubleLess) => {
                let name = Name::parse(parser)?;
                Ok(Primary::Name(Box::new(name)))
            }
            _ => Err(parser.error("expected primary expression")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::LeftParen)?;
        let mut associations = Vec::new();
        // Handle empty parentheses (shouldn't normally occur but be defensive)
        if !parser.at(TokenKind::RightParen) {
            associations.push(ElementAssociation::parse(parser)?);
            while parser.consume_if(TokenKind::Comma).is_some() {
                associations.push(ElementAssociation::parse(parser)?);
            }
        }
        parser.expect(TokenKind::RightParen)?;
        Ok(Aggregate { associations })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "(")?;
        format_comma_separated(&self.associations, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for ElementAssociation {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Try to parse choices => expression first.
        // Save position, try parsing choices, and look for =>
        let save = parser.save();
        match Choices::parse(parser) {
            Ok(choices) => {
                if parser.consume_if(TokenKind::Arrow).is_some() {
                    // Found choices => expression
                    let expression = Expression::parse(parser)?;
                    Ok(ElementAssociation {
                        choices: Some(choices),
                        expression: Box::new(expression),
                    })
                } else {
                    // No =>, backtrack and parse as just expression
                    parser.restore(save);
                    let expression = Expression::parse(parser)?;
                    Ok(ElementAssociation {
                        choices: None,
                        expression: Box::new(expression),
                    })
                }
            }
            Err(_) => {
                parser.restore(save);
                let expression = Expression::parse(parser)?;
                Ok(ElementAssociation {
                    choices: None,
                    expression: Box::new(expression),
                })
            }
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut choices = Vec::new();
        choices.push(Choice::parse(parser)?);
        while parser.consume_if(TokenKind::Bar).is_some() {
            choices.push(Choice::parse(parser)?);
        }
        Ok(Choices { choices })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // OTHERS
        if parser.at_keyword(KeywordKind::Others) {
            parser.consume();
            return Ok(Choice::Others);
        }

        // Try parsing a simple_expression, then check for TO/DOWNTO (discrete_range).
        // A discrete_range could be: simple_expression direction simple_expression
        // or a subtype_indication (type_mark [range constraint]).
        // For simplicity, parse a simple_expression first, then check for direction.
        let save = parser.save();
        let simple_expr = SimpleExpression::parse(parser)?;

        // Check for direction (TO or DOWNTO) -> discrete range
        if parser.at_keyword(KeywordKind::To) {
            parser.consume();
            let right = SimpleExpression::parse(parser)?;
            return Ok(Choice::DiscreteRange(DiscreteRange::Range(
                super::type_def::Range::Explicit {
                    left: simple_expr,
                    direction: Direction::To,
                    right,
                },
            )));
        }
        if parser.at_keyword(KeywordKind::Downto) {
            parser.consume();
            let right = SimpleExpression::parse(parser)?;
            return Ok(Choice::DiscreteRange(DiscreteRange::Range(
                super::type_def::Range::Explicit {
                    left: simple_expr,
                    direction: Direction::Downto,
                    right,
                },
            )));
        }

        // Check if there's a RANGE keyword following (subtype_indication style discrete range)
        if parser.at_keyword(KeywordKind::Range) {
            // This is a type_mark followed by range constraint.
            // The simple_expression we parsed was actually the type_mark.
            // We need to convert it back. For now, restore and try parsing as
            // subtype indication.
            parser.restore(save);
            let name = Name::parse(parser)?;
            let type_mark = TypeMark::TypeName(Box::new(name));
            parser.expect_keyword(KeywordKind::Range)?;
            let left = SimpleExpression::parse(parser)?;
            if parser.at_keyword(KeywordKind::To) {
                parser.consume();
                let right = SimpleExpression::parse(parser)?;
                return Ok(Choice::DiscreteRange(DiscreteRange::SubtypeIndication(
                    SubtypeIndication {
                        resolution: None,
                        type_mark,
                        constraint: Some(super::type_def::Constraint::Range(
                            super::type_def::RangeConstraint {
                                range: super::type_def::Range::Explicit {
                                    left,
                                    direction: Direction::To,
                                    right,
                                },
                            },
                        )),
                    },
                )));
            } else if parser.at_keyword(KeywordKind::Downto) {
                parser.consume();
                let right = SimpleExpression::parse(parser)?;
                return Ok(Choice::DiscreteRange(DiscreteRange::SubtypeIndication(
                    SubtypeIndication {
                        resolution: None,
                        type_mark,
                        constraint: Some(super::type_def::Constraint::Range(
                            super::type_def::RangeConstraint {
                                range: super::type_def::Range::Explicit {
                                    left,
                                    direction: Direction::Downto,
                                    right,
                                },
                            },
                        )),
                    },
                )));
            } else {
                return Err(parser.error("expected 'to' or 'downto' in range constraint"));
            }
        }

        // Otherwise it's just a simple expression choice
        Ok(Choice::Expression(simple_expr))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // type_mark ' ( expression ) | type_mark ' aggregate
        // Parse the type_mark as a name
        let name = Name::parse(parser)?;
        let type_mark = TypeMark::TypeName(Box::new(name));
        parser.expect(TokenKind::Tick)?;
        let operand = QualifiedExpressionOperand::parse(parser)?;
        Ok(QualifiedExpression { type_mark, operand })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // After the tick, we expect '(' ... ')'
        // This is either a parenthesized expression or an aggregate.
        // Parse as aggregate and then decide.
        let agg = Aggregate::parse(parser)?;
        // If single element with no choices, treat as expression
        if agg.associations.len() == 1 && agg.associations[0].choices.is_none() {
            Ok(QualifiedExpressionOperand::Expression(
                agg.associations.into_iter().next().unwrap().expression,
            ))
        } else {
            Ok(QualifiedExpressionOperand::Aggregate(agg))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            QualifiedExpressionOperand::Expression(expr) => expr.format(f, indent_level),
            QualifiedExpressionOperand::Aggregate(agg) => agg.format(f, indent_level),
        }
    }
}

impl AstNode for TypeConversion {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // type_mark ( expression )
        let name = Name::parse(parser)?;
        let type_mark = TypeMark::TypeName(Box::new(name));
        parser.expect(TokenKind::LeftParen)?;
        let expression = Expression::parse(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Ok(TypeConversion {
            type_mark,
            expression: Box::new(expression),
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.type_mark.format(f, indent_level)?;
        write!(f, "(")?;
        self.expression.format(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for Allocator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // NEW subtype_indication | NEW qualified_expression
        parser.expect_keyword(KeywordKind::New)?;

        // Try qualified expression first: name ' ( ... )
        let save = parser.save();
        if let Ok(name) = Name::parse(parser)
            && parser.at(TokenKind::Tick)
            && let Some(next) = parser.peek_nth(1)
            && next.kind == TokenKind::LeftParen
        {
            parser.consume(); // consume tick
            let type_mark = TypeMark::TypeName(Box::new(name));
            let operand = QualifiedExpressionOperand::parse(parser)?;
            return Ok(Allocator::QualifiedExpression(QualifiedExpression {
                type_mark,
                operand,
            }));
        }

        // Otherwise parse as subtype_indication
        parser.restore(save);
        let si = SubtypeIndication::parse(parser)?;
        Ok(Allocator::SubtypeIndication(si))
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::Condition)?;
        Ok(ConditionOperator)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        write!(f, "??")
    }
}

impl AstNode for ConditionalExpressions {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // expression WHEN condition { ELSE expression WHEN condition } [ ELSE expression ]
        let mut alternatives = Vec::new();

        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::When)?;
        let condition = Expression::parse(parser)?;
        alternatives.push(ConditionalAlternative {
            expression: Box::new(expression),
            condition,
        });

        let mut else_expression = None;
        while parser.consume_if_keyword(KeywordKind::Else).is_some() {
            let expr = Expression::parse(parser)?;
            if parser.consume_if_keyword(KeywordKind::When).is_some() {
                let cond = Expression::parse(parser)?;
                alternatives.push(ConditionalAlternative {
                    expression: Box::new(expr),
                    condition: cond,
                });
            } else {
                // Final else expression (no WHEN)
                else_expression = Some(Box::new(expr));
                break;
            }
        }

        Ok(ConditionalExpressions {
            alternatives,
            else_expression,
        })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::When)?;
        let condition = Expression::parse(parser)?;
        Ok(ConditionalAlternative {
            expression: Box::new(expression),
            condition,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)?;
        write!(f, " when ")?;
        self.condition.format(f, indent_level)
    }
}

impl AstNode for SelectedExpressions {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { expression WHEN choices , } expression WHEN choices
        let mut alternatives = Vec::new();

        loop {
            let expression = Expression::parse(parser)?;
            parser.expect_keyword(KeywordKind::When)?;
            let choices = Choices::parse(parser)?;
            alternatives.push(SelectedExpressionAlternative {
                expression: Box::new(expression),
                choices,
            });
            if parser.consume_if(TokenKind::Comma).is_none() {
                break;
            }
        }

        Ok(SelectedExpressions { alternatives })
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let expression = Expression::parse(parser)?;
        parser.expect_keyword(KeywordKind::When)?;
        let choices = Choices::parse(parser)?;
        Ok(SelectedExpressionAlternative {
            expression: Box::new(expression),
            choices,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.expression.format(f, indent_level)?;
        write!(f, " when ")?;
        self.choices.format(f, indent_level)
    }
}

impl AstNode for AddingOperator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Plus) => {
                parser.consume();
                Ok(AddingOperator::Plus)
            }
            Some(TokenKind::Minus) => {
                parser.consume();
                Ok(AddingOperator::Minus)
            }
            Some(TokenKind::Ampersand) => {
                parser.consume();
                Ok(AddingOperator::Concatenation)
            }
            _ => Err(parser.error("expected adding operator (+, -, or &)")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Star) => {
                parser.consume();
                Ok(MultiplyingOperator::Multiply)
            }
            Some(TokenKind::Slash) => {
                parser.consume();
                Ok(MultiplyingOperator::Divide)
            }
            Some(TokenKind::Keyword(KeywordKind::Mod)) => {
                parser.consume();
                Ok(MultiplyingOperator::Mod)
            }
            Some(TokenKind::Keyword(KeywordKind::Rem)) => {
                parser.consume();
                Ok(MultiplyingOperator::Rem)
            }
            _ => Err(parser.error("expected multiplying operator (*, /, mod, or rem)")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Equals) => {
                parser.consume();
                Ok(RelationalOperator::Eq)
            }
            Some(TokenKind::NotEquals) => {
                parser.consume();
                Ok(RelationalOperator::Neq)
            }
            Some(TokenKind::LessThan) => {
                parser.consume();
                Ok(RelationalOperator::Lt)
            }
            Some(TokenKind::LtEquals) => {
                parser.consume();
                Ok(RelationalOperator::Lte)
            }
            Some(TokenKind::GreaterThan) => {
                parser.consume();
                Ok(RelationalOperator::Gt)
            }
            Some(TokenKind::GtEquals) => {
                parser.consume();
                Ok(RelationalOperator::Gte)
            }
            Some(TokenKind::MatchEq) => {
                parser.consume();
                Ok(RelationalOperator::MatchEq)
            }
            Some(TokenKind::MatchNeq) => {
                parser.consume();
                Ok(RelationalOperator::MatchNeq)
            }
            Some(TokenKind::MatchLt) => {
                parser.consume();
                Ok(RelationalOperator::MatchLt)
            }
            Some(TokenKind::MatchLte) => {
                parser.consume();
                Ok(RelationalOperator::MatchLte)
            }
            Some(TokenKind::MatchGt) => {
                parser.consume();
                Ok(RelationalOperator::MatchGt)
            }
            Some(TokenKind::MatchGte) => {
                parser.consume();
                Ok(RelationalOperator::MatchGte)
            }
            _ => Err(parser.error("expected relational operator")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Sll)) => {
                parser.consume();
                Ok(ShiftOperator::Sll)
            }
            Some(TokenKind::Keyword(KeywordKind::Srl)) => {
                parser.consume();
                Ok(ShiftOperator::Srl)
            }
            Some(TokenKind::Keyword(KeywordKind::Sla)) => {
                parser.consume();
                Ok(ShiftOperator::Sla)
            }
            Some(TokenKind::Keyword(KeywordKind::Sra)) => {
                parser.consume();
                Ok(ShiftOperator::Sra)
            }
            Some(TokenKind::Keyword(KeywordKind::Rol)) => {
                parser.consume();
                Ok(ShiftOperator::Rol)
            }
            Some(TokenKind::Keyword(KeywordKind::Ror)) => {
                parser.consume();
                Ok(ShiftOperator::Ror)
            }
            _ => Err(parser.error("expected shift operator (sll, srl, sla, sra, rol, or ror)")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::And)) => {
                parser.consume();
                Ok(LogicalOperator::And)
            }
            Some(TokenKind::Keyword(KeywordKind::Or)) => {
                parser.consume();
                Ok(LogicalOperator::Or)
            }
            Some(TokenKind::Keyword(KeywordKind::Nand)) => {
                parser.consume();
                Ok(LogicalOperator::Nand)
            }
            Some(TokenKind::Keyword(KeywordKind::Nor)) => {
                parser.consume();
                Ok(LogicalOperator::Nor)
            }
            Some(TokenKind::Keyword(KeywordKind::Xor)) => {
                parser.consume();
                Ok(LogicalOperator::Xor)
            }
            Some(TokenKind::Keyword(KeywordKind::Xnor)) => {
                parser.consume();
                Ok(LogicalOperator::Xnor)
            }
            _ => Err(parser.error("expected logical operator (and, or, nand, nor, xor, or xnor)")),
        }
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
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::DoubleStar) => {
                parser.consume();
                Ok(MiscellaneousOperator::DoubleStar)
            }
            Some(TokenKind::Keyword(KeywordKind::Abs)) => {
                parser.consume();
                Ok(MiscellaneousOperator::Abs)
            }
            Some(TokenKind::Keyword(KeywordKind::Not)) => {
                parser.consume();
                Ok(MiscellaneousOperator::Not)
            }
            _ => Err(parser.error("expected miscellaneous operator (**, abs, or not)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, _indent_level: usize) -> std::fmt::Result {
        match self {
            MiscellaneousOperator::DoubleStar => write!(f, "**"),
            MiscellaneousOperator::Abs => write!(f, "abs"),
            MiscellaneousOperator::Not => write!(f, "not"),
        }
    }
}
