//! Lexical resource registration and ordered exits shared with task scopes.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use fpas_ir::{Operation, ValueId};
use fpas_lexer::Span;
use fpas_parser::{DesignatorPart, Expr, LocalBinding, PostfixOperation, Stmt};

use super::context::{LoweringContext, unsupported};
use crate::CompileError;

#[derive(Clone, Copy)]
pub(super) enum ExitScope {
    Resources,
    Tasks,
}

#[derive(Clone, Copy)]
pub(in crate::lowering) enum ProducerKind {
    Call,
    Record,
    RecordUpdate,
    Local,
    Global,
    Cell,
    Reference,
    Field,
    Index,
}

#[derive(Clone, Copy)]
pub(in crate::lowering) struct ResourceProducer {
    span: Span,
    kind: ProducerKind,
}

impl ResourceProducer {
    pub(in crate::lowering) fn matches(&self, span: Span, operation: &Operation) -> bool {
        self.span == span
            && matches!(
                (self.kind, operation),
                (
                    ProducerKind::Call,
                    Operation::CallDirect { .. }
                        | Operation::CallValue { .. }
                        | Operation::Intrinsic { .. }
                ) | (ProducerKind::Record, Operation::MakeRecord { .. })
                    | (ProducerKind::RecordUpdate, Operation::UpdateRecord { .. })
                    | (ProducerKind::Local, Operation::ReadLocal(_))
                    | (ProducerKind::Global, Operation::LoadGlobal(_))
                    | (ProducerKind::Cell, Operation::CellRead(_))
                    | (ProducerKind::Reference, Operation::ReferenceRead(_))
                    | (ProducerKind::Field, Operation::LoadField { .. })
                    | (ProducerKind::Index, Operation::IndexGet { .. })
            )
    }
}

impl LoweringContext {
    pub(super) fn enter_resource_list(
        &mut self,
        statements: &[Stmt],
    ) -> Result<bool, CompileError> {
        let Some(binding) = statements.iter().find_map(|statement| match statement {
            Stmt::Const(binding) if binding.using => Some(binding),
            _ => None,
        }) else {
            return Ok(false);
        };
        self.emit_effect(Operation::EnterCleanupScope, binding.span)?;
        self.exit_scopes.push(ExitScope::Resources);
        Ok(true)
    }

    pub(super) fn finish_resource_list(
        &mut self,
        entered: bool,
        span: Span,
    ) -> Result<(), CompileError> {
        if entered {
            if !self.is_terminated() {
                self.emit_effect(
                    Operation::ExitCleanupScope {
                        outcome: None,
                        propagating: false,
                    },
                    span,
                )?;
            }
            self.exit_scopes.pop();
        }
        Ok(())
    }

    pub(in crate::lowering) fn leave_exit_scopes(
        &mut self,
        depth: usize,
        outcome: Option<ValueId>,
        span: Span,
    ) -> Result<(), CompileError> {
        for index in (depth..self.exit_scopes.len()).rev() {
            match self.exit_scopes[index] {
                ExitScope::Resources => self.emit_effect(
                    Operation::ExitCleanupScope {
                        outcome,
                        propagating: outcome.is_some(),
                    },
                    span,
                )?,
                ExitScope::Tasks => self.emit_effect(Operation::CancelTaskScope, span)?,
            }
        }
        Ok(())
    }

    pub(super) fn prepare_resource_init(
        &mut self,
        expression: &Expr,
        span: Span,
    ) -> Result<(), CompileError> {
        self.emit_effect(Operation::BeginResourceInit, span)?;
        self.collect_resource_producers(expression)?;
        Ok(())
    }

    pub(super) fn register_resource(
        &mut self,
        binding: &LocalBinding,
        resource: ValueId,
    ) -> Result<(), CompileError> {
        let info = self
            .resource_releases
            .get(&(binding.span.source_id, binding.span.offset))
            .cloned()
            .ok_or_else(|| unsupported(binding.span, "resource release contract"))?;
        let (function, value_type, captures) = if info.native {
            let target = self
                .intrinsic_targets
                .get(&binding.span.offset)
                .ok_or_else(|| unsupported(binding.span, "native resource release wrapper"))?;
            (target.function, target.value_type, Vec::new())
        } else {
            let callable = self
                .resolve_callable(&info.name)
                .ok_or_else(|| unsupported(binding.span, "source resource release callable"))?;
            let captures = callable
                .captures
                .iter()
                .map(|capture| self.read_capture(capture, binding.span))
                .collect::<Result<Vec<_>, _>>()?;
            (callable.function, callable.value_type, captures)
        };
        self.record_call_arguments(captures.len().max(1), binding.span)?;
        let release = self.emit_value(
            Operation::MakeClosure { function, captures },
            value_type,
            binding.span,
        )?;
        let family = match info.name.to_ascii_lowercase().as_str() {
            "std.io.fs.close" => fpas_code::CleanupFamily::File,
            "std.net.sockets.close" => fpas_code::CleanupFamily::Socket,
            "std.net.sockets.closelistener" => fpas_code::CleanupFamily::Listener,
            "std.os.proc.close" => fpas_code::CleanupFamily::Process,
            "std.ai.acp.release" => fpas_code::CleanupFamily::Acp,
            "std.ai.mcp.release" => fpas_code::CleanupFamily::Mcp,
            "std.ai.mcp.closeendpoint" => fpas_code::CleanupFamily::McpEndpoint,
            "std.net.http.closestream" => fpas_code::CleanupFamily::HttpStream,
            _ => fpas_code::CleanupFamily::Custom,
        }
        .to_u16();
        self.resource_producers.clear();
        self.emit_effect(
            Operation::RegisterCleanup {
                resource,
                release,
                family,
            },
            binding.span,
        )
    }
    fn collect_resource_producers(&mut self, expression: &Expr) -> Result<(), CompileError> {
        match expression {
            Expr::Try(inner, _) | Expr::Paren(inner, _) => {
                self.collect_resource_producers(inner)?
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                for branch in branches {
                    self.collect_resource_producers(&branch.value)?;
                }
                self.collect_resource_producers(else_value)?;
            }
            Expr::Case { arms, else_arm, .. } => {
                for arm in arms {
                    self.collect_resource_producers(&arm.value)?;
                }
                if let Some(arm) = else_arm {
                    self.collect_resource_producers(&arm.value)?;
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                for operation in operations.iter().rev() {
                    match operation {
                        PostfixOperation::Field { .. } => {}
                        PostfixOperation::MethodCall { span, .. } => {
                            let key = fpas_sema::postfix_operation_lookup_key(operation);
                            if !self.fluent_calls.get(&key).is_some_and(|target| {
                                matches!(
                                    target.name.as_str(),
                                    "Std.Results.Unwrap"
                                        | "Std.Options.Unwrap"
                                        | "Std.Results.MapError"
                                )
                            }) {
                                self.resource_producers.push(ResourceProducer {
                                    span: *span,
                                    kind: ProducerKind::Call,
                                });
                                return Ok(());
                            }
                        }
                        PostfixOperation::Index { span, .. } => {
                            self.resource_producers.push(ResourceProducer {
                                span: *span,
                                kind: ProducerKind::Index,
                            });
                            return Ok(());
                        }
                    }
                }
                self.collect_resource_producers(base)?;
            }
            Expr::Designator(designator) => {
                let (root, root_parts) = self.designator_root(designator)?;
                let (span, kind) = match designator.parts.get(root_parts..) {
                    Some([.., DesignatorPart::Ident(_, span)]) => (*span, ProducerKind::Field),
                    Some([.., DesignatorPart::Index(_, span)]) => (*span, ProducerKind::Index),
                    _ if self.binding_is_cell(&root) => (designator.span, ProducerKind::Cell),
                    _ if self.binding_is_reference(&root) => {
                        (designator.span, ProducerKind::Reference)
                    }
                    _ if self.has_binding(&root) => (designator.span, ProducerKind::Local),
                    _ => (designator.span, ProducerKind::Global),
                };
                self.resource_producers
                    .push(ResourceProducer { span, kind });
            }
            Expr::Call { span, .. } => {
                let kind = if self
                    .record_constructions
                    .contains(&fpas_sema::expr_lookup_key(expression))
                {
                    ProducerKind::Record
                } else {
                    ProducerKind::Call
                };
                self.resource_producers
                    .push(ResourceProducer { span: *span, kind });
            }
            Expr::RecordUpdate { span, .. } => self.resource_producers.push(ResourceProducer {
                span: *span,
                kind: ProducerKind::RecordUpdate,
            }),
            _ => {}
        }
        Ok(())
    }
}
