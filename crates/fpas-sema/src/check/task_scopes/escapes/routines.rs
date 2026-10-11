//! Named routines analyzed together, so parameter escapes can depend on each other.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use std::collections::HashMap;

use fpas_parser::{Decl, FormalParam, FuncBody, RecordMethod, TypeBody};

use crate::check::Checker;
use crate::scope::canonical_symbol_name;

/// One named routine of the analyzed unit or program.
pub(super) struct Routine<'a> {
    /// Name used in diagnostics and, for unit-level routines and methods, in the interface.
    pub(super) name: String,
    /// Declared parameters, including an explicit `Self` receiver.
    pub(super) params: &'a [FormalParam],
    /// Executable body.
    pub(super) body: &'a FuncBody,
    /// Indexes of the routines declared directly in this body.
    pub(super) nested: HashMap<String, usize>,
    /// Names of implicit capture parameters, in the semantic capture order.
    pub(super) captures: Vec<String>,
    /// Lexically enclosing named routine, for sibling and ancestor lookup.
    pub(super) parent: Option<usize>,
}

/// Every named routine with the lookup tables that resolve calls to it.
#[derive(Default)]
pub(super) struct RoutineTable<'a> {
    pub(super) routines: Vec<Routine<'a>>,
    /// Unit-level routines by canonical name.
    pub(super) top_level: HashMap<String, usize>,
    /// Record routines by canonical `Type.Method` name.
    pub(super) methods: HashMap<String, usize>,
}

impl<'a> RoutineTable<'a> {
    /// Collect unit-level routines, record routines, and their nested routines.
    pub(super) fn collect(declarations: &'a [Decl], checker: &Checker) -> Self {
        let mut table = Self::default();
        for declaration in declarations {
            match declaration {
                Decl::Function(function) => {
                    let index = table.add(
                        &function.name,
                        &function.params,
                        &function.body,
                        crate::function_decl_lookup_key(function),
                        None,
                        checker,
                    );
                    table
                        .top_level
                        .insert(canonical_symbol_name(&function.name), index);
                }
                Decl::Procedure(procedure) => {
                    let index = table.add(
                        &procedure.name,
                        &procedure.params,
                        &procedure.body,
                        crate::procedure_decl_lookup_key(procedure),
                        None,
                        checker,
                    );
                    table
                        .top_level
                        .insert(canonical_symbol_name(&procedure.name), index);
                }
                Decl::TypeDef(definition) => {
                    let TypeBody::Record(record) = &definition.body else {
                        continue;
                    };
                    for method in &record.methods {
                        let (params, body, key) = match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => (
                                &function.params,
                                &function.body,
                                crate::function_decl_lookup_key(function),
                            ),
                            RecordMethod::Procedure(procedure)
                            | RecordMethod::StaticProcedure(procedure) => (
                                &procedure.params,
                                &procedure.body,
                                crate::procedure_decl_lookup_key(procedure),
                            ),
                        };
                        let name = format!("{}.{}", definition.name, method.name());
                        let index = table.add(&name, params, body, key, None, checker);
                        table.methods.insert(canonical_symbol_name(&name), index);
                    }
                }
                Decl::Const(_) | Decl::Var(_) => {}
            }
        }
        table
    }

    fn add(
        &mut self,
        name: &str,
        params: &'a [FormalParam],
        body: &'a FuncBody,
        key: usize,
        parent: Option<usize>,
        checker: &Checker,
    ) -> usize {
        let index = self.routines.len();
        self.routines.push(Routine {
            name: name.to_string(),
            params,
            body,
            nested: HashMap::new(),
            captures: checker
                .nested_routine_captures
                .get(&key)
                .map(|info| {
                    info.captures
                        .iter()
                        .map(|capture| capture.name.clone())
                        .collect()
                })
                .unwrap_or_default(),
            parent,
        });
        let FuncBody::Block { nested, .. } = body;
        for declaration in nested {
            let (nested_name, params, body, key) = match declaration {
                Decl::Function(function) => (
                    &function.name,
                    &function.params,
                    &function.body,
                    crate::function_decl_lookup_key(function),
                ),
                Decl::Procedure(procedure) => (
                    &procedure.name,
                    &procedure.params,
                    &procedure.body,
                    crate::procedure_decl_lookup_key(procedure),
                ),
                Decl::TypeDef(_) | Decl::Const(_) | Decl::Var(_) => continue,
            };
            let nested_index = self.add(
                &format!("{name}.{nested_name}"),
                params,
                body,
                key,
                Some(index),
                checker,
            );
            self.routines[index]
                .nested
                .insert(canonical_symbol_name(nested_name), nested_index);
        }
        index
    }
}
