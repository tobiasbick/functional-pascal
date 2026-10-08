//! Statement and expression traversal for lexical captures.

use super::{CaptureBinding, CaptureCollector};
use fpas_parser::{
    CaseLabel, Decl, Designator, DesignatorPart, Expr, FuncBody, Pattern, PostfixOperation, Stmt,
};
use std::collections::HashSet;

impl CaptureCollector<'_> {
    fn push_bound_scope(&mut self) {
        self.bound_scopes.push(HashSet::new());
    }

    fn pop_bound_scope(&mut self) {
        self.bound_scopes.pop();
    }

    fn bind_name(&mut self, name: &str) {
        if let Some(scope) = self.bound_scopes.last_mut() {
            scope.insert(name.to_ascii_lowercase());
        }
    }

    fn collect_statement_list(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.collect_from_stmt(stmt);
            if let Stmt::Const(var) | Stmt::Var(var) = stmt {
                self.bind_name(&var.name);
            }
        }
    }

    fn collect_transitive_captures(&mut self, captures: &[CaptureBinding]) {
        for capture in captures {
            self.consider_name(&capture.name);
        }
    }

    fn collect_from_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Var(var) => {
                self.collect_from_expr(&var.value);
            }
            Decl::Const(var) => {
                self.collect_from_expr(&var.value);
            }
            Decl::Function(function) => {
                let captures = self
                    .nested_routine_captures
                    .get(&crate::function_decl_lookup_key(function))
                    .map(|info| info.captures.clone())
                    .unwrap_or_default();
                self.collect_transitive_captures(&captures);
            }
            Decl::Procedure(procedure) => {
                let captures = self
                    .nested_routine_captures
                    .get(&crate::procedure_decl_lookup_key(procedure))
                    .map(|info| info.captures.clone())
                    .unwrap_or_default();
                self.collect_transitive_captures(&captures);
            }
            Decl::TypeDef(_) => {}
        }
    }

    /// Walks nested routine captures and ordered local declarations in a body.
    pub(super) fn collect_from_body(&mut self, body: &FuncBody) {
        let FuncBody::Block { nested, stmts } = body;
        for decl in nested {
            self.collect_from_decl(decl);
        }
        self.push_bound_scope();
        self.collect_statement_list(stmts);
        self.pop_bound_scope();
    }

    fn collect_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts, _) => {
                self.push_bound_scope();
                self.collect_statement_list(stmts);
                self.pop_bound_scope();
            }
            Stmt::Const(var) | Stmt::Var(var) => self.collect_from_expr(&var.value),
            Stmt::Assign { target, value, .. } => {
                self.collect_from_designator(target);
                self.collect_from_expr(value);
            }
            Stmt::Return(Some(expr), _)
            | Stmt::Panic(expr, _)
            | Stmt::Expression { expr, .. }
            | Stmt::Go { expr, .. }
            | Stmt::Discard { expr, .. } => {
                self.collect_from_expr(expr);
            }
            Stmt::Return(None, _) | Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // `is` bindings in the condition are visible only in the then-branch.
                self.push_bound_scope();
                self.collect_from_expr(condition);
                self.collect_from_stmt(then_branch);
                self.pop_bound_scope();
                if let Some(branch) = else_branch {
                    self.collect_from_stmt(branch);
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.collect_from_expr(expr);
                for arm in arms {
                    self.push_bound_scope();
                    for label in &arm.labels {
                        match label {
                            CaseLabel::Value { start, end, .. } => {
                                self.collect_from_expr(start);
                                if let Some(end) = end {
                                    self.collect_from_expr(end);
                                }
                            }
                            CaseLabel::Binding { .. } => {}
                            CaseLabel::Pattern(pattern) => self.collect_pattern_values(pattern),
                        }
                    }
                    // Comparisons in every label resolve before the arm's names are bound.
                    for label in &arm.labels {
                        match label {
                            CaseLabel::Binding { name, .. } => self.bind_name(name),
                            CaseLabel::Pattern(pattern) => self.bind_pattern(pattern),
                            CaseLabel::Value { .. } => {}
                        }
                    }
                    if let Some(guard) = &arm.guard {
                        self.collect_from_expr(guard);
                    }
                    self.collect_from_stmt(&arm.body);
                    self.pop_bound_scope();
                }
                if let Some(branch) = else_body {
                    self.push_bound_scope();
                    self.collect_statement_list(branch);
                    self.pop_bound_scope();
                }
            }
            Stmt::For {
                var_name,
                start,
                end,
                body,
                ..
            } => {
                self.collect_from_expr(start);
                self.collect_from_expr(end);
                self.push_bound_scope();
                self.bind_name(var_name);
                self.collect_from_stmt(body);
                self.pop_bound_scope();
            }
            Stmt::ForIn {
                var_name,
                iterable,
                body,
                ..
            } => {
                self.collect_from_expr(iterable);
                self.push_bound_scope();
                self.bind_name(var_name);
                self.collect_from_stmt(body);
                self.pop_bound_scope();
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.push_bound_scope();
                self.collect_from_expr(condition);
                self.collect_from_stmt(body);
                self.pop_bound_scope();
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.push_bound_scope();
                self.collect_statement_list(body);
                self.pop_bound_scope();
                self.collect_from_expr(condition);
            }
            Stmt::Call {
                designator, args, ..
            } => {
                self.collect_from_designator(designator);
                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
        }
    }

    fn collect_pattern_values(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Value(expr) => self.collect_from_expr(expr),
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    self.collect_pattern_values(&field.pattern);
                }
            }
            Pattern::Destructure {
                payload: Some(payload),
                ..
            } => {
                self.collect_pattern_values(payload);
            }
            Pattern::Binding { .. } | Pattern::Wildcard(_) | Pattern::Destructure { .. } => {}
        }
    }

    /// Binds names after the pattern's comparison values have been visited.
    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { name, .. } => self.bind_name(name),
            Pattern::Wildcard(_) => {}
            Pattern::Value(_) => {}
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    self.bind_pattern(&field.pattern);
                }
            }
            Pattern::Destructure { payload, .. } => {
                if let Some(payload) = payload {
                    self.bind_pattern(payload);
                }
            }
        }
    }

    fn collect_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Nil(_)
            | Expr::Error(_) => {}
            Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
                self.collect_from_designator(designator);
            }
            Expr::Call {
                designator, args, ..
            } => {
                self.collect_from_designator(designator);
                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
            Expr::UnaryOp { operand, .. }
            | Expr::Paren(operand, _)
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _)
            | Expr::Try(operand, _)
            | Expr::Go(operand, _)
            | Expr::NamedArgument { value: operand, .. } => self.collect_from_expr(operand),
            Expr::Is { value, pattern, .. } => {
                self.collect_from_expr(value);
                self.collect_pattern_values(pattern);
                self.bind_pattern(pattern);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.collect_from_expr(left);
                self.collect_from_expr(right);
            }
            Expr::ArrayLiteral(elements, _) => {
                for element in elements {
                    self.collect_from_expr(element);
                }
            }
            Expr::DictLiteral(pairs, _) => {
                for (key, value) in pairs {
                    self.collect_from_expr(key);
                    self.collect_from_expr(value);
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.collect_from_expr(base);
                for field in fields {
                    self.collect_from_expr(&field.value);
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.collect_from_expr(base);
                for operation in operations {
                    match operation {
                        PostfixOperation::Field { .. } => {}
                        PostfixOperation::Index { index, .. } => self.collect_from_expr(index),
                        PostfixOperation::MethodCall { args, .. } => {
                            for arg in args {
                                self.collect_from_expr(arg);
                            }
                        }
                    }
                }
            }
            Expr::Closure(_) => {
                let captures = self
                    .closure_infos
                    .get(&crate::expr_lookup_key(expr))
                    .map(|info| info.captures.clone())
                    .unwrap_or_default();
                self.collect_transitive_captures(&captures);
            }
        }
    }

    fn collect_from_designator(&mut self, designator: &Designator) {
        if let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() {
            self.consider_name(name);
        }
        for part in &designator.parts {
            if let DesignatorPart::Index(index, _) = part {
                self.collect_from_expr(index);
            }
        }
    }
}
