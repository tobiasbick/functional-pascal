//! Public host provenance for handles returned by the actual HTTP source library.

#![allow(
    clippy::expect_used,
    reason = "host boundary fixtures require concrete results"
)]

use std::path::PathBuf;
use std::sync::Arc;

use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;
use fpas_runtime::execution::{
    CallbackSession, HostArgument, HostChild, HostValue, HostValueError,
};
use fpas_runtime::{FunctionId, Value, VerifiedExecutable};
use fpas_unit::interface::UnitInterface;

fn image(source: &str) -> Arc<VerifiedExecutable> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lib");
    let project =
        fpas_project::load_standard_library_project(&root).expect("actual standard library");
    let sources = project
        .source_files
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).expect("standard library source");
            let (parsed, errors) = fpas_parser::parse_compilation_unit(&source);
            assert!(errors.is_empty(), "{}: {errors:?}", path.display());
            let unit = match parsed {
                fpas_parser::CompilationUnit::Unit(unit) => Some(unit),
                _ => None,
            }
            .expect("standard library source declares a unit");
            (path.clone(), unit)
        })
        .collect();
    let graph = fpas_project::build_unit_graph_for_program_from_parsed_sources(
        &root.join("host-provenance.fpas"),
        sources,
        &project.link_meta,
    )
    .expect("trusted source unit graph");
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let selection = fpas_project::resolve_program_units(&graph, &program.uses, None)
        .expect("actual HTTP unit dependencies");
    // Compile the authoritative parsed sources directly. This fixture exercises the real
    // source library and object linker without reading or publishing cached sidecars.
    let mut interfaces = Vec::new();
    let mut objects = Vec::new();
    for name in selection.order() {
        let unit = graph.get(name).expect("resolved source unit").parsed_unit();
        let direct = direct_interfaces(&unit.uses, &interfaces);
        let compiled = fpas_compiler::compile_unit_object_with_support(unit, &direct, &interfaces)
            .expect("actual HTTP dependency compiles");
        interfaces.push(compiled.interface);
        objects.push(compiled.object);
    }
    let direct = direct_interfaces(&program.uses, &interfaces);
    let program =
        fpas_compiler::compile_program_object_with_support(&program, &direct, &interfaces)
            .expect("HTTP host fixture compiles");
    Arc::new(fpas_linker::link_objects(&objects, &program).expect("HTTP source objects link"))
}

fn direct_interfaces(
    uses: &[fpas_parser::Import],
    interfaces: &[UnitInterface],
) -> Vec<UnitInterface> {
    uses.iter()
        .filter_map(|used| {
            let name = used.unit.parts.join(".");
            interfaces
                .iter()
                .find(|interface| interface.unit_name.eq_ignore_ascii_case(&name))
                .cloned()
        })
        .collect()
}

fn function(image: &VerifiedExecutable, name: &str) -> FunctionId {
    let index = image
        .executable()
        .functions
        .iter()
        .position(|info| {
            image
                .executable()
                .strings
                .get(info.name)
                .is_some_and(|declared| declared.eq_ignore_ascii_case(name))
        })
        .expect("host fixture function");
    FunctionId::try_from_index(index).expect("function id")
}

fn invoke(
    session: &mut CallbackSession,
    image: &VerifiedExecutable,
    name: &str,
    arguments: Vec<HostArgument>,
) -> HostValue {
    session
        .invoke(function(image, name), arguments)
        .expect("host callback")
        .value
}

#[test]
fn real_http_exports_keep_provenance_through_nested_aggregates() {
    let image = image(
        r#"
program HostHttp;
uses Std.Net.Http as Http, Std.Net.Http.Handles as Handles;
function NewDecoder(): Http.SseDecoder;
begin
  return Http.CreateSseDecoder(1024).Unwrap();
end function;
function Feed(Decoder: Http.SseDecoder): integer;
begin
  return Http.FeedSse(Decoder, [100, 97, 116, 97, 58, 120, 10, 10]).Unwrap().Length();
end function;
function Wrapped(): option of array of Http.SseDecoder;
begin
  return Some([NewDecoder()]);
end function;
function Echo(Value: option of array of Http.SseDecoder): option of array of Http.SseDecoder;
begin
  return Value;
end function;
function Body(): Handles.BodyStream;
begin
  return Handles.NewBodyStream(1);
end function;
function EchoBody(Value: Handles.BodyStream): Handles.BodyStream;
begin
  return Value;
end function;
begin
end.
"#,
    );
    let mut first = CallbackSession::from_shared(Arc::clone(&image));
    let mut second = CallbackSession::from_shared(Arc::clone(&image));
    first
        .invoke(image.executable().entry, vec![])
        .expect("first unit initialization");
    second
        .invoke(image.executable().entry, vec![])
        .expect("second unit initialization");
    let decoder = invoke(&mut first, &image, "NewDecoder", vec![]);
    // Both registries allocate slot one; this must not make their handles interchangeable.
    let second_decoder = invoke(&mut second, &image, "NewDecoder", vec![]);
    assert_eq!(
        decoder
            .child(HostChild::Field("slot"))
            .expect("slot")
            .expect("slot field")
            .try_to_data()
            .expect("integer"),
        Value::Integer(1)
    );
    assert_eq!(
        second_decoder
            .child(HostChild::Field("slot"))
            .expect("slot")
            .expect("slot field")
            .try_to_data()
            .expect("integer"),
        Value::Integer(1)
    );
    assert_eq!(decoder.try_to_data(), Err(HostValueError::SessionBound));
    assert_eq!(
        invoke(&mut first, &image, "Feed", vec![decoder.clone().into()])
            .try_to_data()
            .expect("event count"),
        Value::Integer(1)
    );
    assert_eq!(
        second
            .invoke(function(&image, "Feed"), vec![decoder.into()])
            .expect_err("foreign decoder with same slot")
            .code,
        RUNTIME_INVALID_TASK
    );
    let wrapped = invoke(&mut first, &image, "Wrapped", vec![]);
    assert_eq!(wrapped.try_to_data(), Err(HostValueError::SessionBound));
    assert_eq!(
        second
            .invoke(function(&image, "Echo"), vec![wrapped.clone().into()])
            .expect_err("nested HTTP handle")
            .code,
        RUNTIME_INVALID_TASK
    );
    let selected = wrapped
        .child(HostChild::Payload)
        .expect("payload")
        .expect("some")
        .child(HostChild::Index(0))
        .expect("item")
        .expect("decoder");
    assert_eq!(
        invoke(&mut first, &image, "Feed", vec![selected.into()])
            .try_to_data()
            .expect("event count"),
        Value::Integer(1)
    );
    let body = invoke(&mut first, &image, "Body", vec![]);
    assert_eq!(body.try_to_data(), Err(HostValueError::SessionBound));
    assert!(
        first
            .invoke(function(&image, "EchoBody"), vec![body.clone().into()])
            .is_ok()
    );
    assert_eq!(
        second
            .invoke(function(&image, "EchoBody"), vec![body.into()])
            .expect_err("foreign body stream")
            .code,
        RUNTIME_INVALID_TASK
    );
}
