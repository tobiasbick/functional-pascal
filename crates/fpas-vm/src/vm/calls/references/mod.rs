//! Synchronous storage authority, parameter validation and frame cleanup.

mod arguments;
mod debug_stores;
mod operations;
mod scopes;

pub(in crate::vm) use scopes::ReferenceScopes;
