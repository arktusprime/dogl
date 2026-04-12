//! Application-layer placeholder use cases.
//!
//! These functions reserve stable host-facing boundaries while deferring parser,
//! lowering, adapter, and serialization behavior.

use crate::{
    layout,
    domain::DoglFile,
    resolver::{resolve, ResolverOutput},
    syntax::{lex, parse as parse_syntax, SyntaxDocument},
    validation::{self, ValidationReport, ValidationSourceMap},
};

pub trait BpmnExporter {
    fn export_bpmn(&self, file: &DoglFile) -> Result<BpmnExport, ApplicationError>;
}

pub trait DoglExporter {
    fn render_dogl(&self, file: &DoglFile) -> Result<String, ApplicationError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
    #[error("application surface `{0}` is not implemented")]
    NotImplemented(&'static str),
    #[error("layout error: {0}")]
    Layout(#[from] crate::layout::LayoutError),
    #[error("serialization error: {0}")]
    Serialize(String),
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

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStageOutput {
    pub validation: LayoutValidation,
    pub laid_out_file: Option<DoglFile>,
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

pub fn apply_layout(file: &DoglFile) -> Result<DoglFile, ApplicationError> {
    layout::compute(file).map_err(ApplicationError::Layout)
}

pub fn layout_parse_output(output: &ParseOutput) -> Result<LayoutStageOutput, ApplicationError> {
    let validation = validate_for_layout(output);
    if !validation.can_run_layout {
        return Ok(LayoutStageOutput {
            validation,
            laid_out_file: None,
        });
    }

    let Some(file) = output.semantic_file.as_ref() else {
        return Ok(LayoutStageOutput {
            validation,
            laid_out_file: None,
        });
    };

    let laid_out_file = apply_layout(file)?;
    Ok(LayoutStageOutput {
        validation,
        laid_out_file: Some(laid_out_file),
    })
}

pub fn render_dogl(file: &DoglFile, exporter: &impl DoglExporter) -> Result<String, ApplicationError> {
    exporter.render_dogl(file)
}

pub fn to_json(_file: &DoglFile) -> Result<JsonOutput, ApplicationError> {
    Err(ApplicationError::NotImplemented("to_json"))
}

pub fn import_bpmn(_xml: &str) -> Result<ParseOutput, ApplicationError> {
    Err(ApplicationError::NotImplemented("import_bpmn"))
}

pub fn export_bpmn(file: &DoglFile, exporter: &impl BpmnExporter) -> Result<BpmnExport, ApplicationError> {
    exporter.export_bpmn(file)
}
