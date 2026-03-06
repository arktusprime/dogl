/// Placeholder summary of binding work performed by the resolver.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingSummary {
    pub bound_names: usize,
    pub unresolved_names: usize,
}
