//! Statement traversal preserves source evaluation sites without inventing implicit calls.

use super::*;
use fpas_parser::{CaseLabel, Designator, DesignatorPart, Stmt};

impl Walker<'_> {
    pub(super) fn statements(
        &mut self,
        statements: &[Stmt],
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        for statement in statements {
            self.statement(statement, owner)?;
        }
        Ok(())
    }
    fn statement(
        &mut self,
        statement: &Stmt,
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        self.cancellation.check()?;
        match statement {
            Stmt::Block(body, _) | Stmt::Scope { body, .. } => self.statements(body, owner)?,
            Stmt::Const(binding) | Stmt::Var(binding) => self.ordinary(&binding.value, owner)?,
            Stmt::Assign { target, value, .. } => {
                self.indices(target, owner)?;
                self.ordinary(value, owner)?;
            }
            Stmt::Return(value, _) => {
                if let Some(value) = value {
                    self.ordinary(value, owner)?;
                }
            }
            Stmt::Panic(value, _)
            | Stmt::Expression { expr: value, .. }
            | Stmt::Discard { expr: value, .. } => self.ordinary(value, owner)?,
            Stmt::Call {
                designator,
                args,
                span,
            } => {
                self.indices(designator, owner)?;
                self.call(
                    fpas_sema::designator_lookup_key(designator),
                    Some(designator),
                    args,
                    *span,
                    owner,
                    InvocationKind::Ordinary,
                )?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.ordinary(condition, owner)?;
                self.statement(then_branch, owner)?;
                if let Some(body) = else_branch {
                    self.statement(body, owner)?;
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.ordinary(expr, owner)?;
                for arm in arms {
                    self.labels(&arm.labels, owner)?;
                    if let Some(guard) = &arm.guard {
                        self.ordinary(guard, owner)?;
                    }
                    self.statement(&arm.body, owner)?;
                }
                if let Some(body) = else_body {
                    self.statements(body, owner)?;
                }
            }
            Stmt::For {
                start, end, body, ..
            } => {
                self.ordinary(start, owner)?;
                self.ordinary(end, owner)?;
                self.statement(body, owner)?;
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.ordinary(iterable, owner)?;
                self.statement(body, owner)?;
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.ordinary(condition, owner)?;
                self.statement(body, owner)?;
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.statements(body, owner)?;
                self.ordinary(condition, owner)?;
            }
            Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
        Ok(())
    }
    pub(super) fn indices(
        &mut self,
        designator: &Designator,
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        for part in &designator.parts {
            if let DesignatorPart::Index(value, _) = part {
                self.ordinary(value, owner)?;
            }
        }
        Ok(())
    }
    pub(super) fn labels(
        &mut self,
        labels: &[CaseLabel],
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        for label in labels {
            if let CaseLabel::Value { start, end, .. } = label {
                self.ordinary(start, owner)?;
                if let Some(end) = end {
                    self.ordinary(end, owner)?;
                }
            }
        }
        Ok(())
    }
}
