//! Designator, constant, enum-member, and callable-value lowering.

use fpas_ir::{Constant, Operation, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr};
use fpas_sema::Ty;

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};
use super::super::types;

impl LoweringContext {
    pub(super) fn lower_designator_expression(
        &mut self,
        designator: &Designator,
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let designator_key = fpas_sema::designator_lookup_key(designator);
        if self.bound_method_targets.contains_key(&designator_key) {
            return self.lower_bound_method(designator, designator_key);
        }
        if let Some(reads) = self.property_reads.get(&designator_key).cloned() {
            return self.lower_property_read(designator, &reads);
        }
        let qualified = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("."));
        if let Some(name) = qualified.as_deref()
            && (designator.parts.len() > 1 || (!self.has_binding(name) && !self.has_global(name)))
            && let Some(value) = self.constant(name)
        {
            let ty = match value {
                Constant::Boolean(_) => types::BOOLEAN,
                Constant::Integer(_) => types::INTEGER,
                Constant::Real(_) => types::REAL,
                Constant::String(_) => types::STRING,
                Constant::Unit => types::UNIT,
            };
            return self.emit_value(Operation::Const(value), ty, designator.span);
        }
        if let Some(name) = qualified.as_deref()
            && designator
                .parts
                .first()
                .and_then(|part| match part {
                    DesignatorPart::Ident(root, _) => Some(root.as_str()),
                    DesignatorPart::Index(_, _) => None,
                })
                .is_some_and(|root| !self.has_binding(root) && !self.has_global(root))
            && let Some(value) = super::super::builtin_constants::value(name)
        {
            let (constant, ty) = match value {
                fpas_bytecode::Value::Integer(value) => (Constant::Integer(value), types::INTEGER),
                fpas_bytecode::Value::Real(value) => (Constant::Real(value), types::REAL),
                _ => return Err(unsupported(designator.span, "built-in constant value")),
            };
            return self.emit_value(Operation::Const(constant), ty, designator.span);
        }
        if let Ok(Ty::Enum(enumeration)) = self.expression_type(expression)
            && enumeration.has_data()
            && let Some(name) = designator.parts.last().and_then(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name),
                DesignatorPart::Index(_, _) => None,
            })
        {
            let ty = self.expression_ir_type(expression)?;
            if let Some(fpas_ir::IrType::Enum(layout)) = self.type_kind(ty)
                && let Some((variant, fields)) = self.enum_variant(layout, name)
                && fields.is_empty()
            {
                return self.emit_value(
                    Operation::MakeEnum {
                        layout,
                        variant,
                        fields: Vec::new(),
                    },
                    ty,
                    designator.span,
                );
            }
        }
        if let Ok(Ty::Enum(enumeration)) = self.expression_type(expression)
            && !enumeration.has_data()
            && let Some(name) = designator.parts.last().and_then(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name),
                DesignatorPart::Index(_, _) => None,
            })
            && let Some(index) = enumeration
                .variants
                .iter()
                .position(|variant| variant.name.eq_ignore_ascii_case(name))
        {
            let value = i64::try_from(index)
                .map_err(|_| unsupported(designator.span, "enum ordinal overflow"))?;
            return self.emit_value(
                Operation::Const(Constant::Integer(value)),
                types::INTEGER,
                designator.span,
            );
        }
        let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
            return self.lower_designator_read(designator);
        };
        if self.has_binding(name) {
            self.read_named_local(name, designator.span)
        } else if self.has_global(name) {
            self.read_global(name, designator.span)
        } else if let Some(callable) = self.resolve_callable(name) {
            let captures = callable
                .captures
                .iter()
                .map(|capture| self.read_capture(&capture.name, designator.span))
                .collect::<Result<Vec<_>, _>>()?;
            self.emit_value(
                Operation::MakeClosure {
                    function: callable.function,
                    captures,
                },
                callable.value_type,
                designator.span,
            )
        } else {
            Err(unsupported(designator.span, "unresolved designator"))
        }
    }
}
