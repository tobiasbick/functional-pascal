//! Preserve nominal field schemas needed by equality-constrained error construction.

use super::ErrorMetadata;
use fpas_sema::Ty;
use fpas_unit::equality::{EqualityDeclaration, EqualityDeclarations};

impl ErrorMetadata {
    pub(crate) fn equality_declarations(&self) -> Result<Vec<EqualityDeclaration>, String> {
        if !self.schemas.values().any(|schema| {
            schema
                .parameters
                .contains(&Some(fpas_unit::interface::TypeConstraint::Equatable))
        }) {
            return Ok(Vec::new());
        }
        let mut result = self.imported_equality.clone();
        for ty in self.named.values() {
            let (parameters, fields) = match ty {
                Ty::Record(record) if record.type_args.is_empty() => (
                    &record.type_params,
                    record
                        .fields
                        .iter()
                        .map(|(_, field)| field)
                        .collect::<Vec<_>>(),
                ),
                Ty::Enum(enumeration) if enumeration.type_args.is_empty() => (
                    &enumeration.type_params,
                    enumeration
                        .variants
                        .iter()
                        .flat_map(|variant| variant.fields.iter().map(|(_, field)| field))
                        .collect(),
                ),
                _ => continue,
            };
            let schema = EqualityDeclaration {
                identity: self.identity(ty, parameters)?,
                parameters: u16::try_from(parameters.len())
                    .map_err(|_| "too many equality parameters")?,
                fields: fields
                    .into_iter()
                    .map(|field| self.identity(field, parameters))
                    .collect::<Result<_, _>>()?,
            };
            schema.validate().map_err(str::to_owned)?;
            result.push(schema);
        }
        EqualityDeclarations::new(result)
            .map(EqualityDeclarations::into_declarations)
            .map_err(str::to_owned)
    }
}
