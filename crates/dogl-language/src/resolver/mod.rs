//! Resolver-layer placeholders.
//!
//! This layer sits explicitly between source-oriented syntax structures and the
//! semantic domain. Parsing may produce syntax-facing data, but binding, name
//! resolution, normalization, resolver diagnostics, and semantic lowering stay
//! reserved here as explicit intermediate stages.

mod binding;
mod diagnostics;
mod lowering;
mod normalization;
mod resolution;

pub use binding::BindingSummary;
pub use diagnostics::{ResolverDiagnostic, ResolverDiagnosticSeverity};
pub use lowering::{LoweredSemanticFile, ResolverOutput};
pub use normalization::NormalizationPass;
pub use resolution::NameResolutionSummary;
