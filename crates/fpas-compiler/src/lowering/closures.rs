//! Anonymous-closure discovery, capture typing, and body lowering.

mod bound_methods;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{CaptureKind, Function, FunctionId};
use fpas_parser::{CaseLabel, DesignatorPart, Expr, FuncBody, PostfixOperation, Stmt};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::context::{
    BoundMethodTarget, Callable, CaptureInput, ClosureTarget, FunctionInput, LoweringContext,
    ParameterInput, unsupported,
};
use super::types;

pub(super) struct ClosureRoutine<'a> {
    expression: &'a Expr,
    pub id: FunctionId,
    name: String,
    captures: Vec<CaptureInput>,
    pub owner: FunctionId,
}

pub(super) struct BoundMethodRoutine {
    pub id: FunctionId,
    name: String,
    target: FunctionId,
    receiver_ty: fpas_ir::TypeId,
    parameters: Vec<fpas_ir::TypeId>,
    result: fpas_ir::TypeId,
    span: fpas_lexer::Span,
    pub owner: FunctionId,
}

pub(super) struct ClosureRegistry<'a> {
    pub routines: Vec<ClosureRoutine<'a>>,
    pub targets: HashMap<usize, ClosureTarget>,
    pub bound_routines: Vec<BoundMethodRoutine>,
    pub bound_targets: HashMap<usize, BoundMethodTarget>,
    pub cell_names: HashMap<FunctionId, BTreeSet<String>>,
    callables: BTreeMap<String, Callable>,
    next_id: u32,
}

impl<'a> ClosureRegistry<'a> {
    pub fn new(first_id: u32, callables: BTreeMap<String, Callable>) -> Self {
        Self {
            routines: Vec::new(),
            targets: HashMap::new(),
            bound_routines: Vec::new(),
            bound_targets: HashMap::new(),
            cell_names: HashMap::new(),
            callables,
            next_id: first_id,
        }
    }

    /// Mark owner locals that named nested routines capture as cells.
    ///
    /// Anonymous closures already record this during discovery. Named nested captures
    /// come from the callable table and must use the same MakeCell lowering.
    pub fn seed_named_nested_cells(
        &mut self,
        owners: &[FunctionId],
        runtime_names: &[String],
        callables: &BTreeMap<String, Callable>,
    ) {
        for (index, owner) in owners.iter().copied().enumerate() {
            let Some(name) = runtime_names.get(index) else {
                continue;
            };
            let Some(callable) = callables.get(&name.to_ascii_lowercase()) else {
                continue;
            };
            for capture in &callable.captures {
                if capture.kind == CaptureKind::Value {
                    continue;
                }
                self.cell_names
                    .entry(owner)
                    .or_default()
                    .insert(capture.name.to_ascii_lowercase());
            }
        }
    }

    pub fn discover_statements(
        &mut self,
        statements: &'a [Stmt],
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        for statement in statements {
            self.visit_statement(statement, owner, metadata, types)?;
        }
        Ok(())
    }

    pub fn lower(
        &self,
        routine: &ClosureRoutine<'a>,
        metadata: &AnalysisMetadata,
        callables: &BTreeMap<String, Callable>,
        types: &mut types::TypeTable,
        globals: &BTreeMap<String, super::context::GlobalBinding>,
        constants: &BTreeMap<String, fpas_ir::Constant>,
    ) -> Result<(Function, types::TypeTable), CompileError> {
        let Expr::Closure(closure) = routine.expression else {
            return Err(unsupported(
                routine.expression.span(),
                "closure registry entry",
            ));
        };
        let parameters = closure
            .params
            .iter()
            .map(|parameter| {
                types
                    .type_expr(&parameter.type_expr)
                    .map(|ty| ParameterInput {
                        name: parameter.name.clone(),
                        ty,
                        declaration: Some(parameter.span.diagnostic_span_or_synthetic()),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = closure
            .return_type
            .as_ref()
            .map(|result| types.type_expr(result))
            .transpose()?
            .unwrap_or(types::UNIT);
        let mut context = LoweringContext::new(FunctionInput {
            name: &routine.name,
            id: routine.id,
            result,
            parameters: &parameters,
            captures: &routine.captures,
            globals: globals.clone(),
            constants: constants.clone(),
            metadata,
            callables: callables.clone(),
            closure_targets: self.targets.clone(),
            bound_method_targets: self.bound_targets.clone(),
            cell_names: self
                .cell_names
                .get(&routine.id)
                .cloned()
                .unwrap_or_default(),
            type_table: types.clone(),
        })?;
        let FuncBody::Block { stmts, .. } = &closure.body;
        context.lower_statements(stmts)?;
        context.finish(closure.span)
    }

    fn visit_statement(
        &mut self,
        statement: &'a Stmt,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        match statement {
            Stmt::Block(statements, _)
            | Stmt::Repeat {
                body: statements, ..
            } => {
                self.discover_statements(statements, owner, metadata, types)?;
            }
            Stmt::Var(definition) | Stmt::MutableVar(definition) => {
                self.visit_expression(&definition.value, owner, metadata, types)?;
            }
            Stmt::Assign { target, value, .. } => {
                self.visit_designator(target.parts.as_slice(), owner, metadata, types)?;
                self.visit_expression(value, owner, metadata, types)?;
            }
            Stmt::Return(value, _) => {
                if let Some(value) = value {
                    self.visit_expression(value, owner, metadata, types)?;
                }
            }
            Stmt::Panic(value, _)
            | Stmt::Expression { expr: value, .. }
            | Stmt::Go { expr: value, .. } => {
                self.visit_expression(value, owner, metadata, types)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit_expression(condition, owner, metadata, types)?;
                self.visit_statement(then_branch, owner, metadata, types)?;
                if let Some(branch) = else_branch {
                    self.visit_statement(branch, owner, metadata, types)?;
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.visit_expression(expr, owner, metadata, types)?;
                for arm in arms {
                    for label in &arm.labels {
                        if let CaseLabel::Value { start, end, .. } = label {
                            self.visit_expression(start, owner, metadata, types)?;
                            if let Some(end) = end {
                                self.visit_expression(end, owner, metadata, types)?;
                            }
                        }
                    }
                    if let Some(guard) = &arm.guard {
                        self.visit_expression(guard, owner, metadata, types)?;
                    }
                    self.visit_statement(&arm.body, owner, metadata, types)?;
                }
                if let Some(statements) = else_body {
                    self.discover_statements(statements, owner, metadata, types)?;
                }
            }
            Stmt::For {
                start, end, body, ..
            } => {
                self.visit_expression(start, owner, metadata, types)?;
                self.visit_expression(end, owner, metadata, types)?;
                self.visit_statement(body, owner, metadata, types)?;
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.visit_expression(iterable, owner, metadata, types)?;
                self.visit_statement(body, owner, metadata, types)?;
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.visit_expression(condition, owner, metadata, types)?;
                self.visit_statement(body, owner, metadata, types)?;
            }
            Stmt::Call {
                designator, args, ..
            } => {
                self.visit_designator(&designator.parts, owner, metadata, types)?;
                for argument in args {
                    self.visit_expression(argument, owner, metadata, types)?;
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
        }
        Ok(())
    }

    fn visit_expression(
        &mut self,
        expression: &'a Expr,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        match expression {
            Expr::Closure(closure) => {
                let key = fpas_sema::expr_lookup_key(expression);
                let info = metadata.closure_infos.get(&key).ok_or_else(|| {
                    unsupported(closure.span, "closure without semantic capture metadata")
                })?;
                let id = FunctionId::new(self.next_id);
                self.next_id = self
                    .next_id
                    .checked_add(1)
                    .ok_or_else(|| unsupported(closure.span, "function identifier overflow"))?;
                let value_type = types.id(
                    metadata.expr_types.get(&key).ok_or_else(|| {
                        unsupported(closure.span, "closure without semantic type")
                    })?,
                    closure.span.line,
                    closure.span.column,
                )?;
                let captures = info
                    .captures
                    .iter()
                    .map(|capture| {
                        let ty =
                            types.intern(&capture.ty, closure.span.line, closure.span.column)?;
                        let reuses_cell = self
                            .routines
                            .iter()
                            .find(|routine| routine.id == owner)
                            .is_some_and(|routine| {
                                routine.captures.iter().any(|outer| {
                                    outer.name.eq_ignore_ascii_case(&capture.name)
                                        && outer.kind != CaptureKind::Value
                                })
                            });
                        let kind = if reuses_cell {
                            CaptureKind::EnclosingCell
                        } else if capture.mutable {
                            CaptureKind::Cell
                        } else {
                            CaptureKind::Value
                        };
                        let storage_ty = if capture.mutable {
                            types.cell_type(ty, closure.span)?
                        } else {
                            ty
                        };
                        Ok(CaptureInput {
                            name: capture.name.clone(),
                            ty,
                            storage_ty,
                            kind,
                            declaration: Some(capture.declaration.diagnostic_span_or_synthetic()),
                        })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?;
                for capture in &captures {
                    if capture.kind != CaptureKind::Value {
                        self.cell_names
                            .entry(owner)
                            .or_default()
                            .insert(capture.name.to_ascii_lowercase());
                    }
                }
                self.targets.insert(
                    key,
                    ClosureTarget {
                        function: id,
                        value_type,
                        captures: captures.clone(),
                    },
                );
                self.routines.push(ClosureRoutine {
                    expression,
                    id,
                    name: info.synthetic_name.clone(),
                    captures,
                    owner,
                });
                let FuncBody::Block { stmts, .. } = &closure.body;
                self.discover_statements(stmts, id, metadata, types)?;
            }
            Expr::Designator(designator) => {
                let key = fpas_sema::designator_lookup_key(designator);
                if let Some(info) = metadata.bound_methods.get(&key) {
                    self.register_bound_method(key, info, owner, designator.span, types)?;
                }
                self.visit_designator(&designator.parts, owner, metadata, types)?
            }
            Expr::Call {
                designator, args, ..
            } => {
                self.visit_designator(&designator.parts, owner, metadata, types)?;
                for argument in args {
                    self.visit_expression(argument, owner, metadata, types)?;
                }
            }
            Expr::UnaryOp { operand, .. }
            | Expr::Paren(operand, _)
            | Expr::Try(operand, _)
            | Expr::Go(operand, _)
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _) => {
                self.visit_expression(operand, owner, metadata, types)?;
            }
            Expr::BinaryOp { left, right, .. } => {
                self.visit_expression(left, owner, metadata, types)?;
                self.visit_expression(right, owner, metadata, types)?;
            }
            Expr::ArrayLiteral(values, _) => {
                for value in values {
                    self.visit_expression(value, owner, metadata, types)?;
                }
            }
            Expr::DictLiteral(values, _) => {
                for (key, value) in values {
                    self.visit_expression(key, owner, metadata, types)?;
                    self.visit_expression(value, owner, metadata, types)?;
                }
            }
            Expr::RecordLiteral { fields, .. } => {
                for field in fields {
                    self.visit_expression(&field.value, owner, metadata, types)?;
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.visit_expression(base, owner, metadata, types)?;
                for field in fields {
                    self.visit_expression(&field.value, owner, metadata, types)?;
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.visit_expression(base, owner, metadata, types)?;
                for operation in operations {
                    match operation {
                        PostfixOperation::Index { index, .. } => {
                            self.visit_expression(index, owner, metadata, types)?
                        }
                        PostfixOperation::MethodCall { args, .. } => {
                            for argument in args {
                                self.visit_expression(argument, owner, metadata, types)?;
                            }
                        }
                        PostfixOperation::Field { span, .. } => {
                            let key = fpas_sema::postfix_operation_lookup_key(operation);
                            if let Some(info) = metadata.bound_methods.get(&key) {
                                self.register_bound_method(key, info, owner, *span, types)?;
                            }
                        }
                    }
                }
            }
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Nil(_)
            | Expr::Error(_) => {}
        }
        Ok(())
    }

    fn visit_designator(
        &mut self,
        parts: &'a [DesignatorPart],
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        for part in parts {
            if let DesignatorPart::Index(index, _) = part {
                self.visit_expression(index, owner, metadata, types)?;
            }
        }
        Ok(())
    }
}
