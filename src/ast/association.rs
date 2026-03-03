//! Association list AST nodes.

use super::expression::Expression;
use super::interface::InterfaceList;
use super::name::Name;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{SubtypeIndication, TypeMark};
use crate::parser::{ParseError, Parser};
use crate::{KeywordKind, TokenKind};

/// EBNF: `association_element ::= [ formal_part => ] actual_part`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationElement {
    pub formal: Option<FormalPart>,
    pub actual: ActualPart,
}

/// EBNF: `association_list ::= association_element { , association_element }`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationList {
    pub elements: Vec<AssociationElement>,
}

/// EBNF: `formal_designator ::= generic_name | port_name | parameter_name`
///
/// All three forms are just names at the AST level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalDesignator {
    Name(Box<Name>),
}

/// EBNF: `formal_part ::= formal_designator | function_name ( formal_designator )
///     | type_mark ( formal_designator )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalPart {
    Designator(FormalDesignator),
    FunctionConversion {
        function_name: Box<Name>,
        designator: FormalDesignator,
    },
    TypeConversion {
        type_mark: TypeMark,
        designator: FormalDesignator,
    },
}

/// EBNF: `formal_parameter_list ::= parameter_interface_list`
pub type FormalParameterList = InterfaceList;

/// EBNF (VHDL-2008): `actual_designator ::= [ INERTIAL ] expression | signal_name
///     | variable_name | file_name | subtype_indication | subprogram_name
///     | instantiated_package_name | OPEN`
/// EBNF (VHDL-87/93): `actual_designator ::= expression | signal_name | variable_name
///     | file_name | OPEN`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualDesignator {
    Expression {
        /// VHDL-2008.
        inertial: bool,
        expression: Box<Expression>,
    },
    Name(Box<Name>),
    SubtypeIndication(SubtypeIndication),
    Open,
}

/// EBNF: `actual_part ::= actual_designator | function_name ( actual_designator )
///     | type_mark ( actual_designator )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualPart {
    Designator(ActualDesignator),
    FunctionConversion {
        function_name: Box<Name>,
        designator: ActualDesignator,
    },
    TypeConversion {
        type_mark: TypeMark,
        designator: ActualDesignator,
    },
}

/// EBNF: `actual_parameter_part ::= parameter_association_list`
pub type ActualParameterPart = AssociationList;

// ---------------------------------------------------------------------------
// AstNode implementations
// ---------------------------------------------------------------------------

impl AstNode for AssociationElement {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // association_element ::= [ formal_part => ] actual_part
        // Use backtracking: save position, try to parse formal_part + =>.
        // If arrow found, it has a formal part. Otherwise restore and parse just actual_part.
        let save = parser.save();
        let formal = if let Ok(fp) = FormalPart::parse(parser) {
            if parser.consume_if(TokenKind::Arrow).is_some() {
                Some(fp)
            } else {
                parser.restore(save);
                None
            }
        } else {
            parser.restore(save);
            None
        };

        let actual = ActualPart::parse(parser)?;
        Ok(AssociationElement { formal, actual })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        if let Some(formal) = &self.formal {
            formal.format(f, indent_level)?;
            write!(f, " => ")?;
        }
        self.actual.format(f, indent_level)
    }
}

impl AstNode for AssociationList {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let mut elements = vec![AssociationElement::parse(parser)?];
        while parser.consume_if(TokenKind::Comma).is_some() {
            elements.push(AssociationElement::parse(parser)?);
        }
        Ok(AssociationList { elements })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.elements, f, indent_level)
    }
}

impl AstNode for FormalDesignator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let name = Name::parse(parser)?;
        Ok(FormalDesignator::Name(Box::new(name)))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            FormalDesignator::Name(name) => name.format(f, indent_level),
        }
    }
}

impl AstNode for FormalPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // formal_part ::= formal_designator | function_name ( formal_designator )
        //     | type_mark ( formal_designator )
        // For simplicity: parse a name. If it's followed by '(' and then another name + ')',
        // it could be a function/type conversion. But distinguishing function_name from type_mark
        // requires semantic analysis. For now, parse as simple designator (a Name).
        // The Name parser should handle indexed names like `func(signal_name)`.
        let name = Name::parse(parser)?;
        Ok(FormalPart::Designator(FormalDesignator::Name(Box::new(
            name,
        ))))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            FormalPart::Designator(desig) => desig.format(f, indent_level),
            FormalPart::FunctionConversion {
                function_name,
                designator,
            } => {
                function_name.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
            FormalPart::TypeConversion {
                type_mark,
                designator,
            } => {
                type_mark.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}

impl AstNode for ActualDesignator {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // actual_designator ::= [INERTIAL] expression | OPEN
        if parser.at_keyword(KeywordKind::Open) {
            parser.consume();
            return Ok(ActualDesignator::Open);
        }

        let inertial = parser.consume_if_keyword(KeywordKind::Inertial).is_some();

        let expression = Expression::parse(parser)?;
        Ok(ActualDesignator::Expression {
            inertial,
            expression: Box::new(expression),
        })
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ActualDesignator::Expression {
                inertial,
                expression,
            } => {
                if *inertial {
                    write!(f, "inertial ")?;
                }
                expression.format(f, indent_level)
            }
            ActualDesignator::Name(name) => name.format(f, indent_level),
            ActualDesignator::SubtypeIndication(subtype) => subtype.format(f, indent_level),
            ActualDesignator::Open => write!(f, "open"),
        }
    }
}

impl AstNode for ActualPart {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // actual_part ::= actual_designator | function_name ( actual_designator )
        //     | type_mark ( actual_designator )
        // For simplicity, parse as actual_designator. The Name/Expression parser
        // will handle function calls and type conversions within expressions.
        let designator = ActualDesignator::parse(parser)?;
        Ok(ActualPart::Designator(designator))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ActualPart::Designator(desig) => desig.format(f, indent_level),
            ActualPart::FunctionConversion {
                function_name,
                designator,
            } => {
                function_name.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
            ActualPart::TypeConversion {
                type_mark,
                designator,
            } => {
                type_mark.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}
