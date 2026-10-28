//! Detached protocol values cross the VM boundary through admitted typed layouts.

mod pack;
#[cfg(test)]
mod tests;
mod unpack;

use crate::execution::vm::layouts::RuntimeLayouts;
use crate::{DebugType, DebugTypeId, EnumTypeId, Executable, RecordTypeId};
use fpas_acp::{
    CodecError, ErrorKind,
    binding::{Catalog, Type},
};

pub(super) struct Converter<'a> {
    pub catalog: &'a Catalog,
    pub image: &'a Executable,
    pub layouts: &'a RuntimeLayouts,
    pub depth: usize,
}

fn invalid() -> CodecError {
    CodecError::new(ErrorKind::InvalidMessage)
}

impl Converter<'_> {
    fn expected(&self, expected: Option<DebugTypeId>) -> Result<Option<DebugTypeId>, CodecError> {
        // Ordinary generic enums use erased storage. The SDK-derived Type still
        // checks every concrete child; named records/enums retain their ABI identity.
        match expected {
            Some(id) => Ok((!matches!(self.debug(Some(id))?, DebugType::Dynamic)).then_some(id)),
            None => Ok(None),
        }
    }

    fn element(
        &self,
        expected: Option<DebugTypeId>,
        array: bool,
    ) -> Result<Option<DebugTypeId>, CodecError> {
        let Some(expected) = expected else {
            return Ok(None);
        };
        match self.debug(Some(expected))? {
            DebugType::Array(id) if array => Ok(Some(*id)),
            DebugType::Option(id) if !array => Ok(Some(*id)),
            _ => Err(invalid()),
        }
    }

    fn dictionary(&self, expected: Option<DebugTypeId>) -> Result<Option<DebugTypeId>, CodecError> {
        let Some(expected) = expected else {
            return Ok(None);
        };
        let DebugType::Dictionary { key, value } = *self.debug(Some(expected))? else {
            return Err(invalid());
        };
        self.scalar(Some(key), |ty| matches!(ty, DebugType::String))?;
        Ok(Some(value))
    }

    fn debug(&self, expected: Option<DebugTypeId>) -> Result<&DebugType, CodecError> {
        self.image
            .debug_types
            .get(expected.ok_or_else(invalid)?.get() as usize)
            .ok_or_else(invalid)
    }

    fn scalar(
        &self,
        expected: Option<DebugTypeId>,
        predicate: impl FnOnce(&DebugType) -> bool,
    ) -> Result<(), CodecError> {
        if expected.is_none() || predicate(self.debug(expected)?) {
            Ok(())
        } else {
            Err(invalid())
        }
    }

    fn record(
        &self,
        name: &str,
        expected: Option<DebugTypeId>,
    ) -> Result<RecordTypeId, CodecError> {
        let id = if expected.is_some() {
            let DebugType::Record(id) = self.debug(expected)? else {
                return Err(invalid());
            };
            *id
        } else {
            let index = self
                .image
                .records
                .iter()
                .position(|record| {
                    self.image
                        .strings
                        .get(record.name)
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
                })
                .ok_or_else(invalid)?;
            RecordTypeId::try_from_index(index).map_err(|_| invalid())?
        };
        let record = self
            .image
            .records
            .get(usize::from(id.get()))
            .ok_or_else(invalid)?;
        if !self
            .image
            .strings
            .get(record.name)
            .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
        {
            return Err(invalid());
        }
        Ok(id)
    }

    fn enumeration(
        &self,
        name: &str,
        expected: Option<DebugTypeId>,
    ) -> Result<EnumTypeId, CodecError> {
        let id = if expected.is_some() {
            let DebugType::Enum(id) = self.debug(expected)? else {
                return Err(invalid());
            };
            *id
        } else {
            let index = self
                .image
                .enums
                .iter()
                .position(|owner| {
                    self.image
                        .strings
                        .get(owner.name)
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
                })
                .ok_or_else(invalid)?;
            EnumTypeId::try_from_index(index).map_err(|_| invalid())?
        };
        let owner = self
            .image
            .enums
            .get(usize::from(id.get()))
            .ok_or_else(invalid)?;
        if !self.image.strings.get(owner.name).is_some_and(|actual| {
            actual.eq_ignore_ascii_case(name)
                || (name == "Std.Acp.Types.Field" && actual.starts_with("Std.Acp.Types.Field"))
        }) {
            return Err(invalid());
        }
        Ok(id)
    }

    fn variant(&self, owner: EnumTypeId, name: &str) -> Result<usize, CodecError> {
        self.image
            .enum_variants
            .iter()
            .position(|variant| {
                variant.owner == owner && self.image.strings.get(variant.name) == Some(name)
            })
            .ok_or_else(invalid)
    }

    fn variant_fields(&self, index: usize, names: &[&str]) -> Result<(), CodecError> {
        let variant = self.image.enum_variants.get(index).ok_or_else(invalid)?;
        if variant.fields.len() != names.len()
            || variant.field_types.len() != names.len()
            || !variant
                .fields
                .iter()
                .zip(names)
                .all(|(id, name)| self.image.strings.get(*id) == Some(*name))
        {
            return Err(invalid());
        }
        Ok(())
    }

    fn protocol_variant_fields(
        &self,
        key: &str,
        name: &str,
        index: usize,
    ) -> Result<(), CodecError> {
        let fields = if name == "Unknown" {
            match &self
                .catalog
                .declarations
                .get(key)
                .ok_or_else(invalid)?
                .shape
            {
                fpas_acp::binding::Shape::Enum {
                    unknown: Some(fpas_acp::binding::Unknown::String),
                    ..
                } => vec!["Value"],
                _ => vec!["Tag", "Payload"],
            }
        } else if self.enum_types(key, name)?.is_empty() {
            vec![]
        } else {
            vec!["Payload"]
        };
        self.variant_fields(index, &fields)
    }

    fn enum_types(&self, key: &str, name: &str) -> Result<Vec<Type>, CodecError> {
        let declaration = self.catalog.declarations.get(key).ok_or_else(invalid)?;
        let fpas_acp::binding::Shape::Enum { variants, unknown } = &declaration.shape else {
            return Err(invalid());
        };
        if name == "Unknown" {
            return match unknown {
                Some(fpas_acp::binding::Unknown::String) => Ok(vec![Type::String]),
                Some(fpas_acp::binding::Unknown::Tagged(_)) => {
                    Ok(vec![Type::String, Type::Raw { object: true }])
                }
                None => Err(invalid()),
            };
        }
        let variant = variants
            .iter()
            .find(|variant| variant.name == name)
            .ok_or_else(invalid)?;
        Ok(variant.payload.iter().cloned().collect())
    }
}
