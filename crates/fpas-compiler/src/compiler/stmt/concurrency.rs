//! Statement lowering for `go` (task spawning) and `select` (channel multiplexing).
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`

use super::super::Compiler;
use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Intrinsic, Op};
use fpas_diagnostics::codes::COMPILE_INVALID_GO_EXPRESSION;
use fpas_lexer::Span;
use fpas_parser::{
    Designator, DesignatorPart, Expr, FormalParam, FuncBody, QualifiedId, SelectArm, Stmt, TypeExpr,
};
use fpas_sema::Ty;

impl Compiler {
    /// Compile `go Func(args)` as a statement (fire-and-forget: discard task handle).
    pub(super) fn compile_go_stmt(&mut self, expr: &Expr, span: Span) -> Result<(), CompileError> {
        self.compile_go(expr, span, true)
    }

    /// Compile `go CallExpr` as an expression that pushes a `Value::Task(id)`.
    pub(crate) fn compile_go_expr(&mut self, expr: &Expr, span: Span) -> Result<(), CompileError> {
        self.compile_go(expr, span, false)
    }

    fn compile_go(&mut self, expr: &Expr, span: Span, detached: bool) -> Result<(), CompileError> {
        let loc = (span.line, span.column);
        match expr {
            Expr::Call {
                designator, args, ..
            } => {
                let call_key = std::ptr::from_ref(expr) as usize;
                let returns_value = self.go_call_returns_value(expr);

                if let Some(qualified) = self.method_calls.get(&call_key).cloned() {
                    let receiver = Designator {
                        parts: designator.parts[..designator.parts.len() - 1].to_vec(),
                        span: designator.span,
                    };
                    let mut wrapper_args = Vec::with_capacity(args.len() + 1);
                    wrapper_args.push(Expr::Designator(receiver));
                    wrapper_args.extend(args.iter().cloned());
                    self.compile_go_wrapper_call(
                        &qualified,
                        &wrapper_args,
                        returns_value,
                        detached,
                        span,
                    )?;
                    return Ok(());
                }

                let name = Self::resolve_designator_name(designator);
                let qualified = self.qualify_name(&name).to_string();
                if qualified.starts_with("Std.") {
                    self.compile_go_wrapper_call(&qualified, args, returns_value, detached, span)?;
                    return Ok(());
                }

                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.compile_designator_read(designator)?;
                self.emit_go_spawn(args.len() as u8, detached, loc);
                Ok(())
            }
            _ => Err(compile_error(
                COMPILE_INVALID_GO_EXPRESSION,
                "`go` requires a function or procedure call",
                "Use `go FunctionName(args)` or `go SomeCallable(args)`.",
                span,
            )),
        }
    }

    fn compile_go_wrapper_call(
        &mut self,
        callee_name: &str,
        arg_exprs: &[Expr],
        returns_value: bool,
        detached: bool,
        span: Span,
    ) -> Result<(), CompileError> {
        for expr in arg_exprs {
            self.compile_expr(expr)?;
        }

        let params = self.go_wrapper_params(arg_exprs.len(), span);
        let call_args = params
            .iter()
            .map(|param| Expr::Designator(self.go_wrapper_param_designator(&param.name, span)))
            .collect::<Vec<_>>();

        let body = if returns_value {
            vec![Stmt::Return(
                Some(Expr::Call {
                    designator: self.designator_from_qualified_name(callee_name, span),
                    args: call_args,
                    span,
                }),
                span,
            )]
        } else {
            vec![Stmt::Call {
                designator: self.designator_from_qualified_name(callee_name, span),
                args: call_args,
                span,
            }]
        };

        self.compile_function_expr(
            &params,
            &FuncBody::Block {
                nested: vec![],
                stmts: body,
            },
            (span.line, span.column),
        )?;
        self.emit_go_spawn(arg_exprs.len() as u8, detached, (span.line, span.column));
        Ok(())
    }

    fn emit_go_spawn(&mut self, argc: u8, detached: bool, location: (u32, u32)) {
        let op = if detached {
            Op::SpawnDetachedTask(argc)
        } else {
            Op::SpawnTask(argc)
        };
        self.emit(op, location);
    }

    fn go_call_returns_value(&self, expr: &Expr) -> bool {
        let key = std::ptr::from_ref(expr) as usize;
        self.expr_types
            .get(&key)
            .is_none_or(|ty| !matches!(ty, Ty::Unit))
    }

    fn go_wrapper_params(&self, count: usize, span: Span) -> Vec<FormalParam> {
        (0..count)
            .map(|index| FormalParam {
                mutable: false,
                name: format!("$go_arg_{index}"),
                type_expr: self.go_wrapper_placeholder_type(span),
                span,
            })
            .collect()
    }

    fn go_wrapper_param_designator(&self, name: &str, span: Span) -> Designator {
        Designator {
            parts: vec![DesignatorPart::Ident(name.to_string(), span)],
            span,
        }
    }

    fn go_wrapper_placeholder_type(&self, span: Span) -> TypeExpr {
        TypeExpr::Named {
            id: QualifiedId {
                parts: vec!["integer".into()],
                span,
            },
            type_args: vec![],
            span,
        }
    }

    fn designator_from_qualified_name(&self, name: &str, span: Span) -> Designator {
        Designator {
            parts: name
                .split('.')
                .map(|part| DesignatorPart::Ident(part.to_string(), span))
                .collect(),
            span,
        }
    }

    /// Compile `select ... end`.
    pub(super) fn compile_select_stmt(
        &mut self,
        arms: &[SelectArm],
        default_body: Option<&[Stmt]>,
        span: Span,
    ) -> Result<(), CompileError> {
        let loc = (span.line, span.column);
        let loop_start = self.chunk.len();
        let mut skip_jumps: Vec<usize> = Vec::new();

        for arm in arms {
            self.compile_expr(&arm.channel)?;
            self.emit(Op::Intrinsic(Intrinsic::ChannelTryRecv as u16), loc);

            self.emit(Op::Dup, loc);
            self.emit(Op::IsOptionSome, loc);
            let skip_arm = self.emit(Op::JumpIfFalse(0), loc);

            self.emit(Op::UnwrapSome, loc);
            self.begin_scope();
            self.add_local(&arm.binding);
            self.compile_stmt(&arm.body)?;
            self.end_scope(loc);
            skip_jumps.push(self.emit(Op::Jump(0), loc));

            let after_arm = self.chunk.len() as u32;
            self.patch_jump(skip_arm, after_arm, loc)?;
            self.emit(Op::Pop, loc);
        }

        if let Some(body) = default_body {
            for stmt in body {
                self.compile_stmt(stmt)?;
            }
        } else if !arms.is_empty() {
            self.emit(Op::Yield, loc);
            self.emit(Op::Jump(loop_start as u32), loc);
        }

        let select_end = self.chunk.len() as u32;
        for jump in skip_jumps {
            self.patch_jump(jump, select_end, loc)?;
        }

        Ok(())
    }
}
