//! Callable signatures and their explicit parameters.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`

use super::{GenericParamDef, Ty};

/// Resolved function signature used by semantic analysis and editor tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTy {
    /// Whether the function explicitly guarantees pure evaluation.
    pub pure: bool,
    /// Generic parameters declared by the function.
    pub type_params: Vec<GenericParamDef>,
    /// Function parameters in source order.
    pub params: Vec<ParamTy>,
    /// Resolved function return type.
    pub return_type: Box<Ty>,
    /// Accept any number of arguments beyond the declared params (e.g. `Std.Str.Format`).
    pub variadic: bool,
}

/// Resolved procedure signature used by semantic analysis and editor tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureTy {
    /// Generic parameters declared by the procedure.
    pub type_params: Vec<GenericParamDef>,
    /// Procedure parameters in source order.
    pub params: Vec<ParamTy>,
    /// Accept any number of arguments at the call site (e.g. `Std.Console.WriteLn`).
    pub variadic: bool,
}

/// One resolved callable parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamTy {
    /// Whether the parameter borrows caller storage with the explicit `var` mode.
    pub mutable: bool,
    /// Case-preserving parameter name.
    pub name: String,
    /// Resolved parameter type.
    pub ty: Ty,
}

impl Ty {
    /// Generic parameters owned by this callable value, before contextual instantiation.
    pub(crate) fn callable_type_parameters(&self) -> &[GenericParamDef] {
        match self {
            Ty::Function(function) => &function.type_params,
            Ty::Procedure(procedure) => &procedure.type_params,
            _ => &[],
        }
    }
}
