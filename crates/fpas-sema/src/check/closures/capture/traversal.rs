//! Lexical capture traversal through routine bodies.

use super::*;

impl CaptureCollector<'_> {
    fn consider_name(&mut self, name: &str) {
        let canonical = name.to_ascii_lowercase();
        if self
            .bound_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(&canonical))
        {
            return;
        }
        if !self.seen.insert(canonical) {
            return;
        }
        let Some((scope_index, symbol, declaration)) =
            self.scopes.lookup_with_scope_and_declaration(name)
        else {
            return;
        };
        if scope_index == 0 || scope_index >= self.closure_scope_index {
            return;
        }
        if !matches!(
            symbol.kind,
            SymbolKind::Var | SymbolKind::Param | SymbolKind::ForVar
        ) {
            return;
        }
        let Some(declaration) = declaration else {
            return;
        };
        self.captures.push(CaptureBinding {
            name: name.to_string(),
            ty: symbol.ty.clone(),
            mutable: symbol.mutable,
            task_bound: symbol.task_bound,
            declaration,
        });
    }

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
            if let Stmt::Var(var) | Stmt::MutableVar(var) = stmt {
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
            Decl::Var(var) | Decl::MutableVar(var) => {
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
            Stmt::StatementList(statements, _) => self.collect_statement_list(statements),
            Stmt::Block(stmts, _) => {
                self.push_bound_scope();
                self.collect_statement_list(stmts);
                self.pop_bound_scope();
            }
            Stmt::Var(var) | Stmt::MutableVar(var) => self.collect_from_expr(&var.value),
            Stmt::Assign { target, value, .. } => {
                self.collect_from_designator(target);
                self.collect_from_expr(value);
            }
            Stmt::Return(Some(expr), _)
            | Stmt::Panic(expr, _)
            | Stmt::Discard(expr, _)
            | Stmt::Expression { expr, .. }
            | Stmt::Go { expr, .. } => {
                self.collect_from_expr(expr);
            }
            Stmt::Null(_) | Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => {
                self.collect_from_expr(condition);
                self.push_bound_scope();
                self.collect_from_stmt(then_branch);
                self.pop_bound_scope();
                for (condition, body) in elsif_branches {
                    self.collect_from_expr(condition);
                    self.push_bound_scope();
                    self.collect_from_stmt(body);
                    self.pop_bound_scope();
                }
                if let Some(branch) = else_branch {
                    self.push_bound_scope();
                    self.collect_from_stmt(branch);
                    self.pop_bound_scope();
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
                    for pattern in &arm.labels {
                        pattern.visit_expressions(&mut |expr| self.collect_from_expr(expr));
                        pattern.visit_bindings(&mut |name| self.bind_name(name));
                    }
                    if let Some(guard) = &arm.guard {
                        self.collect_from_expr(guard);
                    }
                    self.collect_from_stmt(&arm.body);
                    self.pop_bound_scope();
                }
                if let Some(branch) = else_body {
                    self.collect_statement_list(branch);
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
                self.collect_from_expr(condition);
                self.collect_from_stmt(body);
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

    fn collect_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::If(decision) => {
                self.collect_from_expr(&decision.condition);
                self.collect_from_expr(&decision.then_value);
                for (condition, value) in &decision.elsif_values {
                    self.collect_from_expr(condition);
                    self.collect_from_expr(value);
                }
                self.collect_from_expr(&decision.else_value);
            }
            Expr::Case(decision) => {
                self.collect_from_expr(&decision.value);
                for arm in &decision.arms {
                    self.push_bound_scope();
                    for label in &arm.labels {
                        label.visit_expressions(&mut |value| self.collect_from_expr(value));
                        label.visit_bindings(&mut |name| self.bind_name(name));
                    }
                    if let Some(guard) = &arm.guard {
                        self.collect_from_expr(guard);
                    }
                    self.collect_from_expr(&arm.body);
                    self.pop_bound_scope();
                }
                if let Some(value) = &decision.else_value {
                    self.collect_from_expr(value);
                }
            }
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Nil(_)
            | Expr::InvalidRecord(..)
            | Expr::Error(_) => {}
            Expr::Designator(designator) => self.collect_from_designator(designator),
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
            | Expr::Go(operand, _) => self.collect_from_expr(operand),
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
            Expr::RecordConstruction { fields, .. } => {
                for field in fields {
                    self.collect_from_expr(&field.value);
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
                        PostfixOperation::MethodCall { args, .. }
                        | PostfixOperation::Call { args, .. } => {
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
