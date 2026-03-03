//! Association list AST nodes.

use super::expression::Expression;
use super::interface::InterfaceList;
use super::name::Name;
use super::node::{AstNode, format_comma_separated};
use super::type_def::{SubtypeIndication, TypeMark};
use crate::parser::{Parser, ParseError};

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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        format_comma_separated(&self.elements, f, indent_level)
    }
}

impl AstNode for FormalDesignator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            FormalDesignator::Name(name) => name.format(f, indent_level),
        }
    }
}

impl AstNode for FormalPart {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            FormalPart::Designator(desig) => desig.format(f, indent_level),
            FormalPart::FunctionConversion { function_name, designator } => {
                function_name.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
            FormalPart::TypeConversion { type_mark, designator } => {
                type_mark.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}

impl AstNode for ActualDesignator {
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ActualDesignator::Expression { inertial, expression } => {
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
    fn parse(_parser: &mut Parser) -> Result<Self, ParseError> {
        todo!()
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, indent_level: usize) -> std::fmt::Result {
        match self {
            ActualPart::Designator(desig) => desig.format(f, indent_level),
            ActualPart::FunctionConversion { function_name, designator } => {
                function_name.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
            ActualPart::TypeConversion { type_mark, designator } => {
                type_mark.format(f, indent_level)?;
                write!(f, "(")?;
                designator.format(f, indent_level)?;
                write!(f, ")")
            }
        }
    }
}
