//! DOGL language core crate.
//!
//! This crate establishes the architectural boundaries for syntax, resolution,
//! semantic domain, application use cases, and validation without locking in
//! the final parser or adapter behavior yet.

pub mod syntax;
pub mod resolver;
pub mod domain;
pub mod application;
pub mod validation;
pub mod layout;
pub mod adapters;

pub use application::{
    apply_layout, export_bpmn, import_bpmn, layout_parse_output, parse, render_dogl, to_json,
    validate, validate_for_layout, validate_parse_output, ApplicationError, BpmnExport,
    JsonOutput, LayoutStageOutput, LayoutValidation, ParseOutput,
};
