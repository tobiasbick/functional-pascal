//! Cached document and project semantic analysis.

mod cache;
mod document;
mod project;
mod service;

pub use document::{DiagnosticAnalysis, DocumentAnalysis, SemanticAnalysis};
pub use service::LanguageService;

#[cfg(test)]
mod collection_diagnostics;
#[cfg(test)]
mod disk_reads;
