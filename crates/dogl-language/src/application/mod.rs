//! Application-layer placeholder use cases.
//!
//! These functions reserve stable host-facing boundaries while deferring parser,
//! lowering, adapter, and serialization behavior.

use crate::{
    domain::DoglFile,
    resolver::{resolve, ResolverOutput},
    syntax::{lex, parse as parse_syntax, SyntaxDocument},
    validation::{self, ValidationReport, ValidationSourceMap},
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutValidation {
    pub report: ValidationReport,
    pub can_run_layout: bool,
}

pub fn parse(source: &str) -> ParseOutput {
    let syntax = parse_syntax(lex(source));
    let resolver = resolve(&syntax);
    let semantic_file = resolver.lowering.semantic_file.clone();

    ParseOutput {
        syntax,
        resolver,
        semantic_file,
    }
}

pub fn validate(file: &DoglFile) -> ValidationReport {
    validation::validate(file)
}

pub fn validate_parse_output(output: &ParseOutput) -> ValidationReport {
    let Some(file) = output.semantic_file.as_ref() else {
        return ValidationReport::default();
    };

    let source_map = ValidationSourceMap::from_syntax(&output.syntax);
    validation::validate_with_source_map(file, &source_map)
}

pub fn validate_for_layout(output: &ParseOutput) -> LayoutValidation {
    if output.semantic_file.is_none() {
        return LayoutValidation {
            report: ValidationReport::default(),
            can_run_layout: false,
        };
    }

    let report = validate_parse_output(output);
    LayoutValidation {
        can_run_layout: !report.has_errors(),
        report,
    }
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
