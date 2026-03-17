use super::{
    diagnostics::ParseDiagnostic,
    source::{SourceFile, Span},
    tokens::{SyntaxToken, SyntaxTrivia, SyntaxTriviaKind},
};

/// Placeholder syntax document that keeps source-facing output separate from
/// the semantic domain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyntaxDocument {
    pub source: SourceFile,
    pub tokens: Vec<SyntaxToken>,
    pub nodes: Vec<SyntaxNode>,
    pub root: Option<SyntaxNodeId>,
    pub trivia: Vec<SyntaxTrivia>,
    pub unresolved_names: Vec<UnresolvedName>,
    pub recoveries: Vec<RecoveryNode>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Option<Span>,
    pub token_range: Option<TokenRange>,
    pub children: Vec<SyntaxNodeId>,
    pub text_name: Option<String>,
    pub has_recovery: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    File,
    Collab,
    LayoutSection,
    Pool,
    Lane,
    Stage,
    Event,
    Task,
    Gateway,
    Flow,
    Expression,
    Bounds,
    Identifier,
    StringLiteral,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxNodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenRange {
    pub start: usize,
    pub end: usize,
}

impl TokenRange {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "token range start must not exceed end");
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedName {
    pub kind: UnresolvedNameKind,
    pub text: String,
    pub span: Option<Span>,
}

impl UnresolvedName {
    pub fn new(kind: UnresolvedNameKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnresolvedNameKind {
    Identifier,
    QualifiedName,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryNode {
    pub kind: RecoveryKind,
    pub span: Option<Span>,
    pub message: String,
}

impl RecoveryNode {
    pub fn new(kind: RecoveryKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            span: None,
            message: message.into(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKind {
    MissingToken,
    UnexpectedToken,
    IncompleteNode,
}

impl SyntaxDocument {
    pub fn new(source: SourceFile) -> Self {
        Self {
            source,
            ..Self::default()
        }
    }

    pub fn node(&self, id: SyntaxNodeId) -> Option<&SyntaxNode> {
        self.nodes.get(id.0)
    }

    pub fn node_mut(&mut self, id: SyntaxNodeId) -> Option<&mut SyntaxNode> {
        self.nodes.get_mut(id.0)
    }

    pub fn root_node(&self) -> Option<&SyntaxNode> {
        self.root.and_then(|id| self.node(id))
    }

    pub fn push_token(&mut self, token: SyntaxToken) -> usize {
        let idx = self.tokens.len();
        self.tokens.push(token);
        idx
    }

    pub fn push_trivia(&mut self, trivia: SyntaxTrivia) -> usize {
        let idx = self.trivia.len();
        self.trivia.push(trivia);
        idx
    }

    pub fn push_node(&mut self, node: SyntaxNode) -> SyntaxNodeId {
        let id = SyntaxNodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn set_root(&mut self, root: SyntaxNodeId) {
        self.root = Some(root);
    }

    pub fn comments(&self) -> impl Iterator<Item = &SyntaxTrivia> {
        self.trivia
            .iter()
            .filter(|trivia| trivia.kind == SyntaxTriviaKind::Comment)
    }

    pub fn trivia_slice(&self, range: super::tokens::TriviaRange) -> Option<&[SyntaxTrivia]> {
        self.trivia.get(range.start..range.end)
    }
}

impl SyntaxNode {
    pub fn new(kind: SyntaxKind) -> Self {
        Self {
            kind,
            span: None,
            token_range: None,
            children: Vec::new(),
            text_name: None,
            has_recovery: false,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_token_range(mut self, token_range: TokenRange) -> Self {
        self.token_range = Some(token_range);
        self
    }

    pub fn with_text_name(mut self, text_name: impl Into<String>) -> Self {
        self.text_name = Some(text_name.into());
        self
    }

    pub fn with_children(mut self, children: impl Into<Vec<SyntaxNodeId>>) -> Self {
        self.children = children.into();
        self
    }

    pub fn push_child(&mut self, child: SyntaxNodeId) {
        self.children.push(child);
    }

    pub fn mark_recovered(mut self) -> Self {
        self.has_recovery = true;
        self
    }
}
