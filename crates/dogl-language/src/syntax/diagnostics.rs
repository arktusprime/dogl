use crate::syntax::Span;

/// Placeholder parse-time diagnostic surface for future parser work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub severity: ParseDiagnosticSeverity,
    pub message: String,
    pub span: Option<Span>,
}

impl ParseDiagnostic {
    pub fn new(severity: ParseDiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiagnosticSeverity {
    Error,
    Warning,
    Note,
}
