/// Placeholder diagnostics emitted by name resolution and lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverDiagnostic {
    pub severity: ResolverDiagnosticSeverity,
    pub message: String,
}

impl ResolverDiagnostic {
    pub fn new(severity: ResolverDiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverDiagnosticSeverity {
    Error,
    Warning,
    Note,
}
