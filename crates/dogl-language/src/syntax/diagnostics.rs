use super::source::Span;

/// Placeholder parse-time diagnostic surface for future parser work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub severity: ParseDiagnosticSeverity,
    pub message: String,
    pub span: Option<Span>,
    pub metadata: ParseDiagnosticMetadata,
}

impl ParseDiagnostic {
    pub fn new(severity: ParseDiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            span: None,
            metadata: ParseDiagnosticMetadata::default(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.metadata.code = Some(code);
        self
    }

    pub fn mark_recovered(mut self) -> Self {
        self.metadata.recovered = true;
        self
    }

    pub fn with_related_span(mut self, span: Span) -> Self {
        self.metadata.related_spans.push(span);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseDiagnosticMetadata {
    pub code: Option<&'static str>,
    pub related_spans: Vec<Span>,
    pub recovered: bool,
}
