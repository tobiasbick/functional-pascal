//! Shared noncapturing bodies for checked enum and native-error constructor values.
//! Documentation: `docs/pascal/language/functions/function-types.md`.

use super::*;
use fpas_ir::{EnumLayoutId, IrType, Operation, Terminator, TypeId, VariantId};
use fpas_parser::DesignatorPart;
use fpas_sema::{PayloadConstructor, Ty};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Construction {
    Enum(EnumLayoutId, VariantId),
    Payload(PayloadConstructor),
}

pub(in crate::lowering) struct ConstructorRoutine {
    pub id: FunctionId,
    pub owner: FunctionId,
    name: String,
    parameters: Vec<TypeId>,
    result: TypeId,
    semantic_result: Ty,
    construction: Construction,
    span: fpas_lexer::Span,
}

impl ClosureRegistry<'_> {
    pub(super) fn register_constructor_value(
        &mut self,
        expression: &Expr,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let Expr::Designator(designator) = expression else {
            return Ok(());
        };
        if !metadata
            .enum_members
            .contains(&fpas_sema::designator_lookup_key(designator))
        {
            return Ok(());
        }
        let key = fpas_sema::expr_lookup_key(expression);
        let Some(Ty::Function(signature)) = metadata.expr_types.get(&key) else {
            return Ok(());
        };
        let Some(DesignatorPart::Ident(name, _)) = designator.parts.last() else {
            return Ok(());
        };
        let span = expression.span();
        let result = types.intern(&signature.return_type, span.line, span.column)?;
        let construction = if designator.parts.len() == 1
            && let Some(constructor) = PayloadConstructor::from_name(name)
        {
            Construction::Payload(constructor)
        } else {
            let Some(IrType::Enum(layout)) = types.kind(result) else {
                return Err(unsupported(span, "constructor result layout"));
            };
            let layout = *layout;
            let variant = types.enum_layouts[layout.get() as usize]
                .variants
                .iter()
                .find(|variant| variant.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| unsupported(span, "constructor variant"))?
                .id;
            Construction::Enum(layout, variant)
        };
        let parameters = signature
            .params
            .iter()
            .map(|parameter| types.intern(&parameter.ty, span.line, span.column))
            .collect::<Result<Vec<_>, _>>()?;
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let existing = self.constructor_routines.iter().find(|routine| {
            routine.result == result
                && routine.semantic_result == *signature.return_type
                && routine.construction == construction
                && routine.parameters == parameters
                && routine.owner == owner
        });
        let id = if let Some(existing) = existing {
            existing.id
        } else {
            let id = FunctionId::new(self.next_id);
            self.next_id = self
                .next_id
                .checked_add(1)
                .ok_or_else(|| unsupported(span, "constructor identifier overflow"))?;
            self.constructor_routines.push(ConstructorRoutine {
                id,
                owner,
                name: format!("$constructor_{}", id.get()),
                parameters,
                result,
                semantic_result: *signature.return_type.clone(),
                construction,
                span,
            });
            id
        };
        self.intrinsic_targets.insert(
            key,
            BoundMethodTarget {
                function: id,
                value_type,
            },
        );
        Ok(())
    }

    pub(in crate::lowering) fn lower_constructor(
        &self,
        routine: &ConstructorRoutine,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
        globals: &BTreeMap<String, super::super::context::GlobalBinding>,
        constants: &BTreeMap<String, fpas_ir::Constant>,
    ) -> Result<(Function, types::TypeTable), CompileError> {
        let parameters = routine
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| ParameterInput {
                name: format!("field{index}"),
                type_name: None,
                ty: *ty,
                declaration: None,
                reference: false,
            })
            .collect::<Vec<_>>();
        let mut context = LoweringContext::new(FunctionInput {
            name: &routine.name,
            source_name: &self.source_name,
            id: routine.id,
            result: routine.result,
            parameters: &parameters,
            captures: &[],
            globals: globals.clone(),
            constants: constants.clone(),
            metadata,
            callables: self.callables.clone(),
            closure_targets: self.targets.clone(),
            bound_method_targets: self.bound_targets.clone(),
            intrinsic_targets: self.intrinsic_targets.clone(),
            cell_names: BTreeSet::new(),
            type_table: types.clone(),
        })?;
        if let Construction::Payload(constructor) = routine.construction {
            let parameter = parameters
                .first()
                .ok_or_else(|| unsupported(routine.span, "payload constructor parameter"))?;
            let payload = context.read_named_local(&parameter.name, routine.span)?;
            let operation = match constructor {
                PayloadConstructor::Some => Operation::MakeSome(payload),
                PayloadConstructor::Ok => Operation::MakeOk(payload),
                PayloadConstructor::Error => Operation::MakeError(payload),
            };
            let value = context.emit_value(operation, routine.result, routine.span)?;
            context.terminate(Terminator::Return(Some(value)))?;
            return context.finish(routine.span);
        }
        let Construction::Enum(layout, variant) = routine.construction else {
            return Err(unsupported(routine.span, "enum constructor layout"));
        };
        let field_types = context.type_table.enum_layouts[layout.get() as usize].variants
            [variant.get() as usize]
            .fields
            .clone();
        let fields = parameters
            .iter()
            .zip(field_types)
            .map(|(parameter, ty)| {
                let value = context.read_named_local(&parameter.name, routine.span)?;
                context.value_as(value, ty, routine.span)
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        let value = context.emit_value(
            Operation::MakeEnum {
                layout,
                variant,
                fields,
            },
            routine.result,
            routine.span,
        )?;
        context.terminate(Terminator::Return(Some(value)))?;
        context.finish(routine.span)
    }
}
