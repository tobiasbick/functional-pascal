use super::super::NameRewriter;
use fpas_parser::{CaseArm, CaseLabel, Stmt};

impl NameRewriter<'_> {
    pub(in super::super) fn rewrite_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Block(stmts, _) => {
                self.push_scope();
                self.rewrite_statements(stmts);
                self.pop_scope();
            }
            Stmt::Var(var_def) | Stmt::MutableVar(var_def) => {
                self.rewrite_var_def(var_def);
                self.declare_value(&var_def.name);
            }
            Stmt::Assign { target, value, .. } => {
                self.rewrite_designator(target);
                self.rewrite_expr(value);
            }
            Stmt::Return(value, _) => {
                if let Some(value) = value {
                    self.rewrite_expr(value);
                }
            }
            Stmt::Panic(expr, _) => self.rewrite_expr(expr),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.rewrite_expr(condition);
                self.rewrite_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.rewrite_stmt(else_branch);
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.rewrite_expr(expr);
                for arm in arms {
                    self.rewrite_case_arm(arm);
                }
                if let Some(else_body) = else_body {
                    self.push_scope();
                    self.rewrite_statements(else_body);
                    self.pop_scope();
                }
            }
            Stmt::For {
                var_name,
                var_type,
                start,
                direction: _,
                end,
                body,
                ..
            } => {
                self.rewrite_type_expr(var_type);
                self.rewrite_expr(start);
                self.rewrite_expr(end);
                self.push_scope();
                self.declare_value(var_name);
                self.rewrite_stmt(body);
                self.pop_scope();
            }
            Stmt::ForIn {
                var_name,
                var_type,
                iterable,
                body,
                ..
            } => {
                self.rewrite_type_expr(var_type);
                self.rewrite_expr(iterable);
                self.push_scope();
                self.declare_value(var_name);
                self.rewrite_stmt(body);
                self.pop_scope();
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.rewrite_expr(condition);
                self.rewrite_stmt(body);
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.push_scope();
                self.rewrite_statements(body);
                self.pop_scope();
                self.rewrite_expr(condition);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Call {
                designator, args, ..
            } => {
                self.rewrite_designator(designator);
                for arg in args {
                    self.rewrite_expr(arg);
                }
            }
            Stmt::Go { expr, .. } => self.rewrite_expr(expr),
        }
    }

    fn rewrite_case_arm(&mut self, arm: &mut CaseArm) {
        let bindings = Self::case_arm_bindings(arm);
        for label in &mut arm.labels {
            self.rewrite_case_label(label);
        }

        let opened_scope = !bindings.is_empty();
        if opened_scope {
            self.push_scope();
            for binding in bindings {
                self.declare_value(&binding);
            }
        }

        if let Some(guard) = &mut arm.guard {
            self.rewrite_expr(guard);
        }
        self.rewrite_stmt(&mut arm.body);

        if opened_scope {
            self.pop_scope();
        }
    }

    fn rewrite_case_label(&mut self, label: &mut CaseLabel) {
        match label {
            CaseLabel::Value { start, end, .. } => {
                self.rewrite_case_pattern_expr(start, false);
                if let Some(end) = end {
                    self.rewrite_expr(end);
                }
            }
            CaseLabel::Destructure { .. } => {}
        }
    }

}
