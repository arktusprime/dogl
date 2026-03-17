//! Syntax-facing structures.
//!
//! These placeholders reserve the source-oriented layer for tokens, source
//! locations, syntax nodes, and parse-time diagnostics.

mod diagnostics;
mod lexer;
mod nodes;
mod parser;
mod source;
mod tokens;

pub use diagnostics::{ParseDiagnostic, ParseDiagnosticMetadata, ParseDiagnosticSeverity};
pub use lexer::lex;
pub use nodes::{
    RecoveryKind, RecoveryNode, SyntaxDocument, SyntaxKind, SyntaxNode, SyntaxNodeId, TokenRange,
    UnresolvedName, UnresolvedNameKind,
};
pub use parser::parse;
pub use source::{SourceFile, SourcePosition, Span};
pub use tokens::{SyntaxToken, SyntaxTrivia, SyntaxTriviaKind, TokenKind, TriviaRange};
