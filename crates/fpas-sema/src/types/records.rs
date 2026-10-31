//! Nominal record identities and generic instantiation.
//! See `docs/pascal/language/types/generics.md` and `docs/pascal/language/types/records.md`.

use super::{FunctionTy, GenericParamDef, ProcedureTy, Ty, substitute_type_parameters};
use std::collections::HashMap;

/// Canonical standard resource type identities and their release operations.
pub(crate) fn native_resource_release(name: &str) -> Option<&'static str> {
    use fpas_builtins::std_symbols as s;
    [
        (s::STD_FS_FILE, s::STD_FS_CLOSE),
        (s::STD_NET_CONNECTION, s::STD_NET_CLOSE),
        (s::STD_NET_LISTENER, s::STD_NET_CLOSE_LISTENER),
        (s::STD_PROC_PROCESS, s::STD_PROC_CLOSE),
        ("Std.Ai.Acp.Connection", "Std.Ai.Acp.Release"),
        ("Std.Ai.Mcp.Server", s::STD_MCP_RELEASE),
        ("Std.Ai.Mcp.Endpoint", s::STD_MCP_CLOSE_ENDPOINT),
        (
            "Std.Net.Http.Handles.BodyStream",
            "Std.Net.Http.CloseStream",
        ),
    ]
    .into_iter()
    .find_map(|(ty, release)| name.eq_ignore_ascii_case(ty).then_some(release))
}

/// Operations that borrow resource inputs but return a separately owned resource.
pub(crate) fn native_resource_fresh_result(name: &str) -> bool {
    use fpas_builtins::std_symbols as s;
    [s::STD_NET_ACCEPT, s::STD_NET_ACCEPT_WITH_CANCELLATION]
        .iter()
        .any(|operation| name.eq_ignore_ascii_case(operation))
}

/// Resolved record shape, ownership, generic arguments, and members.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTy {
    /// Case-preserving qualified declaration name, shared by all instantiations.
    pub name: String,
    /// Exact declaring unit, or `None` for local and intrinsic records.
    pub owner_unit: Option<String>,
    /// Generic parameters in declaration order.
    pub type_params: Vec<GenericParamDef>,
    /// Actual arguments; empty on an uninstantiated declaration.
    pub type_args: Vec<Ty>,
    /// Case-preserving names of members not declared `public`.
    pub private_members: Vec<String>,
    /// Stored fields in source order.
    pub fields: Vec<(String, Ty)>,
    /// Instance methods with implicit `Self`.
    pub methods: Vec<(String, MethodKind)>,
    /// Name of the instance procedure used for deterministic resource release.
    pub release_method: Option<String>,
    /// Static functions without a receiver.
    pub static_functions: Vec<(String, FunctionTy)>,
    /// Static procedures without a receiver.
    pub static_procedures: Vec<(String, ProcedureTy)>,
}

/// Whether a record method returns a value.
#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    /// Function signature including its receiver.
    Function(FunctionTy),
    /// Procedure signature including its receiver.
    Procedure(ProcedureTy),
}

impl RecordTy {
    /// Create a structural header before recursive fields and members are resolved.
    pub(crate) fn header(
        name: String,
        owner_unit: Option<String>,
        type_params: Vec<GenericParamDef>,
    ) -> Self {
        Self {
            name,
            owner_unit,
            type_params,
            type_args: Vec::new(),
            private_members: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            release_method: None,
            static_functions: Vec::new(),
            static_procedures: Vec::new(),
        }
    }
    /// Substitute arguments into a declaration's fields and callable signatures.
    pub fn instantiate(&self, arguments: Vec<Ty>) -> Self {
        let bindings: HashMap<_, _> = self
            .type_params
            .iter()
            .zip(&arguments)
            .map(|(parameter, argument)| (parameter.name.to_ascii_lowercase(), argument.clone()))
            .collect();
        let mut record = self.clone();
        let fixed_parameters = if self.type_args.is_empty() {
            self.type_params.len()
        } else {
            0
        };
        record.type_args = arguments;
        for (_, ty) in &mut record.fields {
            *ty = substitute_type_parameters(ty, &bindings);
        }
        for (_, method) in &mut record.methods {
            match method {
                MethodKind::Function(function) => {
                    if let Ty::Function(value) =
                        substitute_type_parameters(&Ty::Function(function.clone()), &bindings)
                    {
                        *function = value;
                        function
                            .type_params
                            .drain(..fixed_parameters.min(function.type_params.len()));
                    }
                }
                MethodKind::Procedure(procedure) => {
                    if let Ty::Procedure(value) =
                        substitute_type_parameters(&Ty::Procedure(procedure.clone()), &bindings)
                    {
                        *procedure = value;
                        procedure
                            .type_params
                            .drain(..fixed_parameters.min(procedure.type_params.len()));
                    }
                }
            }
        }
        for (_, function) in &mut record.static_functions {
            if let Ty::Function(value) =
                substitute_type_parameters(&Ty::Function(function.clone()), &bindings)
            {
                *function = value;
                function
                    .type_params
                    .drain(..fixed_parameters.min(function.type_params.len()));
            }
        }
        for (_, procedure) in &mut record.static_procedures {
            if let Ty::Procedure(value) =
                substitute_type_parameters(&Ty::Procedure(procedure.clone()), &bindings)
            {
                *procedure = value;
                procedure
                    .type_params
                    .drain(..fixed_parameters.min(procedure.type_params.len()));
            }
        }
        record
    }
}
