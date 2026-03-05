//! Project & workspace management for multi-file VHDL compilation.
//!
//! This module provides:
//!
//! - [`source::SourceManager`] — load and deduplicate source files.
//! - [`library::Library`] — group design units into VHDL libraries.
//! - [`workspace::Workspace`] — the top-level project container.
//! - [`dependency::DependencyGraph`] — compute compilation order.

pub mod dependency;
pub mod library;
pub mod source;
pub mod workspace;
