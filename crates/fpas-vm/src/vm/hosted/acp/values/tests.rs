//! Exercise the actual compiler layouts with independent SDK wire fixtures.

#[path = "../../../../../../fpas-acp/tests/methods/v1.rs"]
mod v1;
#[path = "../../../../../../fpas-acp/tests/methods/v2.rs"]
mod v2;
#[path = "../../../../../../fpas-acp/tests/methods/v2_updates.rs"]
mod v2_updates;

use super::{Converter, RuntimeLayouts};
use fpas_acp::{
    RawJson,
    binding::{Type, catalog},
    schema::Version,
};
use fpas_bytecode::{DebugType, DebugTypeId, Executable, InstructionAddress, Value};

fn image() -> Executable {
    let (program, errors) = fpas_parser::parse(
        "program NativeAcp; uses Std.Acp.Types as Acp, Std.Acp.V2 as V2; begin discard Acp.DecodeClientRequest('initialize', Acp.RawJson(Text := '{}')); discard V2.DecodeClientRequest('initialize', Acp.RawJson(Text := '{}')); end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program)
        .expect("native protocol layouts")
        .executable()
        .clone()
}

#[test]
fn every_method_crosses_the_actual_vm_layout_boundary_in_both_directions() {
    let image = image();
    let layouts = RuntimeLayouts::build(&image, InstructionAddress::new(0)).unwrap();
    for (version, fixtures) in [(Version::V1, v1::FIXTURES), (Version::V2, v2::FIXTURES)] {
        let catalog = catalog(version).unwrap();
        for &(root, method, text) in fixtures {
            let decoded = catalog
                .decode_message(root, method, RawJson::parse(text).unwrap())
                .unwrap();
            let mut converter = Converter {
                catalog,
                image: &image,
                layouts: &layouts,
                depth: 0,
            };
            let ty = Type::Named(root.into());
            let native = converter
                .pack(&ty, decoded.payload.clone(), None)
                .unwrap_or_else(|error| panic!("pack {version:?} {method}: {error}"));
            let detached = converter
                .unpack(&ty, &native, None)
                .unwrap_or_else(|error| panic!("unpack {version:?} {method}: {error}"));
            assert_eq!(detached, decoded.payload, "{version:?} {method}");
            catalog.encode_message(root, method, &detached).unwrap();
        }
    }
}

#[test]
fn malformed_native_values_and_dto_metadata_fail_without_publication() {
    let original = image();
    let catalog = catalog(Version::V2).unwrap();
    let ty = Type::Named("AgentNotification".into());
    let decoded = catalog.decode_message("AgentNotification", "session/update", RawJson::parse(r#"{"sessionId":"s","update":{"sessionUpdate":"tool_call_update","toolCallId":"t","content":[]}}"#).unwrap()).unwrap();
    let layouts = RuntimeLayouts::build(&original, InstructionAddress::new(0)).unwrap();
    let mut converter = Converter {
        catalog,
        image: &original,
        layouts: &layouts,
        depth: 0,
    };
    assert!(converter.unpack(&ty, &Value::Integer(1), None).is_err());
    let native = converter.pack(&ty, decoded.payload.clone(), None).unwrap();
    let mut corrupted = native.clone();
    assert!(corrupt_array(&mut corrupted));
    assert!(
        converter.unpack(&ty, &corrupted, None).is_err(),
        "erased generic fields still validate their concrete payload"
    );
    assert!(
        converter
            .unpack(&Type::Named("ClientRequest".into()), &native, None)
            .is_err()
    );
    for mutation in 0..3 {
        let mut image = original.clone();
        let index = image
            .records
            .iter()
            .position(|record| image.strings.get(record.name) == Some("Std.Acp.V2.ToolCallUpdate"))
            .unwrap();
        let field = image.records[index]
            .fields
            .iter()
            .position(|field| image.strings.get(field.name) == Some("Content"))
            .unwrap();
        match mutation {
            0 => {
                image.records[index].fields[field].ty = DebugTypeId::try_from_index(
                    image
                        .debug_types
                        .iter()
                        .position(|ty| *ty == DebugType::Integer)
                        .unwrap(),
                )
                .unwrap()
            }
            1 => {
                image.records[index].fields.remove(field);
            }
            _ => {
                image.records[index].fields[field].name = image.records[index].name;
            }
        }
        let layouts = RuntimeLayouts::build(&image, InstructionAddress::new(0)).unwrap();
        let mut converter = Converter {
            catalog,
            image: &image,
            layouts: &layouts,
            depth: 0,
        };
        assert!(
            converter.pack(&ty, decoded.payload.clone(), None).is_err(),
            "metadata mutation {mutation}"
        );
    }
}

fn corrupt_array(value: &mut Value) -> bool {
    match value {
        Value::Array(_) => {
            *value = Value::Integer(1);
            true
        }
        Value::Record(record) => record.values_mut().iter_mut().any(corrupt_array),
        Value::Enum(enumeration) => enumeration.values_mut().iter_mut().any(corrupt_array),
        _ => false,
    }
}

#[test]
fn v2_update_messages_terminals_and_patches_cross_native_layouts() {
    let image = image();
    let layouts = RuntimeLayouts::build(&image, InstructionAddress::new(0)).unwrap();
    let catalog = catalog(Version::V2).unwrap();
    let mut converter = Converter {
        catalog,
        image: &image,
        layouts: &layouts,
        depth: 0,
    };
    for text in v2_updates::UPDATES {
        let ty = Type::Named("SessionUpdate".into());
        let detached = catalog
            .decode("SessionUpdate", &RawJson::parse(text).unwrap())
            .unwrap();
        let native = converter
            .pack(&ty, detached.clone(), None)
            .unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(converter.unpack(&ty, &native, None).unwrap(), detached);
    }
}
