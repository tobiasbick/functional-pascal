//! Statement rules: bindings, assignments, returns, and nested scopes.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use fpas_parser::{CaseArm, Decl, DesignatorPart, FuncBody, Stmt};

use super::origins::Frame;
use super::walker::Walker;

impl Walker<'_, '_> {
    /// Walk a routine body after registering its nested routines.
    pub(super) fn walk_body(&mut self, body: &FuncBody, nested: Option<usize>) {
        let FuncBody::Block {
            nested: declarations,
            stmts,
        } = body;
        let mut frame = Frame::default();
        if let Some(routine) = nested {
            frame.routines = self.table.routines[routine].nested.clone();
        }
        self.frames.push(frame);
        self.walk_initializers(declarations);
        self.walk_statements(stmts);
        self.frames.pop();
    }

    /// Visit variable and constant initializers, including anonymous routine bodies.
    pub(super) fn walk_initializers(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            let (name, value) = match declaration {
                Decl::Const(binding) => (&binding.name, &binding.value),
                Decl::Var(binding) => (&binding.name, &binding.value),
                _ => continue,
            };
            let owners = self.expr(value);
            self.declare(name, owners);
        }
    }

    /// Visit an ordered statement list.
    pub(super) fn walk_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.walk_statement(statement);
        }
    }

    fn walk_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block(statements, _) => {
                self.in_frame(|walker| walker.walk_statements(statements))
            }
            Stmt::Scope { body, span } => {
                self.scopes.push(*span);
                self.in_frame(|walker| walker.walk_statements(body));
                self.scopes.pop();
            }
            Stmt::Const(binding) | Stmt::Var(binding) => {
                let owners = self.expr(&binding.value);
                self.declare(&binding.name, owners);
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                let owners = self.expr(value);
                for part in &target.parts {
                    if let DesignatorPart::Index(index, _) = part {
                        let _ = self.expr(index);
                    }
                }
                if let Some(DesignatorPart::Ident(name, _)) = target.parts.first() {
                    self.store(name, owners, *span);
                }
            }
            Stmt::Return(value, span) => {
                if let Some(value) = value {
                    let owners = self.expr(value);
                    self.escape_all(&owners, *span, "`return`");
                }
            }
            Stmt::Panic(value, _)
            | Stmt::Expression { expr: value, .. }
            | Stmt::Discard { expr: value, .. } => {
                let _ = self.expr(value);
            }
            Stmt::Call {
                designator,
                args,
                span,
            } => {
                let _ = self.designator_call(
                    crate::designator_lookup_key(designator),
                    designator,
                    args,
                    *span,
                );
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Branch-local declarations remain in their own frame.
                self.in_frame(|walker| {
                    let _ = walker.expr(condition);
                    walker.walk_statement(then_branch);
                });
                if let Some(branch) = else_branch {
                    self.in_frame(|walker| walker.walk_statement(branch));
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                let owners = self.expr(expr);
                for arm in arms {
                    self.walk_case_arm(arm, &owners);
                }
                if let Some(statements) = else_body {
                    self.in_frame(|walker| walker.walk_statements(statements));
                }
            }
            Stmt::For {
                var_name,
                start,
                end,
                body,
                ..
            } => {
                let _ = self.expr(start);
                let _ = self.expr(end);
                self.walk_loop(|walker| {
                    walker.declare(var_name, Vec::new());
                    walker.walk_statement(body);
                });
            }
            Stmt::ForIn {
                var_name,
                iterable,
                body,
                ..
            } => {
                self.walk_loop(|walker| {
                    let owners = walker.expr(iterable);
                    walker.declare(var_name, owners);
                    walker.walk_statement(body);
                });
            }
            Stmt::While {
                condition, body, ..
            } => self.walk_loop(|walker| {
                let _ = walker.expr(condition);
                walker.walk_statement(body);
            }),
            Stmt::Repeat {
                body, condition, ..
            } => self.walk_loop(|walker| {
                walker.walk_statements(body);
                let _ = walker.expr(condition);
            }),
            Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn walk_case_arm(&mut self, arm: &CaseArm, owners: &super::origins::Owners) {
        self.in_frame(|walker| {
            for label in &arm.labels {
                walker.bind_case_label(label, owners);
            }
            if let Some(guard) = &arm.guard {
                let _ = walker.expr(guard);
            }
            walker.walk_statement(&arm.body);
        });
    }
}
