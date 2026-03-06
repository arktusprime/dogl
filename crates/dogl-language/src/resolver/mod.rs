//! Resolution and semantic lowering placeholders.
//!
//! This layer sits explicitly between source-oriented syntax structures and the
//! semantic domain.

mod binding;
mod diagnostics;
mod lowering;
mod normalization;

pub use binding::BindingSummary;
pub use diagnostics::{ResolverDiagnostic, ResolverDiagnosticSeverity};
pub use lowering::{LoweredSemanticFile, ResolverOutput};
pub use normalization::NormalizationPass;
