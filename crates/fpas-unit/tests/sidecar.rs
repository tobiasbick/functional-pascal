//! Source-adjacent `.fpascu` lifecycle tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "filesystem integration fixtures use expect for compact setup"
)]

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use fpas_bytecode::Op;
use fpas_unit::interface::{
    InterfaceSymbol, InterfaceType, SymbolKind, UnitInterface, encode_interface,
};
use fpas_unit::object::{
    ChunkConstant as ObjectConstant, ChunkLocation as ObjectLocation,
    ChunkObject as RelocatableObject, encode_chunk_object as encode_object,
};
use fpas_unit::{
    CompiledUnit, DependencyIdentity, Digest, ExpectedUnitIdentity, IncompatibilityReason,
    InvalidationReason, MAX_SIDECAR_BYTES, SidecarCorruption, SidecarLoad, UnitIdentity, encode,
    load_sidecar, sidecar_path, write_sidecar,
};

fn temp_dir(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-unit-sidecar-{name}-{}-{id}",
        std::process::id()
    ))
}

fn expected() -> ExpectedUnitIdentity {
    ExpectedUnitIdentity {
        unit_name: "demo.core".to_string(),
        source_hash: Digest::of(b"source"),
        compiler_version: "compiler-test".to_string(),
        bytecode_version: 3,
        options_hash: Digest::of(b"options"),
        dependencies: vec![DependencyIdentity {
            unit_name: "demo.base".to_string(),
            interface_hash: Digest::of(b"base-interface"),
        }],
    }
}

fn compiled(object: &[u8]) -> CompiledUnit {
    compiled_payload("demo.core", "demo.core", Vec::new(), object)
}

fn compiled_payload(
    interface_unit: &str,
    object_owner: &str,
    symbols: Vec<InterfaceSymbol>,
    marker: &[u8],
) -> CompiledUnit {
    let expected = expected();
    let interface = encode_interface(&UnitInterface {
        unit_name: interface_unit.to_string(),
        symbols,
    })
    .expect("interface fixture");
    let object = encode_object(&RelocatableObject {
        owner: object_owner.to_string(),
        code: vec![Op::Halt],
        constants: vec![ObjectConstant::String(
            String::from_utf8_lossy(marker).into_owned(),
        )],
        locations: vec![ObjectLocation {
            line: 1,
            column: 1,
            source_id: 0,
        }],
        functions: BTreeMap::new(),
        definitions: Vec::new(),
        imports: Vec::new(),
        relocations: Vec::new(),
    })
    .expect("object fixture");
    CompiledUnit {
        identity: UnitIdentity {
            unit_name: expected.unit_name,
            source_hash: expected.source_hash,
            interface_hash: Digest::of(&interface),
            object_hash: Digest::of(&object),
            compiler_version: expected.compiler_version,
            bytecode_version: expected.bytecode_version,
            options_hash: expected.options_hash,
            dependencies: expected.dependencies,
        },
        interface,
        object,
    }
}

fn create_source(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("temporary directory");
    let source = dir.join("Core.fpas");
    fs::write(&source, "unit Demo.Core;\n").expect("source fixture");
    source
}

#[test]
fn sidecar_path_replaces_source_extension() {
    assert_eq!(
        sidecar_path(Path::new("src/Demo.Core.fpas")),
        PathBuf::from("src/Demo.Core.fpascu")
    );
}

#[test]
fn missing_sidecar_is_classified_without_error() {
    let dir = temp_dir("missing");
    let source = create_source(&dir);

    let loaded = load_sidecar(&source, &expected()).expect("missing is a normal outcome");

    assert_eq!(loaded, SidecarLoad::Missing);
    fs::remove_dir_all(dir).ok();
}

#[test]
fn written_sidecar_is_reusable() {
    let dir = temp_dir("reuse");
    let source = create_source(&dir);
    let unit = compiled(b"object");

    let written = write_sidecar(&source, &unit).expect("sidecar write");
    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    assert_eq!(written, dir.join("Core.fpascu"));
    let SidecarLoad::Reusable(loaded) = loaded else {
        panic!("valid sidecar must be reusable");
    };
    assert_eq!(loaded.compiled, unit);
    assert_eq!(loaded.interface.unit_name, "demo.core");
    assert_eq!(loaded.object.owner, "demo.core");
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_source_is_classified_as_stale() {
    let dir = temp_dir("stale-source");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.source_hash = Digest::of(b"changed");

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(loaded, SidecarLoad::Stale(InvalidationReason::Source));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_dependency_interface_is_classified_as_stale() {
    let dir = temp_dir("stale-dependency");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.dependencies[0].interface_hash = Digest::of(b"changed");

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(loaded, SidecarLoad::Stale(InvalidationReason::Dependencies));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_bytecode_version_is_incompatible() {
    let dir = temp_dir("bytecode");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.bytecode_version += 1;

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(
        loaded,
        SidecarLoad::Incompatible(IncompatibilityReason::Bytecode)
    );
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_compiler_identity_is_incompatible() {
    let dir = temp_dir("compiler");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.compiler_version = "other-compiler".to_string();

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(
        loaded,
        SidecarLoad::Incompatible(IncompatibilityReason::Compiler)
    );
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_options_are_classified_as_stale() {
    let dir = temp_dir("options");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.options_hash = Digest::of(b"other-options");

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(loaded, SidecarLoad::Stale(InvalidationReason::Options));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn changed_unit_name_is_classified_as_stale() {
    let dir = temp_dir("unit-name");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"object")).expect("sidecar write");
    let mut changed = expected();
    changed.unit_name = "demo.renamed".to_string();

    let loaded = load_sidecar(&source, &changed).expect("sidecar load");

    assert_eq!(loaded, SidecarLoad::Stale(InvalidationReason::UnitName));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn unsupported_envelope_version_is_incompatible() {
    let dir = temp_dir("format-version");
    let source = create_source(&dir);
    let mut bytes = encode(&compiled(b"object")).expect("encoding");
    bytes[8..10].copy_from_slice(&42_u16.to_le_bytes());
    fs::write(sidecar_path(&source), bytes).expect("sidecar fixture");

    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    assert_eq!(
        loaded,
        SidecarLoad::Incompatible(IncompatibilityReason::FormatVersion(42))
    );
    fs::remove_dir_all(dir).ok();
}

#[test]
fn corrupt_sidecar_is_classified_for_rebuild() {
    let dir = temp_dir("corrupt");
    let source = create_source(&dir);
    fs::write(sidecar_path(&source), b"not a compiled unit").expect("corrupt fixture");

    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    assert!(matches!(loaded, SidecarLoad::Corrupt(_)));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn replacing_sidecar_keeps_a_complete_valid_object() {
    let dir = temp_dir("replace");
    let source = create_source(&dir);
    write_sidecar(&source, &compiled(b"old")).expect("first write");
    let replacement = compiled(b"new");

    write_sidecar(&source, &replacement).expect("replacement write");
    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    let SidecarLoad::Reusable(loaded) = loaded else {
        panic!("replacement sidecar must be reusable");
    };
    assert_eq!(loaded.compiled, replacement);
    fs::remove_dir_all(dir).ok();
}

#[test]
fn concurrent_writers_publish_one_complete_object() {
    let dir = temp_dir("concurrent");
    let source = Arc::new(create_source(&dir));
    let first_source = Arc::clone(&source);
    let second_source = Arc::clone(&source);
    let first = thread::spawn(move || write_sidecar(&first_source, &compiled(b"first")));
    let second = thread::spawn(move || write_sidecar(&second_source, &compiled(b"second")));

    first.join().expect("first writer").expect("first write");
    second.join().expect("second writer").expect("second write");
    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    let SidecarLoad::Reusable(unit) = loaded else {
        panic!("final sidecar must be reusable");
    };
    assert!(
        unit.object.constants == vec![ObjectConstant::String("first".to_string())]
            || unit.object.constants == vec![ObjectConstant::String("second".to_string())]
    );
    fs::remove_dir_all(dir).ok();
}

#[test]
fn oversized_sidecar_is_rejected_from_metadata() {
    let dir = temp_dir("oversized");
    let source = create_source(&dir);
    let path = sidecar_path(&source);
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("sparse sidecar fixture");
    file.set_len(MAX_SIDECAR_BYTES as u64 + 1)
        .expect("oversized sparse sidecar");

    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");

    assert!(matches!(
        loaded,
        SidecarLoad::Corrupt(SidecarCorruption::Format(
            fpas_unit::FormatError::LimitExceeded { field: "file", .. }
        ))
    ));
    fs::remove_dir_all(dir).ok();
}

#[test]
fn mismatched_interface_unit_name_is_corrupt() {
    let loaded = load_payload_fixture(compiled_payload(
        "demo.other",
        "demo.core",
        Vec::new(),
        b"object",
    ));

    assert!(matches!(
        loaded,
        SidecarLoad::Corrupt(SidecarCorruption::InterfaceUnitName { .. })
    ));
}

#[test]
fn mismatched_object_owner_is_corrupt() {
    let loaded = load_payload_fixture(compiled_payload(
        "demo.core",
        "demo.other",
        Vec::new(),
        b"object",
    ));

    assert!(matches!(
        loaded,
        SidecarLoad::Corrupt(SidecarCorruption::ObjectOwner { .. })
    ));
}

#[test]
fn case_insensitive_duplicate_interface_symbols_are_corrupt() {
    let symbols = ["Value", "value"]
        .into_iter()
        .map(|name| InterfaceSymbol {
            name: name.to_string(),
            qualified_name: format!("demo.core.{name}"),
            ty: InterfaceType::Integer,
            kind: SymbolKind::Variable,
        })
        .collect();
    let loaded = load_payload_fixture(compiled_payload(
        "demo.core",
        "demo.core",
        symbols,
        b"object",
    ));

    assert!(matches!(
        loaded,
        SidecarLoad::Corrupt(SidecarCorruption::DuplicateSymbol(_))
    ));
}

fn load_payload_fixture(unit: CompiledUnit) -> SidecarLoad {
    let dir = temp_dir("payload");
    let source = create_source(&dir);
    fs::write(
        sidecar_path(&source),
        encode(&unit).expect("sidecar encoding"),
    )
    .expect("sidecar fixture");
    let loaded = load_sidecar(&source, &expected()).expect("sidecar load");
    fs::remove_dir_all(dir).ok();
    loaded
}
