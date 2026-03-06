/// Placeholder source container for parser-facing APIs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceFile {
    pub text: String,
}

impl SourceFile {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourcePosition {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl Span {
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }
}
