//! Verify the native operational-error ABI once, before executing hosted calls.
use super::super::{VmError, diagnostics};
use crate::stdlib::native_errors::{ErrorFieldType, NativeErrorFamily as Family};
use crate::{
    ErrorCause, Executable, InstructionAddress, Intrinsic, NominalKind, Opcode, TypeIdentity,
};

pub(super) fn validate(image: &Executable, address: InstructionAddress) -> Result<(), VmError> {
    let mut required = vec![false; Family::ALL.len()];
    for instruction in &image.code {
        if !instruction.opcode().is_ok_and(|op| op == Opcode::Intrinsic) {
            continue;
        }
        let family = match Intrinsic::from_u16(instruction.abc_payload().b) {
            Some(Intrinsic::Str(crate::StrIntrinsic::Utf8Decode)) => Some(Family::Utf8),
            Some(Intrinsic::Str(
                crate::StrIntrinsic::Utf8Encode | crate::StrIntrinsic::Utf8Append,
            )) => Some(Family::Buffer),
            Some(Intrinsic::Bytes(operation))
                if !matches!(
                    operation,
                    crate::BytesIntrinsic::Length | crate::BytesIntrinsic::Capacity
                ) =>
            {
                Some(Family::Buffer)
            }
            Some(Intrinsic::Acp(operation)) => {
                let record = if matches!(
                    operation,
                    crate::AcpIntrinsic::ParseRawJson | crate::AcpIntrinsic::RawJsonText
                ) {
                    "Std.Ai.Acp.Types.RawJson"
                } else {
                    "Std.Ai.Acp.Types.UInt64"
                };
                super::native_records::require(image, record, address)?;
                Some(Family::Acp)
            }
            Some(Intrinsic::Fs(operation))
                if !matches!(
                    operation,
                    crate::FsIntrinsic::Exists
                        | crate::FsIntrinsic::IsFile
                        | crate::FsIntrinsic::IsDir
                        | crate::FsIntrinsic::IsSymLink
                        | crate::FsIntrinsic::TempDir
                ) =>
            {
                Some(Family::Fs)
            }
            Some(Intrinsic::Net(_))
            | Some(Intrinsic::Server(crate::ServerIntrinsic::ShutdownErrors)) => Some(Family::Net),
            Some(Intrinsic::Server(
                crate::ServerIntrinsic::CreateLifetime
                | crate::ServerIntrinsic::OwnListener
                | crate::ServerIntrinsic::ObserveSignals
                | crate::ServerIntrinsic::FinishShutdown,
            )) => Some(Family::Server),
            Some(Intrinsic::Task(
                crate::TaskIntrinsic::Send
                | crate::TaskIntrinsic::TrySend
                | crate::TaskIntrinsic::SendWithCancellation
                | crate::TaskIntrinsic::SendWithTimeout
                | crate::TaskIntrinsic::Receive
                | crate::TaskIntrinsic::TryReceive
                | crate::TaskIntrinsic::ReceiveWithCancellation
                | crate::TaskIntrinsic::ReceiveWithTimeout
                | crate::TaskIntrinsic::WaitAnyWithTimeout
                | crate::TaskIntrinsic::WaitAnyWithCancellation
                | crate::TaskIntrinsic::CloseTaskGroupWithTimeout
                | crate::TaskIntrinsic::ReceiveCase
                | crate::TaskIntrinsic::SendCase,
            )) => Some(Family::Task),
            Some(Intrinsic::Proc(operation))
                if !matches!(
                    operation,
                    crate::ProcIntrinsic::DefaultOptions
                        | crate::ProcIntrinsic::DefaultStartOptions
                ) =>
            {
                Some(Family::Proc)
            }
            Some(Intrinsic::Parse(_)) => Some(Family::Parse),
            Some(Intrinsic::Encoding(_)) => Some(Family::Encoding),
            Some(Intrinsic::Crypto(_)) => Some(Family::Crypto),
            Some(Intrinsic::Tokenizer(operation))
                if operation != crate::TokenizerIntrinsic::Encodings =>
            {
                Some(Family::Tokenizer)
            }
            Some(Intrinsic::Json(crate::JsonIntrinsic::Parse)) => Some(Family::Json),
            Some(Intrinsic::Toml(crate::TomlIntrinsic::Parse)) => Some(Family::Toml),
            _ => None,
        };
        if let Some(family) = family {
            required[family as usize] = true;
        }
    }
    // Close over declared Cause families without requiring unrelated native modules.
    loop {
        let mut changed = false;
        for family in Family::ALL
            .iter()
            .copied()
            .filter(|f| required[*f as usize])
            .collect::<Vec<_>>()
        {
            for dependency in family
                .variants()
                .iter()
                .flat_map(|v| v.fields.iter())
                .filter_map(|(_, ty)| ty.cause_family())
            {
                changed |= !required[dependency as usize];
                required[dependency as usize] = true;
            }
        }
        if !changed {
            break;
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
            for (_, shape) in expected.fields {
                if let ErrorFieldType::Record(name) | ErrorFieldType::OptionalRecord(name) = shape {
                    super::native_records::require(image, name, address)?;
                }
            }
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
                .position(|(_, ty)| ty.cause_family().is_some())
            {
                let family = expected.fields[field]
                    .1
                    .cause_family()
                    .ok_or_else(failure)?;
                let index = image
                    .enums
                    .iter()
                    .position(|e| {
                        image
                            .strings
                            .get(e.name)
                            .is_some_and(|name| name.eq_ignore_ascii_case(family.name()))
                    })
                    .ok_or_else(failure)?;
                Some(ErrorCause {
                    field: field as u16,
                    family: crate::EnumTypeId::new(index as u16),
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
        ErrorFieldType::Record(name) => TypeIdentity::Nominal {
            kind: NominalKind::Record,
            declaration: name.to_ascii_lowercase(),
            arguments: vec![],
        },
        ErrorFieldType::OptionalRecord(name) => {
            TypeIdentity::Option(Box::new(identity(ErrorFieldType::Record(name))))
        }
        ErrorFieldType::Host | ErrorFieldType::Net => TypeIdentity::Nominal {
            kind: NominalKind::Error,
            declaration: (if matches!(ty, ErrorFieldType::Host) {
                Family::Host
            } else {
                Family::Net
            })
            .name()
            .to_ascii_lowercase(),
            arguments: vec![],
        },
    }
}
