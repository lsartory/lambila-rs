//! Signal-related AST nodes.

use super::expression::Expression;
use super::name::Name;
use super::node::{AstNode, format_comma_separated, write_indent};
use super::type_def::TypeMark;
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `signal_list ::= signal_name { , signal_name } | OTHERS | ALL`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalList {
    Names(Vec<Name>),
    Others,
    All,
}

/// EBNF: `guarded_signal_specification ::= guarded_signal_list : type_mark`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedSignalSpecification {
    pub signal_list: SignalList,
    pub type_mark: TypeMark,
}

/// EBNF: `disconnection_specification ::= DISCONNECT guarded_signal_specification
///     AFTER time_expression ;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectionSpecification {
    pub guarded_signal_spec: GuardedSignalSpecification,
    pub time_expression: Expression,
}

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for SignalList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        if parser.consume_if_keyword(KeywordKind::Others).is_some() {
            Ok(SignalList::Others)
        } else if parser.consume_if_keyword(KeywordKind::All).is_some() {
            Ok(SignalList::All)
        } else {
            let mut names = vec![Name::parse(parser)?];
            while parser.consume_if(TokenKind::Comma).is_some() {
                names.push(Name::parse(parser)?);
            }
            Ok(SignalList::Names(names))
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            SignalList::Names(names) => format_comma_separated(names, f, indent_level),
            SignalList::Others => write!(f, "others"),
            SignalList::All => write!(f, "all"),
        }
    }
}

impl AstNode for GuardedSignalSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let signal_list = SignalList::parse(parser)?;
        parser.expect(TokenKind::Colon)?;
        let type_mark = TypeMark::parse(parser)?;
        Ok(GuardedSignalSpecification {
            signal_list,
            type_mark,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        self.signal_list.format(f, indent_level)?;
        write!(f, " : ")?;
        self.type_mark.format(f, indent_level)
    }
}

impl AstNode for DisconnectionSpecification {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        parser.expect_keyword(KeywordKind::Disconnect)?;
        let guarded_signal_spec = GuardedSignalSpecification::parse(parser)?;
        parser.expect_keyword(KeywordKind::After)?;
        let time_expression = Expression::parse(parser)?;
        parser.expect(TokenKind::Semicolon)?;
        Ok(DisconnectionSpecification {
            guarded_signal_spec,
            time_expression,
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        write_indent(f, indent_level)?;
        write!(f, "disconnect ")?;
        self.guarded_signal_spec.format(f, indent_level)?;
        write!(f, " after ")?;
        self.time_expression.format(f, indent_level)?;
        writeln!(f, ";")
    }
}
