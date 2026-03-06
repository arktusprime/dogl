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

pub use application::{
    export_bpmn, import_bpmn, parse, to_json, validate, ApplicationError, BpmnExport,
    JsonOutput, ParseOutput,
};
