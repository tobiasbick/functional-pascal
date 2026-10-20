//! Verify the native operational-error ABI once, before executing hosted calls.
use super::super::{VmError, diagnostics};
use fpas_bytecode::{
    ErrorCause, Executable, InstructionAddress, Intrinsic, NominalKind, Opcode, TypeIdentity,
};
use fpas_std::native_errors::{ErrorFieldType, NativeErrorFamily as Family};

pub(super) fn validate(image: &Executable, address: InstructionAddress) -> Result<(), VmError> {
    let mut required = vec![false; Family::ALL.len()];
    for instruction in &image.code {
        if !instruction.opcode().is_ok_and(|op| op == Opcode::Intrinsic) {
            continue;
        }
        let family = match Intrinsic::from_u16(instruction.abc_payload().b) {
            Some(Intrinsic::Fs(operation))
                if !matches!(
                    operation,
                    fpas_bytecode::FsIntrinsic::Exists
                        | fpas_bytecode::FsIntrinsic::IsFile
                        | fpas_bytecode::FsIntrinsic::IsDir
                        | fpas_bytecode::FsIntrinsic::IsSymLink
                        | fpas_bytecode::FsIntrinsic::TempDir
                ) =>
            {
                Some(Family::Fs)
            }
            Some(Intrinsic::Proc(_)) => Some(Family::Proc),
            Some(Intrinsic::Parse(_)) => Some(Family::Parse),
            Some(Intrinsic::Crypto(_)) => Some(Family::Crypto),
            Some(Intrinsic::Json(fpas_bytecode::JsonIntrinsic::Parse)) => Some(Family::Json),
            Some(Intrinsic::Toml(fpas_bytecode::TomlIntrinsic::Parse)) => Some(Family::Toml),
            _ => None,
        };
        if let Some(family) = family {
            required[family as usize] = true;
            if matches!(family, Family::Fs | Family::Proc | Family::Crypto) {
                required[Family::Host as usize] = true;
            }
        }
    }
    for family in Family::ALL
        .iter()
        .copied()
        .filter(|family| required[*family as usize])
    {
        let failure = || {
            diagnostics::internal(
                image,
                address,
                format!(
                    "Native error image contract does not match {}",
                    family.name()
                ),
            )
        };
        let mut owners = image.enums.iter().enumerate().filter(|(_, e)| {
            image
                .strings
                .get(e.name)
                .is_some_and(|name| name.eq_ignore_ascii_case(family.name()))
        });
        let (index, owner) = owners.next().ok_or_else(failure)?;
        if owners.next().is_some() {
            return Err(failure());
        }
        let key = owner.error.as_ref().ok_or_else(failure)?;
        if key.declaration != family.name().to_ascii_lowercase() || !key.arguments.is_empty() {
            return Err(failure());
        }
        let variants = image
            .enum_variants
            .iter()
            .filter(|v| usize::from(v.owner.get()) == index)
            .collect::<Vec<_>>();
        if variants.len() != family.variants().len() {
            return Err(failure());
        }
        for expected in family.variants() {
            let variant = variants
                .iter()
                .find(|v| image.strings.get(v.name) == Some(expected.name))
                .ok_or_else(failure)?;
            let metadata = variant.error.as_ref().ok_or_else(failure)?;
            if variant.fields.len() != expected.fields.len()
                || variant
                    .fields
                    .iter()
                    .zip(expected.fields)
                    .any(|(id, (name, _))| image.strings.get(*id) != Some(*name))
                || metadata.field_types
                    != expected
                        .fields
                        .iter()
                        .map(|(_, ty)| identity(*ty))
                        .collect::<Vec<_>>()
                || metadata.message.and_then(|id| image.strings.get(id)) != Some(expected.message)
            {
                return Err(failure());
            }
            let cause = if let Some(field) = expected
                .fields
                .iter()
                .position(|(_, ty)| matches!(ty, ErrorFieldType::Host))
            {
                let index = image
                    .enums
                    .iter()
                    .position(|e| {
                        image
                            .strings
                            .get(e.name)
                            .is_some_and(|name| name.eq_ignore_ascii_case(Family::Host.name()))
                    })
                    .ok_or_else(failure)?;
                Some(ErrorCause {
                    field: field as u16,
                    family: fpas_bytecode::EnumTypeId::new(index as u16),
                })
            } else {
                None
            };
            if metadata.cause != cause {
                return Err(failure());
            }
        }
    }
    Ok(())
}

fn identity(ty: ErrorFieldType) -> TypeIdentity {
    match ty {
        ErrorFieldType::String => TypeIdentity::String,
        ErrorFieldType::Integer => TypeIdentity::Integer,
        ErrorFieldType::Strings => TypeIdentity::Array(Box::new(TypeIdentity::String)),
        ErrorFieldType::OptionalString => TypeIdentity::Option(Box::new(TypeIdentity::String)),
        ErrorFieldType::OptionalInteger => TypeIdentity::Option(Box::new(TypeIdentity::Integer)),
        ErrorFieldType::Host => TypeIdentity::Nominal {
            kind: NominalKind::Error,
            declaration: Family::Host.name().to_ascii_lowercase(),
            arguments: vec![],
        },
    }
}
