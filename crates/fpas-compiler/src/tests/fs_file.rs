//! Opaque file metadata preserves private construction through portable images.

use super::parse_ok;
use fpas_runtime::VerifiedExecutable;
use fpas_runtime::execution::{DebugErrorKind, DebugExpression, DebugSession};

const SOURCE: &str = r#"program FileMetadata;
uses Std.Io.Fs as Fs;
type FileAlias = Fs.File;
var Marker: integer := 0;
function Keep(Value: FileAlias): FileAlias;
begin return Value; end function;
begin
  Marker := Marker + 1;
  if Marker <> 1 then panic('marker'); end if;
end."#;

fn portable(executable: VerifiedExecutable) -> VerifiedExecutable {
    let image = executable.executable();
    let paths = image
        .source_map
        .sources
        .iter()
        .map(|id| image.strings.get(*id).expect("source path").to_owned())
        .collect::<Vec<_>>();
    let hashes = paths
        .iter()
        .map(|path| fpas_program::Digest::of(path.as_bytes()))
        .collect();
    let image = fpas_program::ProgramImage::new(
        fpas_program::ProgramIdentity {
            contracts_enabled: true,
            compiler_version: "opaque-file-fixture".into(),
            bytecode_version: fpas_runtime::BYTECODE_VERSION,
            source_hash: fpas_program::Digest::of(SOURCE.as_bytes()),
            options_hash: fpas_program::Digest::of(b"default"),
            units: vec![],
        },
        paths,
        hashes,
        executable,
    )
    .expect("portable image");
    let bytes = fpas_program::encode(&image).expect("encode image");
    fpas_program::decode(&bytes)
        .expect("decode image")
        .executable()
        .clone()
}

#[test]
fn file_constructor_privacy_survives_portable_debug_metadata() {
    let original = crate::compile(&parse_ok(SOURCE)).expect("file type metadata");
    for executable in [original.clone(), portable(original)] {
        let image = executable.executable();
        let record = image
            .records
            .iter()
            .find(|record| image.strings.get(record.name) == Some("Std.Io.Fs.File"))
            .expect("file layout");
        let construction = record.construction.as_ref().expect("construction metadata");
        assert!(
            construction.requires_owner,
            "opaque handles require their intrinsic owner"
        );
        assert_eq!(
            construction.owner_unit.and_then(|id| image.strings.get(id)),
            Some("Std.Io.Fs")
        );
        let mut session = DebugSession::new(executable).expect("debug session");
        // Construction metadata is available at the initial entry stop.
        let frame = session.stack(0, 1).expect("stack").items[0].id;
        for name in ["Fs.File", "FileAlias"] {
            let error = session
                .evaluate(
                    &DebugExpression::Call {
                        callee: Box::new(DebugExpression::Callable(name.into())),
                        arguments: vec![],
                    },
                    Some(frame),
                )
                .expect_err("opaque constructor must be private");
            if name == "Fs.File" && error.kind == DebugErrorKind::UnknownCallable {
                // Intrinsic import aliases need not expose a debugger constructor target.
                continue;
            }
            assert_eq!(error.kind, DebugErrorKind::EvaluationType, "{error:?}");
            assert!(error.message.contains("private fields"), "{error:?}");
        }
    }
}
