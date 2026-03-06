//! Syntax-facing structures.
//!
//! These placeholders reserve the source-oriented layer for tokens, source
//! locations, syntax nodes, and parse-time diagnostics.

mod diagnostics;
mod nodes;
mod source;
mod tokens;

pub use diagnostics::{ParseDiagnostic, ParseDiagnosticMetadata, ParseDiagnosticSeverity};
pub use nodes::{
    RecoveryKind, RecoveryNode, SyntaxDocument, SyntaxKind, SyntaxNode, SyntaxNodeId, TokenRange,
    UnresolvedName, UnresolvedNameKind,
};
pub use source::{SourceFile, SourcePosition, Span};
pub use tokens::{SyntaxToken, SyntaxTrivia, SyntaxTriviaKind, TokenKind, TriviaRange};
