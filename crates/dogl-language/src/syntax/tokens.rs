use super::source::Span;

/// Placeholder token stream entry for future lexer and parser work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
    pub leading_trivia: TriviaRange,
    pub trailing_trivia: TriviaRange,
}

impl SyntaxToken {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
            leading_trivia: TriviaRange::empty(),
            trailing_trivia: TriviaRange::empty(),
        }
    }

    pub fn with_leading_trivia(mut self, leading_trivia: TriviaRange) -> Self {
        self.leading_trivia = leading_trivia;
        self
    }

    pub fn with_trailing_trivia(mut self, trailing_trivia: TriviaRange) -> Self {
        self.trailing_trivia = trailing_trivia;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Keyword,
    Symbol,
    Literal,
    Newline,
    Indent,
    Dedent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTrivia {
    pub kind: SyntaxTriviaKind,
    pub span: Span,
    pub text: String,
}

impl SyntaxTrivia {
    pub fn new(kind: SyntaxTriviaKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTriviaKind {
    Whitespace,
    Comment,
    Annotation,
}

/// Half-open range into [`SyntaxDocument::trivia`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TriviaRange {
    pub start: usize,
    pub end: usize,
}

impl TriviaRange {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "trivia range start must not exceed end");
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}
