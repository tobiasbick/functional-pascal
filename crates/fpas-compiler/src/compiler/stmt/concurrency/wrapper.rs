use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_lexer::Span;
use fpas_parser::{
    Designator, DesignatorPart, Expr, FormalParam, FuncBody, QualifiedId, Stmt, TypeExpr,
};

impl Compiler {
    pub(super) fn compile_go_wrapper_call(
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

        self.compile_callable_wrapper(
            &params,
            &FuncBody::Block {
                nested: vec![],
                stmts: body,
            },
            span,
        )?;
        self.emit_go_spawn(arg_exprs.len(), detached, span)?;
        Ok(())
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
}
