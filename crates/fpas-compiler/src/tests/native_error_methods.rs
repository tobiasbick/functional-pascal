//! Record-owner substitutions through error-producing methods and cached interfaces.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::{assert_succeeds, compiled_unit, parse_ok};
use fpas_unit::object::{decode_object, encode_object};

#[test]
fn native_error_methods_retain_record_and_method_arguments() {
    assert_succeeds(
        r#"
program ErrorMethods;
type First = error Failed; end error;
type Second = error Failed; end error;
type Context of E: Error = error Wrapped(Cause: E); end error;
type PairContext of (E: Error, F: Error) = error Wrapped(Cause: E; Other: F); end error;
type Holder of E: Error = record
  Value: E;
  function Wrap(Self: Holder of E): Context of E;
  begin return Context.Wrapped(Self.Value); end function;
  function Combine<F: Error>(Self: Holder of E; Other: F): PairContext of (E, F);
  begin return PairContext.Wrapped(Self.Value, Other); end function;
end record;
function Bind<E: Error>(Value: E): function(): Context of E;
begin
  const Item: Holder of E := Holder(Value := Value);
  return Item.Wrap;
end function;
function MakeHolder(): Holder of First;
begin return Holder(Value := First.Failed); end function;
begin
  const Item: Holder of First := Holder(Value := First.Failed);
  const Direct := Item.Wrap();
  const Bound: function(): Context of First := Item.Wrap;
  const Pair: function(Other: Second): PairContext of (First, Second) := Item.Combine;
  const Deferred := Bind(Second.Failed);
  if not (Direct is Context.Wrapped(First.Failed)) then panic('direct record argument'); end if;
  if not (Bound() is Context.Wrapped(First.Failed)) then panic('bound record argument'); end if;
  if not (Item.Combine(Second.Failed) is PairContext.Wrapped(First.Failed, Second.Failed)) then panic('direct method argument'); end if;
  if not (Pair(Second.Failed) is PairContext.Wrapped(First.Failed, Second.Failed)) then panic('bound method argument'); end if;
  if not (Deferred() is Context.Wrapped(Second.Failed)) then panic('generic bound receiver'); end if;
  scope
    discard go MakeHolder().Wrap();
  end scope;
end.
"#,
    );
}

#[test]
fn native_error_methods_keep_constructor_requirements_in_cached_interfaces() {
    let unit = compiled_unit(
        r#"
unit Demo.MethodErrors;
public type First = error Failed; end error;
public type Second = error Failed; end error;
public type Context of E: Error = error Wrapped(Cause: E); end error;
public type PairContext of (E: Error, F: Error) = error Wrapped(Cause: E; Other: F); end error;
public type Holder of E: Error = record
  public Value: E;
  public function Wrap(Self: Holder of E): Context of E;
  begin return Context.Wrapped(Self.Value); end function;
  public function Combine<F: Error>(Self: Holder of E; Other: F): PairContext of (E, F);
  begin return PairContext.Wrapped(Self.Value, Other); end function;
end record;
end unit;
"#,
        &[],
    );
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            r#"
program CachedErrorMethods;
uses Demo.MethodErrors as Domain;
type Alias = Domain.First;
type HolderAlias = Domain.Holder of Alias;
function Bind<E: Error>(Value: E): function(): Domain.Context of E;
begin
  const Item: Domain.Holder of E := Domain.Holder(Value := Value);
  return Item.Wrap;
end function;
begin
  const Item: HolderAlias := HolderAlias(Value := Alias.Failed);
  const Bound: function(): Domain.Context of Alias := Item.Wrap;
  const Pair: function(Other: Domain.Second): Domain.PairContext of (Alias, Domain.Second) := Item.Combine;
  const Deferred := Bind(Domain.Second.Failed);
  if not (Item.Wrap() is Domain.Context.Wrapped(Domain.First.Failed)) then panic('imported direct'); end if;
  if not (Bound() is Domain.Context.Wrapped(Domain.First.Failed)) then panic('imported bound'); end if;
  if not (Item.Combine(Domain.Second.Failed) is Domain.PairContext.Wrapped(Domain.First.Failed, Domain.Second.Failed)) then panic('imported method argument'); end if;
  if not (Pair(Domain.Second.Failed) is Domain.PairContext.Wrapped(Domain.First.Failed, Domain.Second.Failed)) then panic('imported bound method argument'); end if;
  if not (Deferred() is Domain.Context.Wrapped(Domain.Second.Failed)) then panic('imported generic bound'); end if;
end.
"#,
        ),
        std::slice::from_ref(&unit.interface),
        std::slice::from_ref(&unit.interface),
    )
    .expect("source-less method consumer");
    let object = decode_object(&encode_object(&unit.object).expect("encode method unit"))
        .expect("decode method unit");
    let executable = fpas_linker::link_objects(&[object], &root).expect("cached methods link");
    fpas_vm::Vm::new(executable)
        .run()
        .expect("cached error-producing methods execute");
}
