//! Association list AST nodes.

use super::expression::Expression;
use super::interface::InterfaceList;
use super::name::Name;
use super::type_def::{SubtypeIndication, TypeMark};

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
