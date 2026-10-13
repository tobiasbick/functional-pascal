//! Routine body scopes, contracts, and lexical capture analysis.
//! Documentation: docs/pascal/language/functions/closures.md and
//! docs/pascal/language/functions/contracts.md.

use crate::check::Checker;
use crate::check::closures::{CaptureAnalysis, collect_captures};
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{ParamTy, Ty, TypeConstraint};
use fpas_parser::{FuncBody, Stmt};

impl Checker {
    /// Check a routine body and return lexical captures of the routine itself.
    ///
    /// Heading clauses are checked and captured before body-local declarations are introduced.
    /// **Documentation:** `docs/pascal/language/functions/closures.md` and
    /// `docs/pascal/language/functions/contracts.md`.
    pub(crate) fn check_routine_body_collecting_captures(
        &mut self,
        name: &str,
        type_params: &[fpas_parser::TypeParam],
        params: &[ParamTy],
        source_params: &[fpas_parser::FormalParam],
        return_type: Option<Ty>,
        input: super::super::contracts::RoutineBody<'_>,
    ) -> CaptureAnalysis {
        let previous_task_call_depth = std::mem::replace(&mut self.task_call_depth, 0);
        let super::super::contracts::RoutineBody {
            body,
            requires,
            ensures,
        } = input;
        let FuncBody::Block { nested, stmts } = body;
        self.routine_type_path.push(name.to_owned());
        self.discard_results.push(true);

        self.scopes.push_scope();
        let routine_scope_index = self.scopes.scope_count() - 1;
        self.task_scopes.enter_routine();

        let resolved_parameters = self
            .scopes
            .lookup(name)
            .map(|symbol| match &symbol.ty {
                Ty::Function(function) => function.type_params.clone(),
                Ty::Procedure(procedure) => procedure.type_params.clone(),
                _ => Vec::new(),
            })
            .unwrap_or_default();
        for tp in type_params {
            let constraint = tp
                .constraint
                .as_ref()
                .and_then(|c| TypeConstraint::from_name(c));
            self.scopes.define(
                &tp.name,
                Symbol {
                    constant: None,
                    ty: Ty::GenericParam(
                        resolved_parameters
                            .iter()
                            .find(|parameter| {
                                parameter.display_name().eq_ignore_ascii_case(&tp.name)
                            })
                            .map_or_else(|| tp.name.clone(), |parameter| parameter.name.clone()),
                        constraint,
                    ),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }

        for (p, source) in params.iter().zip(source_params) {
            self.scopes.define_with_declaration(
                &p.name,
                Symbol {
                    constant: None,
                    ty: p.ty.clone(),
                    mutable: p.is_var(),
                    kind: SymbolKind::Param,
                    task_bound: false,
                },
                source.span,
            );
            self.record_binding_discard_info(&p.name, &p.ty, false, None);
        }

        let prev_ctx = self.scopes.function_ctx.take();
        let owner_unit = if let Some(context) = &prev_ctx {
            context.owner_unit.clone()
        } else {
            name.rsplit_once('.').map(|(unit, _)| unit.to_string())
        };
        self.scopes.function_ctx = Some(FunctionCtx {
            name: name.to_string(),
            return_type,
            owner_unit,
        });

        self.check_preconditions(requires);
        self.check_postconditions(ensures);
        let clause_body = FuncBody::Block {
            nested: Vec::new(),
            stmts: Vec::new(),
        };
        let clause_captures = collect_captures(
            &self.scopes,
            routine_scope_index,
            super::super::contracts::RoutineBody {
                body: &clause_body,
                requires,
                ensures,
            },
            &self.closure_infos,
            &self.nested_routine_captures,
            &self.pending_routine_captures,
            &self.capture_dependencies.closures,
        );

        self.collect_routine_signatures(nested);
        let hoisted = if nested.is_empty() {
            Vec::new()
        } else {
            self.hoist_body_locals(stmts)
        };
        self.check_scope_declarations(nested);
        for name in &hoisted {
            self.scopes.remove_from_current(name);
        }
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        let mut captures = collect_captures(
            &self.scopes,
            routine_scope_index,
            super::super::contracts::RoutineBody {
                body,
                requires: &[],
                ensures: &[],
            },
            &self.closure_infos,
            &self.nested_routine_captures,
            &self.pending_routine_captures,
            &self.capture_dependencies.closures,
        );

        for capture in clause_captures.captures {
            if !captures.captures.iter().any(|existing| {
                existing.name.eq_ignore_ascii_case(&capture.name)
                    && existing.declaration == capture.declaration
            }) {
                captures.captures.push(capture);
            }
        }
        captures.captures.sort_by_key(|capture| {
            self.scopes
                .capture_scope_index(&capture.name, capture.declaration)
        });

        self.complete_capture_discard_info(&mut captures.captures);
        self.remember_capture_scopes(&captures.captures);
        self.task_scopes.exit_routine();
        self.scopes.function_ctx = prev_ctx;
        self.routine_type_path.pop();
        self.scopes.pop_scope();
        let result = self.discard_results.pop().unwrap_or(false);
        self.task_call_depth = previous_task_call_depth;
        self.routine_discard_results
            .insert(name.to_string(), result);
        self.scopes.set_discard_info(
            name,
            fpas_unit::interface::DiscardInfo {
                value: captures.captures.iter().all(|capture| capture.task_free),
                result,
            },
        );
        captures
    }

    /// Make function-body locals visible to nested routine bodies without changing
    /// source-order value visibility in the enclosing body.
    ///
    /// Nested declarations are checked before body statements, but
    /// `docs/pascal/language/functions/closures.md` lets named nested routines capture
    /// enclosing locals. Locals nested inside inner `begin` blocks stay hidden.
    pub(crate) fn hoist_body_locals(&mut self, stmts: &[Stmt]) -> Vec<String> {
        let mut names = Vec::new();
        for stmt in stmts {
            let (variable, mutable) = match stmt {
                Stmt::Const(variable) => (variable, false),
                Stmt::Var(variable) => (variable, true),
                _ => continue,
            };
            let ty = match &variable.type_expr {
                Some(annotation) => self.resolve_type_expr(annotation),
                None => {
                    // Probe the capture type now; ordered statement checking reports
                    // initializer diagnostics and applies the local inference rules.
                    let diagnostic_count = self.errors.len();
                    let ty = self.check_expr(&variable.value);
                    self.errors.truncate(diagnostic_count);
                    ty
                }
            };
            let constant = matches!(stmt, Stmt::Const(_)).then(|| {
                // Nested routines can use static enclosing constants in labels before
                // the enclosing statement list is checked in execution order.
                let compile_time =
                    self.const_initializer_is_compile_time_known(&variable.value, &ty);
                crate::scope::ConstantInfo {
                    record: compile_time
                        .then(|| self.record_constant_value(&variable.value))
                        .flatten(),
                    compile_time,
                    value: if compile_time {
                        self.scalar_constant_value(&variable.value)
                    } else {
                        None
                    },
                }
            });
            let defined = self.scopes.define_with_declaration(
                &variable.name,
                Symbol {
                    constant,
                    ty: ty.clone(),
                    mutable,
                    kind: if matches!(stmt, Stmt::Const(_)) {
                        SymbolKind::Const
                    } else {
                        SymbolKind::Var
                    },
                    task_bound: false,
                },
                variable.span,
            );
            if defined {
                self.record_binding_discard_info(&variable.name, &ty, mutable, None);
                names.push(variable.name.clone());
            }
        }
        names
    }
}
