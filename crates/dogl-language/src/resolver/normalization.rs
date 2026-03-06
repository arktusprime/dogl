/// Placeholder normalization pass descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationPass {
    pub name: &'static str,
}

impl NormalizationPass {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}
