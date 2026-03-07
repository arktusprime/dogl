/// Placeholder summary of name-resolution work performed after binding and
/// before normalization and semantic lowering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NameResolutionSummary {
    pub resolved_references: usize,
    pub unresolved_references: usize,
}
