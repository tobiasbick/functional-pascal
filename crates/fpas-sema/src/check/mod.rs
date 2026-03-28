mod calls;
mod context;
mod decl;
mod entry;
mod expr;
mod name_resolution;
mod spans;
mod stmt;

pub(crate) use context::Checker;
pub use context::ExprTypeMap;
pub use context::MethodCallMap;
