//! Abstract Syntax Tree (AST) definitions for all VHDL language constructs.
//!
//! This module contains struct and enum definitions for every syntactic
//! grammar rule defined in the VHDL EBNF specifications across all
//! supported language versions:
//!
//! - **VHDL-1987** (IEEE Std 1076-1987)
//! - **VHDL-1993** (IEEE Std 1076-1993)
//! - **VHDL-2008** (IEEE Std 1076-2008)
//!
//! The types represent the superset of all versions. Version-specific
//! constructs are documented with their applicable version and use
//! `Option<>` fields or enum variants to naturally accommodate version
//! differences.
//!
//! This module contains **only data structures** — no parsing, lowering,
//! or analysis logic.

pub mod architecture;
pub mod association;
pub mod attribute;
pub mod common;
pub mod component;
pub mod concurrent;
pub mod configuration;
pub mod context;
pub mod design_unit;
pub mod entity;
pub mod expression;
pub mod generate;
pub mod group;
pub mod interface;
pub mod literal;
pub mod name;
pub mod object_decl;
pub mod package;
pub mod sequential;
pub mod signal;
pub mod subprogram;
pub mod type_def;
