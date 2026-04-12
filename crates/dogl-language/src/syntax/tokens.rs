use super::source::Span;

/// Token stream entry emitted by the syntax-layer lexer.
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
    KeywordCollab,
    KeywordLayout,
    PoolMarker,
    LaneMarker,
    StageMarker,
    EventMarker,
    EventStartMarker,
    EventIntermediateMarker,
    EventEndMarker,
    GatewayMarker,
    GatewayExclusiveMarker,
    GatewayParallelMarker,
    GatewayEventBasedMarker,
    GatewayInclusiveMarker,
    GatewayComplexMarker,
    TaskMarker,
    BracketCommand,
    CommandValue,
    FlowArrow,
    Identifier,
    StringLiteral,
    Number,
    LeftBrace,
    RightBrace,
    Newline,
    Indent,
    Dedent,
    Eof,
    Unknown,
}

impl TokenKind {
    pub fn is_event_marker(self) -> bool {
        matches!(
            self,
            TokenKind::EventMarker
                | TokenKind::EventStartMarker
                | TokenKind::EventIntermediateMarker
                | TokenKind::EventEndMarker
        )
    }

    pub fn is_gateway_marker(self) -> bool {
        matches!(
            self,
            TokenKind::GatewayMarker
                | TokenKind::GatewayExclusiveMarker
                | TokenKind::GatewayParallelMarker
                | TokenKind::GatewayEventBasedMarker
                | TokenKind::GatewayInclusiveMarker
                | TokenKind::GatewayComplexMarker
        )
    }
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
