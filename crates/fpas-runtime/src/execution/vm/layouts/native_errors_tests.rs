//! Native ABI checks are stricter than the general enum verifier.
use super::*;

fn image(source: &str) -> Executable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program)
        .expect("compile")
        .executable()
        .clone()
}

#[test]
fn required_native_error_metadata_is_checked_before_successful_calls() {
    let original =
        image("program T; uses Std.Text.Parse as Parse; begin discard Parse.Int('1'); end.");
    let address = InstructionAddress::new(0);
    RuntimeLayouts::build(&original, address).expect("native contract");
    for mutation in 0..5 {
        let mut changed = original.clone();
        match mutation {
            0 => {
                for family in &mut changed.enums {
                    family.error = None;
                }
                for variant in &mut changed.enum_variants {
                    variant.error = None;
                }
            }
            1 => {
                changed.enum_variants[0]
                    .error
                    .as_mut()
                    .expect("metadata")
                    .message = None;
            }
            2 => {
                changed.enums.clear();
                changed.enum_variants.clear();
                changed.debug_types.clear();
            }
            3 => {
                changed.enum_variants[0]
                    .error
                    .as_mut()
                    .expect("metadata")
                    .field_types[0] = crate::TypeIdentity::Boolean;
            }
            _ => {
                let mut duplicate = changed.enums[0].clone();
                duplicate.error = None;
                let owner = crate::EnumTypeId::new(changed.enums.len() as u16);
                changed.enums.push(duplicate);
                let mut rogue = changed.enum_variants[0].clone();
                rogue.owner = owner;
                rogue.error = None;
                changed.enum_variants.insert(0, rogue);
            }
        }
        assert!(
            RuntimeLayouts::build(&changed, address).is_err(),
            "mutation {mutation}"
        );
        if matches!(mutation, 0 | 1 | 4) {
            changed
                .verify()
                .expect("ordinary enums and unlabeled errors remain valid bytecode");
        }
    }
}

#[test]
fn unused_native_families_are_not_required() {
    for source in [
        "program T; begin end.",
        "program T; uses Std.Io.Fs as Fs; begin discard Fs.Exists('missing'); end.",
        "program T; uses Std.Text.Parse as Parse; begin discard Parse.Bool('true'); end.",
    ] {
        let mut image = image(source);
        if source.contains("Fs.Exists") {
            image.enums.clear();
            image.enum_variants.clear();
        }
        RuntimeLayouts::build(&image, InstructionAddress::new(0))
            .expect("only used native contracts are required");
        assert!(
            !image
                .enums
                .iter()
                .any(|e| image.strings.get(e.name) == Some("Std.Io.Fs.FsError")),
            "infallible Fs operations do not require FsError"
        );
    }
}

#[test]
fn acp_error_records_require_their_registered_nested_layouts() {
    let original =
        image("program T; uses Std.Ai.Acp.Types as Acp; begin discard Acp.ParseUInt64('1'); end.");
    let address = InstructionAddress::new(0);
    RuntimeLayouts::build(&original, address).expect("structured error contract");
    for name in ["Std.Ai.Acp.Types.RawJson", "Std.Ai.Acp.Types.ProtocolError"] {
        let mut changed = original.clone();
        let index = changed
            .records
            .iter()
            .position(|record| changed.strings.get(record.name) == Some(name))
            .expect("native record");
        let wrong = changed
            .debug_types
            .iter()
            .position(|ty| matches!(ty, crate::DebugType::Boolean))
            .expect("boolean type");
        changed.records[index].fields[0].ty = crate::DebugTypeId::new(wrong as u32);
        assert!(
            RuntimeLayouts::build(&changed, address).is_err(),
            "{name} field cannot change shape"
        );
    }
}

#[test]
fn hosted_families_require_the_exact_host_cause_contract() {
    for source in [
        "program T; uses Std.Io.Fs as Fs; begin discard Fs.ReadText('file'); end.",
        "program T; uses Std.Crypto as Crypto; begin discard Crypto.RandomBytes(0); end.",
    ] {
        let original = image(source);
        RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("host dependency");
        for mutation in 0..2 {
            let mut changed = original.clone();
            let host = changed
                .enums
                .iter()
                .position(|family| {
                    changed.strings.get(family.name) == Some("Std.Os.Errors.HostError")
                })
                .expect("host family");
            if mutation == 0 {
                changed.enums[host].error = None;
            } else {
                let outer = changed
                    .enum_variants
                    .iter_mut()
                    .find(|variant| {
                        variant
                            .error
                            .as_ref()
                            .is_some_and(|error| error.cause.is_some())
                    })
                    .expect("host cause");
                outer.error.as_mut().expect("metadata").cause = None;
            }
            assert!(RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err());
        }
    }
}

#[test]
fn selected_operations_require_their_native_contracts() {
    for (source, family) in [
        (
            "program T; uses Std.Encoding as Codec; begin discard Codec.HexDecode('!'); end.",
            "Std.Encoding.EncodingError",
        ),
        (
            "program T; uses Std.Tasks as Tasks; begin const Q: channel of integer := Tasks.CreateChannel(1); discard Tasks.TryReceive(Q); end.",
            "Std.Tasks.TaskError",
        ),
        (
            "program T; uses Std.Net.Sockets as Net; begin discard Net.Connect('localhost', 0, 1); end.",
            "Std.Net.Sockets.NetError",
        ),
        (
            "program T; uses Std.Net.Server as Server; begin discard Server.CreateLifetime(-1, false); end.",
            "Std.Net.Server.ServerError",
        ),
    ] {
        let original = image(source);
        RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("native contract");
        let mut changed = original.clone();
        let owner = changed
            .enums
            .iter_mut()
            .find(|owner| changed.strings.get(owner.name) == Some(family))
            .expect("native family");
        owner.error = None;
        assert!(
            RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err(),
            "{family}"
        );
    }
}

#[test]
fn server_listener_causes_admit_the_net_and_host_dependency_closure() {
    let original = image(
        "program T; uses Std.Net.Server as Server; begin discard Server.CreateLifetime(-1, false); end.",
    );
    for family in ["Std.Net.Sockets.NetError", "Std.Os.Errors.HostError"] {
        let mut changed = original.clone();
        let owner = changed
            .enums
            .iter_mut()
            .find(|owner| changed.strings.get(owner.name) == Some(family))
            .expect("Cause dependency");
        owner.error = None;
        assert!(
            RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err(),
            "{family}"
        );
    }
    let mut changed = original;
    let listener = changed
        .enum_variants
        .iter_mut()
        .find(|variant| changed.strings.get(variant.name) == Some("ListenerFailure"))
        .expect("listener cause");
    listener.error.as_mut().expect("essential metadata").cause = None;
    assert!(RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err());
}

#[test]
fn encoding_requires_its_exact_payload_without_host_dependencies() {
    let mut original =
        image("program T; uses Std.Encoding as Codec; begin discard Codec.HexDecode('!'); end.");
    RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("encoding ABI");
    assert!(
        !original
            .enums
            .iter()
            .any(|owner| original.strings.get(owner.name) == Some("Std.Os.Errors.HostError"))
    );
    let variant = original
        .enum_variants
        .iter_mut()
        .find(|variant| original.strings.get(variant.name) == Some("InvalidByte"))
        .expect("byte variant");
    variant
        .error
        .as_mut()
        .expect("essential metadata")
        .field_types[1] = crate::TypeIdentity::String;
    assert!(RuntimeLayouts::build(&original, InstructionAddress::new(0)).is_err());
}

#[test]
fn binary_file_operations_require_the_complete_filesystem_error_abi() {
    for source in [
        "program T; uses Std.Io.Fs as Fs; begin discard Fs.ReadBytes('file'); end.",
        "program T; uses Std.Io.Fs as Fs; begin discard Fs.WriteBytes('file', [0, 255]); end.",
    ] {
        let original = image(source);
        RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("binary Fs ABI");
        let variant = original
            .enum_variants
            .iter()
            .position(|variant| original.strings.get(variant.name) == Some("InvalidByte"))
            .expect("byte validation error is retained without a source constructor");
        let mut wrong_payload = original.clone();
        wrong_payload.enum_variants[variant]
            .error
            .as_mut()
            .expect("essential metadata")
            .field_types[1] = crate::TypeIdentity::String;
        assert!(RuntimeLayouts::build(&wrong_payload, InstructionAddress::new(0)).is_err());
        let mut missing_variant = original;
        missing_variant.enum_variants.remove(variant);
        assert!(RuntimeLayouts::build(&missing_variant, InstructionAddress::new(0)).is_err());
    }
}

#[test]
fn filesystem_lifecycle_errors_append_and_keep_operation_payloads() {
    let catalog = crate::stdlib::native_errors::NativeErrorFamily::Fs.variants();
    assert_eq!(catalog.len(), 15);
    assert_eq!(catalog[12].name, "InvalidByte");
    assert_eq!(catalog[13].name, "Closed");
    assert_eq!(catalog[14].name, "Cancelled");
    let original =
        image("program T; uses Std.Io.Fs as Fs; begin discard Fs.ReadText('missing'); end.");
    RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("filesystem lifecycle ABI");
    let owner = original
        .enums
        .iter()
        .position(|owner| original.strings.get(owner.name) == Some("Std.Io.Fs.FsError"))
        .expect("filesystem family");
    for name in ["Closed", "Cancelled"] {
        let mut changed = original.clone();
        let variant = changed
            .enum_variants
            .iter_mut()
            .find(|variant| {
                usize::from(variant.owner.get()) == owner
                    && changed.strings.get(variant.name) == Some(name)
            })
            .expect("lifecycle variant");
        variant
            .error
            .as_mut()
            .expect("essential lifecycle metadata")
            .field_types[0] = crate::TypeIdentity::Integer;
        assert!(
            RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err(),
            "{name}"
        );
    }
}

#[test]
fn process_control_errors_retain_partial_output_and_original_host_causes() {
    let catalog = crate::stdlib::native_errors::NativeErrorFamily::Proc.variants();
    assert_eq!(catalog.len(), 8);
    assert_eq!(catalog[0].name, "StartFailed");
    assert_eq!(catalog[3].name, "IoFailure");
    assert_eq!(catalog[4].name, "Timeout");
    assert_eq!(catalog[5].name, "Cancelled");
    assert_eq!(catalog[6].name, "LimitExceeded");
    assert_eq!(catalog[7].name, "ExecutionFailed");
    let original = image(
        "program T; uses Std.Os.Proc as Proc; begin discard Proc.RunCaptureWithOptions('tool', [], Proc.DefaultOptions()); end.",
    );
    RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("process control ABI");
    let owner = original
        .enums
        .iter()
        .position(|owner| original.strings.get(owner.name) == Some("Std.Os.Proc.ProcError"))
        .expect("process error owner");
    for name in ["Timeout", "Cancelled", "LimitExceeded", "ExecutionFailed"] {
        let mut changed = original.clone();
        let variant = changed
            .enum_variants
            .iter_mut()
            .find(|variant| {
                usize::from(variant.owner.get()) == owner
                    && changed.strings.get(variant.name) == Some(name)
            })
            .expect("control error");
        let error = variant.error.as_mut().expect("essential process metadata");
        if name == "ExecutionFailed" {
            error.cause = None;
        } else {
            error.field_types[1] = crate::TypeIdentity::Integer;
        }
        assert!(
            RuntimeLayouts::build(&changed, InstructionAddress::new(0)).is_err(),
            "{name}"
        );
    }
}

#[test]
fn pure_process_defaults_do_not_require_process_or_host_error_metadata() {
    let mut defaults =
        image("program T; uses Std.Os.Proc as Proc; begin discard Proc.DefaultOptions(); end.");
    for owner in &mut defaults.enums {
        owner.error = None;
    }
    for variant in &mut defaults.enum_variants {
        variant.error = None;
    }
    RuntimeLayouts::build(&defaults, InstructionAddress::new(0))
        .expect("pure record construction does not admit unused error families");
}
