//! Source-less native-error construction, concrete identities and image transport.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::{compiled_unit, parse_ok};
use fpas_unit::object::{decode_object, encode_object};

#[test]
fn native_errors_keep_transitive_generic_factories_through_cached_units_and_images() {
    let errors = compiled_unit(
        r#"
unit Demo.Errors;
public type FirstError = error Failed; end error;
public type FirstAlias = FirstError;
public type AnotherFirstAlias = FirstAlias;
public type SecondError = error Failed; end error;
public type ContextError of E: Error = error
  Wrapped(Cause: E; Context: string) = 'Wrapped failure';
end error;
public type DataError of T: Printable = error Tagged(Data: T); end error;
public type MarkerError of T: Printable = error Failed; end error;
end unit;
"#,
        &[],
    );
    let builders = compiled_unit(
        r#"
unit Demo.Builders;
uses Demo.Errors as Domain;
type PrivateError of T: Printable = error Tagged(Data: T); end error;
function Build<X: Error>(Cause: X): Domain.ContextError of X;
begin return Domain.ContextError.Wrapped(Cause, 'cached'); end function;
public function Wrap<E: Error>(Value: E): Domain.ContextError of E;
begin return Build(Value); end function;
public function RecursiveWrap<E: Error>(Value: E; Depth: integer): Domain.ContextError of E;
begin
  if Depth = 0 then return Wrap(Value); end if;
  return RecursiveWrap(Value, Depth - 1);
end function;
public function Tag<T: Printable>(Value: T): Domain.DataError of T;
begin return Domain.DataError.Tagged(Value); end function;
public function DescribePrivate<T: Printable>(Value: T): string;
begin return PrivateError.Tagged(Value).Message(); end function;
public function Mark<T: Printable>(Value: T): Domain.MarkerError of T;
begin return Domain.MarkerError.Failed; end function;
end unit;
"#,
        std::slice::from_ref(&errors.interface),
    );
    let relay = compiled_unit(
        r#"
unit Demo.Relay;
uses Demo.Errors as Types, Demo.Builders as Builders;
public function Wrap<C: Error>(Value: C): Types.ContextError of C;
begin return Builders.RecursiveWrap(Value, 2); end function;
end unit;
"#,
        &[errors.interface.clone(), builders.interface.clone()],
    );
    let interfaces = vec![errors.interface, builders.interface, relay.interface];
    let root = crate::compile_program_object_with_support(&parse_ok(r#"
program CachedErrors;
uses Demo.Errors as Domain, Demo.Builders as Builders, Demo.Relay as Relay;
type FirstAlias = Domain.FirstError;
type WrappedAlias = Domain.ContextError of FirstAlias;
function Cause<E: Error>(Value: Domain.ContextError of E): E;
begin return case Value of when Domain.ContextError.Wrapped(const Original, _): Original; end case; end function;
begin
  const First := Relay.Wrap(Domain.FirstError.Failed);
  const Second := Relay.Wrap(Domain.SecondError.Failed);
  const Bound: function(Value: FirstAlias): WrappedAlias := Relay.Wrap;
  const Aliased := Bound(FirstAlias.Failed);
  if First.Message() <> Aliased.Message() then panic('alias changed error identity or output'); end if;
  if First.Message() <> 'Demo.Errors.ContextError.Wrapped: Wrapped failure'#10'  Context = "cached"'#10'caused by Demo.Errors.FirstError.Failed' then panic('canonical output'); end if;
  if not (First is Domain.ContextError.Wrapped(Domain.FirstError.Failed, 'cached')) then panic('typed first cause'); end if;
  if not (Second is Domain.ContextError.Wrapped(Domain.SecondError.Failed, 'cached')) then panic('typed second cause'); end if;
  if Cause(Second) <> Domain.SecondError.Failed then panic('generic pattern'); end if;
  if Builders.Tag(7).Message() <> 'Demo.Errors.DataError.Tagged'#10'  Data = 7' then panic('integer data argument'); end if;
  if Builders.Tag('seven').Message() <> 'Demo.Errors.DataError.Tagged'#10'  Data = "seven"' then panic('string data argument'); end if;
  if Builders.DescribePrivate(8) <> 'Demo.Builders.PrivateError.Tagged'#10'  Data = 8' then panic('hidden private schema'); end if;
  if Builders.Mark(7).Message() <> 'Demo.Errors.MarkerError.Failed' then panic('fieldless generic integer'); end if;
  if Builders.Mark('seven').Message() <> 'Demo.Errors.MarkerError.Failed' then panic('fieldless generic string'); end if;
end.
"#), &interfaces, &interfaces).expect("source-less consumer");
    let objects = [errors.object, builders.object, relay.object]
        .into_iter()
        .map(|object| {
            decode_object(&encode_object(&object).expect("encode cached object"))
                .expect("decode cached object")
        })
        .collect::<Vec<_>>();
    let executable = fpas_linker::link_objects(&objects, &root).expect("cached objects link");
    let image = executable.executable();
    let instances = image
        .enums
        .iter()
        .filter_map(|layout| layout.error.as_ref())
        .filter(|key| key.declaration == "demo.errors.contexterror")
        .collect::<Vec<_>>();
    assert_eq!(instances.len(), 2, "aliases reuse concrete error instances");
    assert_ne!(
        instances[0], instances[1],
        "different cause families retain distinct identities"
    );
    let markers = image
        .enums
        .iter()
        .filter_map(|layout| layout.error.as_ref())
        .filter(|key| key.declaration == "demo.errors.markererror")
        .collect::<Vec<_>>();
    assert_eq!(
        markers.len(),
        2,
        "fieldless families retain applied arguments"
    );
    fpas_vm::Vm::new(executable.clone())
        .run()
        .expect("cached native errors execute");

    // Remove the optional source-type information for every error payload. Essential
    // semantic fields and causes must survive independently of those debugger types.
    let mut stripped = executable.into_unverified();
    for ty in &mut stripped.debug_types {
        if matches!(ty, fpas_bytecode::DebugType::Enum(_)) {
            *ty = fpas_bytecode::DebugType::Dynamic;
        }
    }
    for variant in &mut stripped.enum_variants {
        variant.field_types.fill(fpas_bytecode::DebugTypeId::new(0));
    }
    let paths = stripped
        .source_map
        .sources
        .iter()
        .map(|id| stripped.strings.get(*id).expect("source label").to_owned())
        .collect::<Vec<_>>();
    let hashes = paths
        .iter()
        .map(|path| fpas_program::Digest::of(path.as_bytes()))
        .collect();
    let image = fpas_program::ProgramImage::new(
        fpas_program::ProgramIdentity {
            contracts_enabled: true,
            compiler_version: "native-error-source-fixture".into(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: fpas_program::Digest::of(b"source"),
            options_hash: fpas_program::Digest::of(b"options"),
            units: vec![],
        },
        paths,
        hashes,
        stripped.verify().expect("image without error debug types"),
    )
    .expect("portable image");
    let bytes = fpas_program::encode(&image).expect("encode image");
    let decoded = fpas_program::decode(&bytes).expect("decode image");
    fpas_vm::Vm::new(decoded.executable().clone())
        .run()
        .expect("native errors execute after image transport");
}

#[test]
fn capturing_closures_preserve_applied_error_parameters_and_results() {
    super::assert_succeeds(
        r#"
program CapturedErrors;
uses Std.Test as Test;
type First = error Item(Number: integer); end error;
type Alias = First;
type Second = error Item(Number: integer); end error;
type Wrapped of E: Error = error Item(Cause: E; Context: string); end error;
function Make(Context: string): function(Cause: Alias): Wrapped of First;
begin
  return function(Cause: Alias): Wrapped of First begin
    return Wrapped.Item(Cause, Context);
  end function;
end function;
begin
  const LabelText := 'second';
  const A := Make('first');
  const B: function(Cause: Second): Wrapped of Second :=
    function(Cause: Second): Wrapped of Second begin
      return Wrapped.Item(Cause, LabelText);
    end function;
  const Read: function(Value: Wrapped of First): string :=
    function(Value: Wrapped of First): string begin
      return case Value of when Wrapped.Item(First.Item(_), const Context): Context; end case;
    end function;
  Test.AssertEquals('first', Read(A(First.Item(1))));
  Test.AssertTrue(B(Second.Item(2)) is Wrapped.Item(Second.Item(2), 'second'));
end.
"#,
    );
}
