//! Application-layer placeholder use cases.
//!
//! These functions reserve stable host-facing boundaries while deferring parser,
//! lowering, adapter, and serialization behavior.

use crate::{
    domain::DoglFile,
    resolver::ResolverOutput,
    syntax::{SourceFile, SyntaxDocument},
    validation::{self, ValidationReport},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    NotImplemented(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonOutput {
    pub version: &'static str,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnExport {
    pub xml: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParseOutput {
    pub syntax: SyntaxDocument,
    pub resolver: ResolverOutput,
    pub semantic_file: Option<DoglFile>,
}

pub fn parse(source: &str) -> ParseOutput {
    ParseOutput {
        syntax: SyntaxDocument {
            source: SourceFile::new(source),
            ..SyntaxDocument::default()
        },
        resolver: ResolverOutput::default(),
        semantic_file: None,
    }
}

pub fn validate(file: &DoglFile) -> ValidationReport {
    validation::validate(file)
}

pub fn to_json(_file: &DoglFile) -> Result<JsonOutput, ApplicationError> {
    Err(ApplicationError::NotImplemented("to_json"))
}

pub fn import_bpmn(xml: &str) -> ParseOutput {
    parse(xml)
}

pub fn export_bpmn(_file: &DoglFile) -> Result<BpmnExport, ApplicationError> {
    Err(ApplicationError::NotImplemented("export_bpmn"))
}
