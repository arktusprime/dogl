use crate::{
    domain::DoglFile,
    resolver::{BindingSummary, NormalizationPass, ResolverDiagnostic},
};

/// Placeholder semantic output of the lowering pipeline.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoweredSemanticFile {
    pub semantic_file: Option<DoglFile>,
}

/// Placeholder resolver output that keeps binding, normalization, diagnostics,
/// and semantic lowering as explicit phases.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolverOutput {
    pub bindings: BindingSummary,
    pub lowering: LoweredSemanticFile,
    pub diagnostics: Vec<ResolverDiagnostic>,
    pub normalization_passes: Vec<NormalizationPass>,
}
