//! Syntax-facing structures.
//!
//! These placeholders reserve the source-oriented layer for tokens, source
//! locations, syntax nodes, and parse-time diagnostics.

mod diagnostics;
mod source;
mod tree;

pub use diagnostics::{ParseDiagnostic, ParseDiagnosticSeverity};
pub use source::{SourceFile, SourcePosition, Span};
pub use tree::{SyntaxDocument, SyntaxKind, SyntaxNode, TokenKind, TokenSpan, TriviaKind};
