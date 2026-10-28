//! Essential native-error metadata at the semantic/IR and artifact boundaries.
//! Documentation: `docs/pascal/language/types/errors.md`.

use crate::{CompileError, error::internal_compiler_error};
use fpas_code::{ErrorInstanceKey, TypeIdentity};
use fpas_ir::EnumLayoutId;
use fpas_sema::{EnumTy, GenericParamDef, Ty};
use std::collections::BTreeMap;
mod equality;
mod object;

#[derive(Debug, Clone, Default)]
pub(crate) struct ErrorMetadata {
    pub owner: String,
    pub named: std::sync::Arc<BTreeMap<String, Ty>>,
    pub layouts: BTreeMap<EnumLayoutId, EnumTy>,
    pub schemas: BTreeMap<String, fpas_unit::errors::ErrorSchema>,
    pub imported_equality: Vec<fpas_unit::equality::EqualityDeclaration>,
    pub locations: BTreeMap<String, fpas_lexer::Span>,
    pub instances: BTreeMap<EnumLayoutId, (ErrorInstanceKey, fpas_unit::errors::ErrorSchema)>,
    pub requirements: BTreeMap<fpas_ir::FunctionId, fpas_unit::errors::ErrorConstructorSignature>,
    pub factories: BTreeMap<fpas_ir::FunctionId, fpas_unit::errors::ErrorConstructorRequest>,
    pub displays: BTreeMap<fpas_ir::FunctionId, String>,
}

impl ErrorMetadata {
    pub fn identity(
        &self,
        ty: &Ty,
        parameters: &[GenericParamDef],
    ) -> Result<TypeIdentity, String> {
        fpas_sema::semantic_type_identity(ty, &self.owner, parameters, &|name| {
            self.named.get(&name.to_ascii_lowercase()).cloned()
        })
    }

    pub fn instance(
        &self,
        enumeration: &EnumTy,
        parameters: &[GenericParamDef],
    ) -> Result<ErrorInstanceKey, String> {
        self.identity(
            &Ty::Enum(std::sync::Arc::new(enumeration.clone())),
            parameters,
        )?
        .error_key()
        .ok_or_else(|| "native error identity is unavailable".into())
    }

    pub fn concrete(&self, id: EnumLayoutId) -> Option<ErrorInstanceKey> {
        if let Some((key, _)) = self.instances.get(&id) {
            return Some(key.clone());
        }
        self.layouts
            .get(&id)
            .and_then(|enumeration| self.instance(enumeration, &[]).ok())
    }

    pub fn display_name(&self, enumeration: &EnumTy) -> String {
        let declaration = if enumeration.name.contains('.') {
            enumeration.name.to_ascii_lowercase()
        } else {
            format!("{}.{}", self.owner, enumeration.name).to_ascii_lowercase()
        };
        if let Some(schema) = self.schemas.get(&declaration) {
            return schema.display_name.clone();
        }
        if enumeration.name.contains('.') {
            enumeration.name.clone()
        } else {
            format!("{}.{}", self.owner, enumeration.name)
        }
    }

    pub fn variant(
        &self,
        layout: EnumLayoutId,
        variant: usize,
        strings: &mut impl FnMut(&str) -> Result<fpas_code::StringId, CompileError>,
    ) -> Result<Option<fpas_code::ErrorVariant>, CompileError> {
        if self.concrete(layout).is_none() {
            return Ok(None);
        }
        let (field_types, message, cause) = if let Some((_, schema)) = self.instances.get(&layout) {
            let variant = &schema.variants[variant];
            (
                variant
                    .fields
                    .iter()
                    .map(|field| field.ty.clone())
                    .collect::<Vec<_>>(),
                variant.message.clone(),
                variant.cause.map(usize::from),
            )
        } else {
            let variant = &self.layouts[&layout].variants[variant];
            (
                variant
                    .fields
                    .iter()
                    .map(|(_, ty)| self.identity(ty, &[]).map_err(error))
                    .collect::<Result<Vec<_>, _>>()?,
                variant.message.clone(),
                variant
                    .fields
                    .iter()
                    .position(|(name, _)| name.eq_ignore_ascii_case("Cause")),
            )
        };
        let cause = cause
            .map(|field| {
                let key = field_types[field]
                    .error_key()
                    .ok_or_else(|| error("Cause is not a native error"))?;
                let family = self
                    .layouts
                    .keys()
                    .chain(self.instances.keys())
                    .find(|id| self.concrete(**id).as_ref() == Some(&key))
                    .ok_or_else(|| error("concrete Cause layout is unavailable"))?;
                Ok::<_, CompileError>(fpas_code::ErrorCause {
                    field: u16::try_from(field).map_err(|_| error("Cause slot overflow"))?,
                    family: fpas_code::EnumTypeId::new(
                        u16::try_from(family.get()).map_err(|_| error("Cause family overflow"))?,
                    ),
                })
            })
            .transpose()?;
        Ok(Some(fpas_code::ErrorVariant {
            field_types,
            message: message.as_deref().map(strings).transpose()?,
            cause,
        }))
    }
}

pub(super) fn error(message: impl Into<String>) -> CompileError {
    internal_compiler_error(
        message,
        "This is an internal native-error lowering error. Report the source program.",
        1,
        1,
    )
}
