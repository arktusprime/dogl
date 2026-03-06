use crate::syntax::{ParseDiagnostic, SourceFile, Span};

/// Placeholder syntax document that keeps source-facing output separate from
/// the semantic domain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyntaxDocument {
    pub source: SourceFile,
    pub nodes: Vec<SyntaxNode>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Option<Span>,
    pub trivia: Vec<TriviaKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    File,
    Collaboration,
    Participant,
    Process,
    Statement,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Keyword,
    Symbol,
    Trivia,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSpan {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    Comment,
    Whitespace,
}
