//! Name-related AST nodes.

use super::common::*;
use super::expression::Expression;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{DiscreteRange, SubtypeIndication};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// A VHDL name.
///
/// EBNF (VHDL-2008): `name ::= simple_name | operator_symbol | character_literal
///     | selected_name | indexed_name | slice_name | attribute_name | external_name`
///
/// VHDL-87/93 omit `external_name` and (87 omits) `character_literal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Simple(SimpleName),
    OperatorSymbol(OperatorSymbol),
    CharacterLiteral(String),
    Selected(Box<SelectedName>),
    Indexed(Box<IndexedName>),
    Slice(Box<SliceName>),
    Attribute(Box<AttributeName>),
    /// VHDL-2008 external name.
    External(Box<ExternalName>),
}

/// A prefix for selected / indexed / slice / attribute names.
///
/// EBNF: `prefix ::= name | function_call`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix {
    Name(Box<Name>),
    FunctionCall(Box<FunctionCall>),
}

/// A suffix used in selected names.
///
/// EBNF: `suffix ::= simple_name | character_literal | operator_symbol | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suffix {
    SimpleName(SimpleName),
    CharacterLiteral(String),
    OperatorSymbol(OperatorSymbol),
    All,
}

/// A selected name (dot notation).
///
/// EBNF: `selected_name ::= prefix . suffix`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedName {
    pub prefix: Prefix,
    pub suffix: Suffix,
}

/// An indexed name.
///
/// EBNF: `indexed_name ::= prefix ( expression { , expression } )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedName {
    pub prefix: Prefix,
    pub expressions: Vec<Expression>,
}

/// A slice name.
///
/// EBNF: `slice_name ::= prefix ( discrete_range )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceName {
    pub prefix: Prefix,
    pub discrete_range: DiscreteRange,
}

/// An attribute designator.
///
/// EBNF: `attribute_designator ::= attribute_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDesignator {
    pub name: SimpleName,
}

/// An attribute name.
///
/// EBNF (VHDL-2008): `attribute_name ::= prefix [ signature ] ' attribute_designator [ ( expression ) ]`
///
/// VHDL-87: omits optional signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeName {
    pub prefix: Prefix,
    /// VHDL-93+ only.
    pub signature: Option<Signature>,
    pub designator: AttributeDesignator,
    pub expression: Option<Box<Expression>>,
}

/// An external name (VHDL-2008).
///
/// EBNF: `external_name ::= external_constant_name | external_signal_name | external_variable_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalName {
    Constant(ExternalConstantName),
    Signal(ExternalSignalName),
    Variable(ExternalVariableName),
}

/// EBNF: `external_constant_name ::= << CONSTANT external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalConstantName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_signal_name ::= << SIGNAL external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSignalName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_variable_name ::= << VARIABLE external_pathname : subtype_indication >>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalVariableName {
    pub pathname: ExternalPathname,
    pub subtype_indication: SubtypeIndication,
}

/// EBNF: `external_pathname ::= package_pathname | absolute_pathname | relative_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalPathname {
    Package(PackagePathname),
    Absolute(AbsolutePathname),
    Relative(RelativePathname),
}

/// EBNF: `package_pathname ::= @ library_logical_name . { package_simple_name . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePathname {
    pub library_name: Identifier,
    pub package_names: Vec<SimpleName>,
    pub object_name: SimpleName,
}

/// EBNF: `absolute_pathname ::= . partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsolutePathname {
    pub partial: PartialPathname,
}

/// EBNF: `relative_pathname ::= { ^ . } partial_pathname`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePathname {
    pub up_count: usize,
    pub partial: PartialPathname,
}

/// EBNF: `partial_pathname ::= { pathname_element . } object_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialPathname {
    pub elements: Vec<PathnameElement>,
    pub object_name: SimpleName,
}

/// EBNF: `pathname_element ::= entity_simple_name | component_instantiation_label
///     | block_label | generate_statement_label [ ( static_expression ) ] | package_simple_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathnameElement {
    pub name: Identifier,
    /// Present for generate_statement_label with an index expression.
    pub expression: Option<Box<Expression>>,
}

/// A function call.
///
/// EBNF: `function_call ::= function_name [ ( actual_parameter_part ) ]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub function_name: Box<Name>,
    pub parameters: Option<super::association::ActualParameterPart>,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for Name {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // 1. Parse a base name
        let mut name = parse_base_name(parser)?;

        // 2. Suffix loop: while the next token indicates a suffix, consume it
        loop {
            match parser.peek_kind() {
                // `.` -> SelectedName
                Some(TokenKind::Dot) => {
                    parser.consume(); // consume the dot
                    let suffix = Suffix::parse(parser)?;
                    name = Name::Selected(Box::new(SelectedName {
                        prefix: Prefix::Name(Box::new(name)),
                        suffix,
                    }));
                }
                // `(` -> Could be IndexedName, SliceName, or FunctionCall
                Some(TokenKind::LeftParen) => {
                    name = parse_paren_suffix(parser, name)?;
                }
                // `'` (Tick) -> AttributeName (but NOT qualified expression: tick+paren
                // is handled by the expression parser, not here)
                Some(TokenKind::Tick) => {
                    // Check if the token after tick is an identifier (attribute designator).
                    // If it's `(`, this is a qualified expression, not an attribute name --
                    // leave it for the expression parser.
                    if let Some(next) = parser.peek_nth(1) {
                        if next.kind == TokenKind::Identifier
                            || next.kind == TokenKind::ExtendedIdentifier
                        {
                            // Also need to check it's not a keyword being used as an attribute
                            // like 'range, 'left, etc. -- but those would be identifiers in
                            // a well-formed file. Some predefined attributes use keywords though.
                            parser.consume(); // consume tick
                            let designator = AttributeDesignator::parse(parser)?;
                            // Optional ( expression )
                            let expression = if parser.at(TokenKind::LeftParen) {
                                parser.consume();
                                let expr = Expression::parse(parser)?;
                                parser.expect(TokenKind::RightParen)?;
                                Some(Box::new(expr))
                            } else {
                                None
                            };
                            name = Name::Attribute(Box::new(AttributeName {
                                prefix: Prefix::Name(Box::new(name)),
                                signature: None,
                                designator,
                                expression,
                            }));
                        } else if next.kind == TokenKind::Keyword(KeywordKind::Range) {
                            // 'range is a predefined attribute
                            parser.consume(); // consume tick
                            let token = parser.consume().unwrap(); // consume 'range'
                            let designator = AttributeDesignator {
                                name: SimpleName {
                                    identifier: Identifier::Basic(token.text.clone()),
                                },
                            };
                            let expression = if parser.at(TokenKind::LeftParen) {
                                parser.consume();
                                let expr = Expression::parse(parser)?;
                                parser.expect(TokenKind::RightParen)?;
                                Some(Box::new(expr))
                            } else {
                                None
                            };
                            name = Name::Attribute(Box::new(AttributeName {
                                prefix: Prefix::Name(Box::new(name)),
                                signature: None,
                                designator,
                                expression,
                            }));
                        } else {
                            // Not an attribute name (could be tick+paren for qualified expr)
                            break;
                        }
                    } else {
                        break;
                    }
                }
                // `[` -> Signature followed by `'` for AttributeName
                Some(TokenKind::LeftBracket) => {
                    let signature = Signature::parse(parser)?;
                    parser.expect(TokenKind::Tick)?;
                    let designator = AttributeDesignator::parse(parser)?;
                    let expression = if parser.at(TokenKind::LeftParen) {
                        parser.consume();
                        let expr = Expression::parse(parser)?;
                        parser.expect(TokenKind::RightParen)?;
                        Some(Box::new(expr))
                    } else {
                        None
                    };
                    name = Name::Attribute(Box::new(AttributeName {
                        prefix: Prefix::Name(Box::new(name)),
                        signature: Some(signature),
                        designator,
                        expression,
                    }));
                }
                _ => break,
            }
        }

        Ok(name)
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Name::Simple(n) => n.format(f, indent_level),
            Name::OperatorSymbol(n) => n.format(f, indent_level),
            Name::CharacterLiteral(c) => write!(f, "'{}'", c),
            Name::Selected(n) => n.format(f, indent_level),
            Name::Indexed(n) => n.format(f, indent_level),
            Name::Slice(n) => n.format(f, indent_level),
            Name::Attribute(n) => n.format(f, indent_level),
            Name::External(n) => n.format(f, indent_level),
        }
    }
}

/// Parse a base name (the initial part before any suffixes).
fn parse_base_name(parser: &mut Parser) -> Result<Name, ParseError> {
    match parser.peek_kind() {
        Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
            let id = Identifier::parse(parser)?;
            Ok(Name::Simple(SimpleName { identifier: id }))
        }
        Some(TokenKind::StringLiteral) => {
            let token = parser.consume().unwrap();
            // Strip surrounding quotes from text
            let inner = token
                .text
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string();
            Ok(Name::OperatorSymbol(OperatorSymbol { text: inner }))
        }
        Some(TokenKind::CharacterLiteral) => {
            let token = parser.consume().unwrap();
            // Strip surrounding single quotes
            let inner = token
                .text
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .to_string();
            Ok(Name::CharacterLiteral(inner))
        }
        Some(TokenKind::DoubleLess) => {
            // External name: << ... >>
            let ext = ExternalName::parse(parser)?;
            Ok(Name::External(Box::new(ext)))
        }
        _ => Err(parser.error(
            "expected name (identifier, operator symbol, character literal, or external name)",
        )),
    }
}

/// Parse a parenthesized suffix after a name: could be indexed, slice, or function call.
/// Disambiguate by checking for TO/DOWNTO (discrete_range -> slice) vs expression list (indexed).
fn parse_paren_suffix(parser: &mut Parser, prefix_name: Name) -> Result<Name, ParseError> {
    parser.expect(TokenKind::LeftParen)?;

    // If the parenthesized list is empty, treat as indexed with empty list (shouldn't occur normally)
    if parser.at(TokenKind::RightParen) {
        parser.consume();
        return Ok(Name::Indexed(Box::new(IndexedName {
            prefix: Prefix::Name(Box::new(prefix_name)),
            expressions: Vec::new(),
        })));
    }

    // Parse the first expression and check if it contains TO/DOWNTO for a discrete_range.
    // We need to try parsing as a discrete_range first, then fall back to expression list.
    let save = parser.save();

    // Try to parse as a single expression, then check for TO/DOWNTO
    let first_expr = Expression::parse(parser)?;

    if parser.at_keyword(KeywordKind::To) || parser.at_keyword(KeywordKind::Downto) {
        // This is a slice name: prefix ( simple_expression direction simple_expression )
        // However, we parsed a full Expression, which might be more than a SimpleExpression.
        // For the discrete_range, we need to backtrack and parse properly.
        parser.restore(save);
        let discrete_range = parse_discrete_range_in_parens(parser)?;
        parser.expect(TokenKind::RightParen)?;
        return Ok(Name::Slice(Box::new(SliceName {
            prefix: Prefix::Name(Box::new(prefix_name)),
            discrete_range,
        })));
    }

    // Check for RANGE keyword after a name (subtype_indication style discrete range)
    if parser.at_keyword(KeywordKind::Range) {
        parser.restore(save);
        let discrete_range = parse_discrete_range_in_parens(parser)?;
        parser.expect(TokenKind::RightParen)?;
        return Ok(Name::Slice(Box::new(SliceName {
            prefix: Prefix::Name(Box::new(prefix_name)),
            discrete_range,
        })));
    }

    // Otherwise it's an indexed name with a list of expressions
    let mut expressions = vec![first_expr];
    while parser.consume_if(TokenKind::Comma).is_some() {
        expressions.push(Expression::parse(parser)?);
    }
    parser.expect(TokenKind::RightParen)?;

    Ok(Name::Indexed(Box::new(IndexedName {
        prefix: Prefix::Name(Box::new(prefix_name)),
        expressions,
    })))
}

/// Parse a discrete_range inside parentheses (for slice names).
fn parse_discrete_range_in_parens(parser: &mut Parser) -> Result<DiscreteRange, ParseError> {
    use super::expression::SimpleExpression;
    use super::type_def::{Range, TypeMark};

    // Try parsing as simple_expression direction simple_expression first.
    let save = parser.save();
    if let Ok(left) = SimpleExpression::parse(parser) {
        if parser.at_keyword(KeywordKind::To) {
            parser.consume();
            let right = SimpleExpression::parse(parser)?;
            return Ok(DiscreteRange::Range(Range::Explicit {
                left,
                direction: Direction::To,
                right,
            }));
        } else if parser.at_keyword(KeywordKind::Downto) {
            parser.consume();
            let right = SimpleExpression::parse(parser)?;
            return Ok(DiscreteRange::Range(Range::Explicit {
                left,
                direction: Direction::Downto,
                right,
            }));
        } else if parser.at_keyword(KeywordKind::Range) {
            // This was a type_mark followed by RANGE: subtype indication style
            parser.restore(save);
            let name = Name::parse(parser)?;
            let type_mark = TypeMark::TypeName(Box::new(name));
            parser.expect_keyword(KeywordKind::Range)?;

            let range_left = SimpleExpression::parse(parser)?;
            if parser.at_keyword(KeywordKind::To) {
                parser.consume();
                let range_right = SimpleExpression::parse(parser)?;
                return Ok(DiscreteRange::SubtypeIndication(SubtypeIndication {
                    resolution: None,
                    type_mark,
                    constraint: Some(super::type_def::Constraint::Range(
                        super::type_def::RangeConstraint {
                            range: Range::Explicit {
                                left: range_left,
                                direction: Direction::To,
                                right: range_right,
                            },
                        },
                    )),
                }));
            } else if parser.at_keyword(KeywordKind::Downto) {
                parser.consume();
                let range_right = SimpleExpression::parse(parser)?;
                return Ok(DiscreteRange::SubtypeIndication(SubtypeIndication {
                    resolution: None,
                    type_mark,
                    constraint: Some(super::type_def::Constraint::Range(
                        super::type_def::RangeConstraint {
                            range: Range::Explicit {
                                left: range_left,
                                direction: Direction::Downto,
                                right: range_right,
                            },
                        },
                    )),
                }));
            } else {
                return Err(parser.error("expected 'to' or 'downto' in range constraint"));
            }
        }
    }

    // If we get here, something went wrong
    parser.restore(save);
    Err(parser.error("expected discrete range"))
}

impl AstNode for Prefix {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Prefix is either a name or a function call.
        // In practice, we parse as a name (function calls are syntactically names with parens).
        let name = Name::parse(parser)?;
        Ok(Prefix::Name(Box::new(name)))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Prefix::Name(n) => n.format(f, indent_level),
            Prefix::FunctionCall(fc) => fc.format(f, indent_level),
        }
    }
}

impl AstNode for Suffix {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        match parser.peek_kind() {
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let name = SimpleName::parse(parser)?;
                Ok(Suffix::SimpleName(name))
            }
            Some(TokenKind::CharacterLiteral) => {
                let token = parser.consume().unwrap();
                let inner = token
                    .text
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();
                Ok(Suffix::CharacterLiteral(inner))
            }
            Some(TokenKind::StringLiteral) => {
                let token = parser.consume().unwrap();
                let inner = token
                    .text
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string();
                Ok(Suffix::OperatorSymbol(OperatorSymbol { text: inner }))
            }
            Some(TokenKind::Keyword(KeywordKind::All)) => {
                parser.consume();
                Ok(Suffix::All)
            }
            _ => Err(parser.error(
                "expected suffix (identifier, character literal, operator symbol, or 'all')",
            )),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            Suffix::SimpleName(n) => n.format(f, indent_level),
            Suffix::CharacterLiteral(c) => write!(f, "'{}'", c),
            Suffix::OperatorSymbol(n) => n.format(f, indent_level),
            Suffix::All => write!(f, "all"),
        }
    }
}

impl AstNode for SelectedName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // selected_name ::= prefix . suffix
        // Parse a full Name (which handles the suffix chain) and extract
        // the outermost SelectedName.
        let name = Name::parse(parser)?;
        match name {
            Name::Selected(sel) => Ok(*sel),
            _ => Err(parser.error("expected selected name (name.suffix)")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, ".")?;
        self.suffix.format(f, indent_level)
    }
}

impl AstNode for IndexedName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // indexed_name ::= prefix ( expression { , expression } )
        // In practice this is handled in the Name::parse suffix loop.
        let prefix = Prefix::parse(parser)?;
        parser.expect(TokenKind::LeftParen)?;
        let mut expressions = Vec::new();
        expressions.push(Expression::parse(parser)?);
        while parser.consume_if(TokenKind::Comma).is_some() {
            expressions.push(Expression::parse(parser)?);
        }
        parser.expect(TokenKind::RightParen)?;
        Ok(IndexedName {
            prefix,
            expressions,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, "(")?;
        format_comma_separated(&self.expressions, f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for SliceName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // slice_name ::= prefix ( discrete_range )
        // In practice this is handled in the Name::parse suffix loop.
        let prefix = Prefix::parse(parser)?;
        parser.expect(TokenKind::LeftParen)?;
        let discrete_range = parse_discrete_range_in_parens(parser)?;
        parser.expect(TokenKind::RightParen)?;
        Ok(SliceName {
            prefix,
            discrete_range,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        write!(f, "(")?;
        self.discrete_range.format(f, indent_level)?;
        write!(f, ")")
    }
}

impl AstNode for AttributeDesignator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // attribute_designator ::= attribute_simple_name
        // This is an identifier, but predefined attributes could be keywords.
        // Check for identifier first, then fall back to keywords used as attributes.
        match parser.peek_kind() {
            Some(TokenKind::Identifier) | Some(TokenKind::ExtendedIdentifier) => {
                let name = SimpleName::parse(parser)?;
                Ok(AttributeDesignator { name })
            }
            _ => {
                // Some predefined attributes (range, etc.) are keywords
                // but that case is handled in the Name::parse tick branch.
                Err(parser.error("expected attribute designator (identifier)"))
            }
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name.format(f, indent_level)
    }
}

impl AstNode for AttributeName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // attribute_name ::= prefix [ signature ] ' attribute_designator [ ( expression ) ]
        // In practice this is handled in the Name::parse suffix loop.
        let prefix = Prefix::parse(parser)?;

        let signature = if parser.at(TokenKind::LeftBracket) {
            Some(Signature::parse(parser)?)
        } else {
            None
        };

        parser.expect(TokenKind::Tick)?;
        let designator = AttributeDesignator::parse(parser)?;

        let expression = if parser.at(TokenKind::LeftParen) {
            parser.consume();
            let expr = Expression::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Some(Box::new(expr))
        } else {
            None
        };

        Ok(AttributeName {
            prefix,
            signature,
            designator,
            expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.prefix.format(f, indent_level)?;
        if let Some(ref sig) = self.signature {
            sig.format(f, indent_level)?;
        }
        write!(f, "'")?;
        self.designator.format(f, indent_level)?;
        if let Some(ref expr) = self.expression {
            write!(f, "(")?;
            expr.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for ExternalName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // << CONSTANT/SIGNAL/VARIABLE external_pathname : subtype_indication >>
        parser.expect(TokenKind::DoubleLess)?;
        match parser.peek_kind() {
            Some(TokenKind::Keyword(KeywordKind::Constant)) => {
                parser.consume();
                let pathname = ExternalPathname::parse(parser)?;
                parser.expect(TokenKind::Colon)?;
                let subtype_indication = SubtypeIndication::parse(parser)?;
                parser.expect(TokenKind::DoubleGreater)?;
                Ok(ExternalName::Constant(ExternalConstantName {
                    pathname,
                    subtype_indication,
                }))
            }
            Some(TokenKind::Keyword(KeywordKind::Signal)) => {
                parser.consume();
                let pathname = ExternalPathname::parse(parser)?;
                parser.expect(TokenKind::Colon)?;
                let subtype_indication = SubtypeIndication::parse(parser)?;
                parser.expect(TokenKind::DoubleGreater)?;
                Ok(ExternalName::Signal(ExternalSignalName {
                    pathname,
                    subtype_indication,
                }))
            }
            Some(TokenKind::Keyword(KeywordKind::Variable)) => {
                parser.consume();
                let pathname = ExternalPathname::parse(parser)?;
                parser.expect(TokenKind::Colon)?;
                let subtype_indication = SubtypeIndication::parse(parser)?;
                parser.expect(TokenKind::DoubleGreater)?;
                Ok(ExternalName::Variable(ExternalVariableName {
                    pathname,
                    subtype_indication,
                }))
            }
            _ => Err(parser.error("expected 'constant', 'signal', or 'variable' in external name")),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ExternalName::Constant(n) => n.format(f, indent_level),
            ExternalName::Signal(n) => n.format(f, indent_level),
            ExternalName::Variable(n) => n.format(f, indent_level),
        }
    }
}

impl AstNode for ExternalConstantName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::DoubleLess)?;
        parser.expect_keyword(KeywordKind::Constant)?;
        let pathname = ExternalPathname::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        parser.expect(TokenKind::DoubleGreater)?;
        Ok(ExternalConstantName {
            pathname,
            subtype_indication,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< constant ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalSignalName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::DoubleLess)?;
        parser.expect_keyword(KeywordKind::Signal)?;
        let pathname = ExternalPathname::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        parser.expect(TokenKind::DoubleGreater)?;
        Ok(ExternalSignalName {
            pathname,
            subtype_indication,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< signal ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalVariableName {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect(TokenKind::DoubleLess)?;
        parser.expect_keyword(KeywordKind::Variable)?;
        let pathname = ExternalPathname::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let subtype_indication = SubtypeIndication::parse(parser)?;
        parser.expect(TokenKind::DoubleGreater)?;
        Ok(ExternalVariableName {
            pathname,
            subtype_indication,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "<< variable ")?;
        self.pathname.format(f, indent_level)?;
        write!(f, " : ")?;
        self.subtype_indication.format(f, indent_level)?;
        write!(f, " >>")
    }
}

impl AstNode for ExternalPathname {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // Determine which kind of pathname based on the first token:
        // - `.` -> absolute_pathname
        // - `^` -> relative_pathname (but ^ is not tokenized; handle as identifier text)
        // - `@` -> package_pathname (but @ is not tokenized; handle via Error token text)
        // - identifier -> relative_pathname with up_count=0

        if parser.at(TokenKind::Dot) {
            // absolute_pathname ::= . partial_pathname
            let abs = AbsolutePathname::parse(parser)?;
            return Ok(ExternalPathname::Absolute(abs));
        }

        // Check for ^ (relative pathname with up-count) by looking at the token text.
        // The lexer may produce these as Error tokens. Check the token text.
        if let Some(token) = parser.peek() {
            if token.text == "@" {
                // package_pathname
                let pkg = PackagePathname::parse(parser)?;
                return Ok(ExternalPathname::Package(pkg));
            }
            if token.text == "^" {
                // relative_pathname with up-count
                let rel = RelativePathname::parse(parser)?;
                return Ok(ExternalPathname::Relative(rel));
            }
        }

        // Default: relative_pathname with up_count=0 (just a partial pathname)
        let rel = RelativePathname::parse(parser)?;
        Ok(ExternalPathname::Relative(rel))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ExternalPathname::Package(p) => p.format(f, indent_level),
            ExternalPathname::Absolute(p) => p.format(f, indent_level),
            ExternalPathname::Relative(p) => p.format(f, indent_level),
        }
    }
}

impl AstNode for PackagePathname {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // @ library_logical_name . { package_simple_name . } object_simple_name
        // @ is not a standard token; consume it (might be Error token with text "@")
        if parser.eof() {
            return Err(parser.error("expected '@' in package pathname"));
        }
        let token = parser.consume().unwrap();
        if token.text != "@" {
            return Err(ParseError {
                message: format!("expected '@', found '{}'", token.text),
                span: Some(token.span),
            });
        }

        let library_name = Identifier::parse(parser)?;
        parser.expect(TokenKind::Dot)?;

        // Parse { package_simple_name . } object_simple_name
        // We need to look ahead to see if there's another dot after each name.
        let mut names = Vec::new();
        names.push(SimpleName::parse(parser)?);

        while parser.consume_if(TokenKind::Dot).is_some() {
            names.push(SimpleName::parse(parser)?);
        }

        // The last name is the object_name, everything else is package_names
        let object_name = names.pop().unwrap();

        Ok(PackagePathname {
            library_name,
            package_names: names,
            object_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, "@")?;
        self.library_name.format(f, indent_level)?;
        for pkg in &self.package_names {
            write!(f, ".")?;
            pkg.format(f, indent_level)?;
        }
        write!(f, ".")?;
        self.object_name.format(f, indent_level)
    }
}

impl AstNode for AbsolutePathname {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // . partial_pathname
        parser.expect(TokenKind::Dot)?;
        let partial = PartialPathname::parse(parser)?;
        Ok(AbsolutePathname { partial })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write!(f, ".")?;
        self.partial.format(f, indent_level)
    }
}

impl AstNode for RelativePathname {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { ^ . } partial_pathname
        let mut up_count = 0;
        // ^ is not a standard token, check token text
        while let Some(token) = parser.peek() {
            if token.text == "^" {
                parser.consume();
                parser.expect(TokenKind::Dot)?;
                up_count += 1;
            } else {
                break;
            }
        }
        let partial = PartialPathname::parse(parser)?;
        Ok(RelativePathname { up_count, partial })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for i in 0..self.up_count {
            write!(f, "^")?;
            if i < self.up_count - 1 {
                write!(f, ".")?;
            }
        }
        if self.up_count > 0 {
            write!(f, ".")?;
        }
        self.partial.format(f, indent_level)
    }
}

impl AstNode for PartialPathname {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // { pathname_element . } object_simple_name
        // Parse a sequence of identifiers (with optional generate index) separated by dots.
        // All but the last are pathname_elements; the last is the object_name.
        let mut all = Vec::new();
        all.push(PathnameElement::parse(parser)?);

        while parser.at(TokenKind::Dot) {
            // Look ahead: is there an identifier after the dot?
            // (Colon would indicate end of pathname in external name context)
            if let Some(next) = parser.peek_nth(1) {
                if next.kind == TokenKind::Identifier || next.kind == TokenKind::ExtendedIdentifier
                {
                    parser.consume(); // consume dot
                    all.push(PathnameElement::parse(parser)?);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Last element becomes object_name, rest are pathname elements
        let last = all.pop().unwrap();
        let object_name = SimpleName {
            identifier: last.name,
        };

        Ok(PartialPathname {
            elements: all,
            object_name,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        for elem in &self.elements {
            elem.format(f, indent_level)?;
            write!(f, ".")?;
        }
        self.object_name.format(f, indent_level)
    }
}

impl AstNode for PathnameElement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let name = Identifier::parse(parser)?;
        let expression = if parser.at(TokenKind::LeftParen) {
            parser.consume();
            let expr = Expression::parse(parser)?;
            parser.expect(TokenKind::RightParen)?;
            Some(Box::new(expr))
        } else {
            None
        };
        Ok(PathnameElement { name, expression })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.name.format(f, indent_level)?;
        if let Some(ref expr) = self.expression {
            write!(f, "(")?;
            expr.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl AstNode for FunctionCall {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // function_call ::= function_name [ ( actual_parameter_part ) ]
        // Parse the function name (which could be any Name).
        let function_name = Name::parse(parser)?;

        // In a name context, the parenthesized arguments are already consumed
        // as part of IndexedName in Name::parse. So if we got here with Name::Indexed,
        // we need to extract the parts. Otherwise, check for optional parens.
        // Actually, the task description says: "FunctionCall: parse Name and extract it."
        // So we just wrap the parsed name.
        Ok(FunctionCall {
            function_name: Box::new(function_name),
            parameters: None,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.function_name.format(f, indent_level)?;
        if let Some(ref params) = self.parameters {
            write!(f, "(")?;
            params.format(f, indent_level)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}
