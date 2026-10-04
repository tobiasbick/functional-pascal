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
        let qualified = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("."));
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
            && let Some(variant) = enumeration
                .variants
                .iter()
                .find(|variant| variant.name.eq_ignore_ascii_case(name))
        {
            let value = variant
                .backing_value
                .ok_or_else(|| unsupported(designator.span, "simple enum member backing value"))?;
            return self.emit_value(
                Operation::Const(Constant::Integer(value)),
                types::INTEGER,
                designator.span,
            );
        }
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
            && let Some(value) =
                fpas_std::intrinsic_std_constant_value(&self.qualified_import_name(name))
        {
            let (constant, ty) = match value {
                fpas_bytecode::Value::Integer(value) => (Constant::Integer(value), types::INTEGER),
                fpas_bytecode::Value::Real(value) => (Constant::Real(value), types::REAL),
                _ => return Err(unsupported(designator.span, "built-in constant value")),
            };
            return self.emit_value(Operation::Const(constant), ty, designator.span);
        }
        if self.designator_root(designator).is_some() {
            return self.lower_designator_read(designator);
        }
        if let Some(name) = qualified.as_deref()
            && self.resolve_callable(name).is_some()
        {
            let value = self.read_callable_value(name, designator.span)?;
            let expected = self.expression_ir_type(expression)?;
            self.coerce_value_type(value, expected, designator.span)
        } else {
            Err(unsupported(
                designator.span,
                &format!(
                    "unresolved designator `{}`",
                    qualified.as_deref().unwrap_or("indexed value")
                ),
            ))
        }
    }
}
