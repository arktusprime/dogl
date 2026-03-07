use crate::{
    domain::DoglFile,
    resolver::{
        BindingSummary, NameResolutionSummary, NormalizationPass, ResolverDiagnostic,
    },
};

/// Placeholder semantic output of the explicit lowering boundary between
/// resolver-owned intermediate data and the semantic domain.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoweredSemanticFile {
    pub semantic_file: Option<DoglFile>,
}

/// Placeholder resolver output that keeps binding, name resolution,
/// normalization, diagnostics, and semantic lowering as explicit phases.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolverOutput {
    /// Placeholder result of binding declarations and local names.
    pub bindings: BindingSummary,
    /// Placeholder result of resolving references after binding.
    pub resolution: NameResolutionSummary,
    /// Placeholder list of normalization passes run before lowering.
    pub normalization_passes: Vec<NormalizationPass>,
    /// Placeholder semantic-domain output produced only by explicit lowering.
    pub lowering: LoweredSemanticFile,
    /// Placeholder diagnostics emitted by resolver-owned stages.
    pub diagnostics: Vec<ResolverDiagnostic>,
}
