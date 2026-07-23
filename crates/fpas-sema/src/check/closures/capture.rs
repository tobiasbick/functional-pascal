//! Free-variable capture analysis for closures.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use std::collections::HashSet;

use crate::scope::{ScopeStack, SymbolKind, canonical_symbol_name};
use fpas_parser::{
    CaseLabel, Decl, Designator, DesignatorPart, Expr, FuncBody, PostfixOperation, Stmt,
};

use super::{ClosureInfoMap, NestedRoutineCaptureMap};

/// A free variable captured by a closure from an enclosing scope.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureBinding {
    /// Source name of the captured binding.
    pub name: String,
    /// `true` when the capture is mutable (cell-backed at runtime).
    pub mutable: bool,
    /// `true` when the captured binding already holds a task-bound value
    /// (for example a nested closure that captured a mutable cell).
    pub task_bound: bool,
}

/// Collect lexical captures referenced by `body`.
///
/// A name is captured when it resolves to a `Var`, `Param`, or `ForVar` in a non-root scope
/// outside the closure's own scope frame (`closure_scope_index`). Captures required only by a
/// nested closure or named routine are propagated from that routine's analyzed metadata, which
/// preserves its own parameter and local shadowing.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[must_use]
pub fn collect_captures(
    scopes: &ScopeStack,
    closure_scope_index: usize,
    body: &FuncBody,
    closure_infos: &ClosureInfoMap,
    nested_routine_captures: &NestedRoutineCaptureMap,
) -> Vec<CaptureBinding> {
    let mut collector = CaptureCollector {
        scopes,
        closure_scope_index,
        closure_infos,
        nested_routine_captures,
        captures: Vec::new(),
        seen: HashSet::new(),
        bound_scopes: Vec::new(),
    };
    collector.collect_from_body(body);
    collector.captures
}

struct CaptureCollector<'a> {
    scopes: &'a ScopeStack,
    closure_scope_index: usize,
    closure_infos: &'a ClosureInfoMap,
    nested_routine_captures: &'a NestedRoutineCaptureMap,
    captures: Vec<CaptureBinding>,
    seen: HashSet<String>,
    bound_scopes: Vec<HashSet<String>>,
}

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
        let Some((scope_index, symbol)) = self.scopes.lookup_with_scope(name) else {
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
        self.captures.push(CaptureBinding {
            name: name.to_string(),
            mutable: symbol.mutable,
            task_bound: symbol.task_bound,
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
                    .get(&canonical_symbol_name(&function.name))
                    .map(|info| info.captures.clone())
                    .unwrap_or_default();
                self.collect_transitive_captures(&captures);
            }
            Decl::Procedure(procedure) => {
                let captures = self
                    .nested_routine_captures
                    .get(&canonical_symbol_name(&procedure.name))
                    .map(|info| info.captures.clone())
                    .unwrap_or_default();
                self.collect_transitive_captures(&captures);
            }
            Decl::TypeDef(_) => {}
        }
    }

    fn collect_from_body(&mut self, body: &FuncBody) {
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
            Stmt::Var(var) | Stmt::MutableVar(var) => self.collect_from_expr(&var.value),
            Stmt::Assign { target, value, .. } => {
                self.collect_from_designator(target);
                self.collect_from_expr(value);
            }
            Stmt::Return(Some(expr), _)
            | Stmt::Panic(expr, _)
            | Stmt::Expression { expr, .. }
            | Stmt::Go { expr, .. } => {
                self.collect_from_expr(expr);
            }
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_from_expr(condition);
                self.collect_from_stmt(then_branch);
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
                    // Mirror `check_case_stmt` scalar-guard bindings so a bare
                    // designator label with a guard does not capture a shadowed
                    // outer variable of the same name.
                    self.bind_scalar_guard_name(&arm.labels, &arm.guard);
                    for label in &arm.labels {
                        match label {
                            CaseLabel::Value { start, end, .. } => {
                                self.bind_pattern_names(start);
                                self.collect_from_expr(start);
                                if let Some(end) = end {
                                    self.collect_from_expr(end);
                                }
                            }
                            CaseLabel::Destructure { binding, .. } => {
                                if let Some(binding) = binding {
                                    self.bind_name(binding);
                                }
                            }
                        }
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

    fn bind_pattern_names(&mut self, expr: &Expr) {
        let Expr::Call { args, .. } = expr else {
            return;
        };
        for arg in args {
            if let Expr::Designator(designator) = arg
                && let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice()
            {
                self.bind_name(name);
            }
        }
    }

    /// Bind a scalar `case` guard label (`m if m > 0`) the same way type checking does.
    ///
    /// Without this, a bare designator label is walked as a free reference and can
    /// spuriously capture an outer variable that the arm actually shadows.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`,
    /// `docs/pascal/language/pattern-matching/guards.md`
    fn bind_scalar_guard_name(&mut self, labels: &[CaseLabel], guard: &Option<Expr>) {
        if guard.is_none() || labels.len() != 1 {
            return;
        }
        let CaseLabel::Value {
            start, end: None, ..
        } = &labels[0]
        else {
            return;
        };
        let Expr::Designator(designator) = start else {
            return;
        };
        if designator.parts.len() != 1 {
            return;
        }
        let DesignatorPart::Ident(name, _) = &designator.parts[0] else {
            return;
        };
        if name == "_" {
            return;
        }
        match self.scopes.lookup(name) {
            Some(symbol) if matches!(symbol.kind, SymbolKind::Const | SymbolKind::EnumMember) => {}
            _ => self.bind_name(name),
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
            Expr::RecordLiteral { fields, .. } => {
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
