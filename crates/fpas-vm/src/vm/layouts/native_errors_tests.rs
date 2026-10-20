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
    let original = image("program T; uses Std.Parse as Parse; begin discard Parse.Int('1'); end.");
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
                    .field_types[0] = fpas_bytecode::TypeIdentity::Boolean;
            }
            _ => {
                let mut duplicate = changed.enums[0].clone();
                duplicate.error = None;
                let owner = fpas_bytecode::EnumTypeId::new(changed.enums.len() as u16);
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
        "program T; uses Std.Fs as Fs; begin discard Fs.Exists('missing'); end.",
        "program T; uses Std.Parse as Parse; begin discard Parse.Bool('true'); end.",
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
                .any(|e| image.strings.get(e.name) == Some("Std.Fs.FsError")),
            "infallible Fs operations do not require FsError"
        );
    }
}

#[test]
fn hosted_families_require_the_exact_host_cause_contract() {
    for source in [
        "program T; uses Std.Fs as Fs; begin discard Fs.ReadText('file'); end.",
        "program T; uses Std.Crypto as Crypto; begin discard Crypto.RandomBytes(0); end.",
    ] {
        let original = image(source);
        RuntimeLayouts::build(&original, InstructionAddress::new(0)).expect("host dependency");
        for mutation in 0..2 {
            let mut changed = original.clone();
            let host = changed
                .enums
                .iter()
                .position(|family| changed.strings.get(family.name) == Some("Std.Errors.HostError"))
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
