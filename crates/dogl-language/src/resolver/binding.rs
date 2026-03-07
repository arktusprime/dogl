/// Placeholder summary of binding work performed before name resolution and
/// semantic lowering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingSummary {
    pub bound_names: usize,
    pub unresolved_names: usize,
}
