//! Classification at checked allocation identities and exact binding declarations.

use super::{
    index::{Index, native},
    model::{CallArgument, SourceCallable},
};
use fpas_parser::{Designator, Expr};
use fpas_sema::{AnalysisMetadata, MethodCallTarget, SourceBindingKind, Ty};
use std::{path::Path, sync::Arc};

pub(super) struct Resolved {
    pub(super) target: Option<Arc<SourceCallable>>,
    pub(super) signature: Option<Ty>,
    pub(super) receiver: bool,
    pub(super) excluded: bool,
    pub(super) reason: Option<&'static str>,
}

impl Resolved {
    fn unresolved(reason: &'static str, signature: Option<Ty>) -> Self {
        Self {
            target: None,
            signature,
            receiver: false,
            excluded: false,
            reason: Some(reason),
        }
    }
    fn known(target: Option<Arc<SourceCallable>>, signature: Option<Ty>, receiver: bool) -> Self {
        let reason = target.is_none().then_some("declaration_unavailable");
        Self {
            target,
            signature,
            receiver,
            excluded: false,
            reason,
        }
    }
    fn builtin(name: &str, receiver: bool) -> Option<Self> {
        let (target, signature) = native(name)?;
        Some(Self::known(Some(target), Some(signature), receiver))
    }
}

pub(super) fn call(
    index: &Index,
    metadata: &AnalysisMetadata,
    path: &Path,
    key: usize,
    designator: Option<&Designator>,
) -> Resolved {
    if metadata.record_constructions.contains(&key)
        || metadata.enum_constructions.contains(&key)
        || metadata.distinct_conversions.contains(&key)
        || metadata.subrange_conversions.contains_key(&key)
    {
        return Resolved {
            target: None,
            signature: None,
            receiver: false,
            excluded: true,
            reason: None,
        };
    }
    if let Some(target) = metadata.method_calls.get(&key) {
        let target_source = index.method(path, target.qualified_name());
        let signature = target_source
            .as_ref()
            .and_then(|t| index.signatures.get(&t.identity))
            .cloned();
        return Resolved::known(
            target_source,
            signature,
            matches!(target, MethodCallTarget::Instance { .. }),
        );
    }
    if let Some(target) = metadata.fluent_calls.get(&key)
        && let Some(operation) = fpas_sema::native_operation_by_implementation(&target.name)
        && let Some(mut resolved) =
            Resolved::builtin(&target.name, !operation.receiver.is_factory())
    {
        resolved.signature = Some(Ty::Function(
            operation.signature_for(Some(&target.receiver_ty)),
        ));
        return resolved;
    }
    if let Some(target) = metadata.intrinsic_calls.get(&key) {
        if let Some(resolved) = Resolved::builtin(target, false) {
            return resolved;
        }
        // The compiler also records source-backed Std calls here. They retain their real body.
        let source = index.qualified(target);
        let signature = source
            .as_ref()
            .and_then(|s| index.signatures.get(&s.identity))
            .cloned();
        return Resolved::known(source, signature, false);
    }
    if metadata.member_value_calls.contains_key(&key) {
        return Resolved::unresolved("dynamic_callable", None);
    }
    let Some(designator) = designator else {
        return Resolved::unresolved("checked_target_unavailable", None);
    };
    binding(index, metadata, path, designator)
}

fn binding(
    index: &Index,
    metadata: &AnalysisMetadata,
    path: &Path,
    designator: &Designator,
) -> Resolved {
    let Some(binding) = metadata
        .source_bindings
        .get(&(designator.span.source_id, designator.span.offset))
    else {
        return Resolved::unresolved("checked_target_unavailable", None);
    };
    if binding.parts != designator.parts.len() {
        return Resolved::unresolved("dynamic_callable", None);
    }
    match binding.kind {
        SourceBindingKind::Routine => {
            let target = if let Some((_, start)) = binding.declaration {
                index.declaration(path, start)
            } else {
                index.qualified(&binding.name)
            };
            Resolved::known(target, Some(binding.ty.clone()), false)
        }
        SourceBindingKind::Native => Resolved::builtin(&binding.name, false).unwrap_or_else(|| {
            Resolved::unresolved("checked_target_unavailable", Some(binding.ty.clone()))
        }),
        SourceBindingKind::Value => {
            Resolved::unresolved("dynamic_callable", Some(binding.ty.clone()))
        }
        SourceBindingKind::Type | SourceBindingKind::EnumConstructor => Resolved {
            target: None,
            signature: None,
            receiver: false,
            excluded: true,
            reason: None,
        },
    }
}

pub(super) fn value(
    index: &Index,
    metadata: &AnalysisMetadata,
    path: &Path,
    key: usize,
    designator: Option<&Designator>,
    ty: Option<&Ty>,
) -> Option<Resolved> {
    if let Some(bound) = metadata.bound_methods.get(&key) {
        let target = index.method(path, &bound.qualified_name);
        return Some(Resolved::known(target, ty.cloned(), true));
    }
    if let Some(native) = metadata.native_values.get(&key) {
        return Resolved::builtin(&native.name, native.receiver_ty.is_some());
    }
    if let Some(designator) = designator {
        let resolved = binding(index, metadata, path, designator);
        if !resolved.excluded
            && (resolved.target.is_some()
                || is_callable(ty)
                || is_callable(resolved.signature.as_ref()))
        {
            return Some(resolved);
        }
    }
    is_callable(ty).then(|| Resolved::unresolved("dynamic_callable_value", ty.cloned()))
}

pub(super) fn is_callable(ty: Option<&Ty>) -> bool {
    matches!(ty, Some(Ty::Function(_) | Ty::Procedure(_)))
}

pub(super) fn arguments(
    metadata: &AnalysisMetadata,
    args: &[Expr],
    resolved: &Resolved,
) -> (Vec<CallArgument>, bool) {
    let named = args.iter().any(|arg| arg.argument_name().is_some());
    let order = args.first().and_then(|arg| {
        metadata
            .named_argument_orders
            .get(&fpas_sema::expr_lookup_key(arg))
    });
    let params = match resolved.signature.as_ref() {
        Some(Ty::Function(f)) => Some(f.params.as_slice()),
        Some(Ty::Procedure(p)) => Some(p.params.as_slice()),
        _ => None,
    };
    let source_receiver = resolved.receiver
        && resolved
            .target
            .as_ref()
            .is_some_and(|t| t.snapshot.is_some());
    let receiver_offset = usize::from(resolved.receiver);
    let complete = params.is_some() && (!named || order.is_some()) && resolved.reason.is_none();
    let arguments: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(written, argument)| {
            let value = argument.argument_value();
            let parameter = if named {
                order.and_then(|order| order.iter().position(|&i| i == written))
            } else {
                Some(written)
            };
            let mapped = parameter
                .filter(|&i| params.is_some_and(|p| i + usize::from(source_receiver) < p.len()));
            CallArgument {
                span: argument.span().diagnostic_span_or_synthetic(),
                value_span: value.span().diagnostic_span_or_synthetic(),
                name: argument.argument_name().map(str::to_owned),
                parameter_index: mapped.map(|i| i + receiver_offset),
                by_reference: matches!(value, Expr::VarArgument { .. }),
            }
        })
        .collect();
    let complete = complete
        && arguments
            .iter()
            .all(|argument| argument.parameter_index.is_some());
    (arguments, complete)
}
