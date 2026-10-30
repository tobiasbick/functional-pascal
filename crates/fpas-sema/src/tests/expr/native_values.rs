//! Native callable references preserve operation validation and writable modes.

use super::{check_errors, check_ok};

#[test]
fn native_callable_values_require_sufficient_context_and_preserve_argument_types() {
    for body in [
        "const F := Math.Abs;",
        "const F: function(N: string): string := Math.Abs;",
        "const F: function(N: integer): real := Math.Abs;",
        "const F: function(A: integer; B: real): integer := Math.Min;",
        "const F: function(var N: integer): integer := Math.Abs;",
        "const F: function(N: integer; Extra: integer): integer := Math.Abs;",
        "const F := Console.WriteLn;",
        "const F := Some;",
        "const F := Ok;",
        "const F := Error;",
        "const F := string.Length;",
        "const F: procedure(N: integer) := Some;",
        "const F: function(var N: integer): option of integer := Some;",
        "const F: function(N: integer): option of string := Some;",
        "const F: function(Value: integer; Count: integer): array of string := array.Fill;",
        "const F: procedure(var N: integer) := Console.WriteLn;",
        "var Text := 'a'; const F := Text.TrimInPlace;",
        "const A := [Some(1)]; const F := A.Sort;",
    ] {
        let source =
            format!("program T; uses Std.Math as Math, Std.Console as Console; begin {body} end.");
        let errors = check_errors(&source);
        assert!(!errors.is_empty(), "unexpectedly accepted: {source}");
    }
}

#[test]
fn native_callable_values_do_not_capture_type_names_as_receivers() {
    for declaration in [
        "type Text = string;",
        "type Text = error Failed; end error;",
    ] {
        let member = if declaration.contains("error") {
            "Message"
        } else {
            "Trim"
        };
        check_errors(&format!(
            "program T; {declaration} begin const F := Text.{member}; end."
        ));
    }
}

#[test]
fn native_callable_values_do_not_erase_distinct_types_in_variadic_adapters() {
    for operation in ["Console.WriteLn", "'%d'.Format"] {
        let signature = if operation.starts_with("Console") {
            "procedure(Value: Id)"
        } else {
            "function(Value: Id): string"
        };
        check_errors(&format!(
            "program T; uses Std.Console as Console;
             type Id = distinct integer;
             begin const F: {signature} := {operation}; end."
        ));
    }
}

#[test]
fn native_callable_values_keep_procedure_callbacks_and_nested_receivers() {
    check_ok(
        r#"
program T;
uses Std.Test as Test;
begin
  const Values := [1, 2];
  const Visit: procedure(F: procedure(X: integer)) := Values.ForEach;
  Visit(procedure(X: integer) begin Test.AssertTrue(X > 0); end procedure);
  const Texts := [' a '];
  const Trim := Texts[0].Trim;
  Test.AssertEquals('a', Trim());
end.
"#,
    );
}

#[test]
fn native_callable_values_preserve_task_transfer_and_scope_handle_boundaries() {
    for source in [
        r#"program T;
          uses Std.Tasks as Tasks;
          begin scope
            var Counter := 0;
            const Local := function(): integer begin
              Counter := Counter + 1; return Counter;
            end function;
            const Values := [Local];
            const Length := Values.Length;
            discard go Length();
          end scope; end."#,
        r#"program T;
          uses Std.Tasks as Tasks;
          function Work(): integer; begin return 1; end function;
          function Leak(): function(): task of integer;
          begin scope
            const Job := go Work();
            const Maybe: option of task of integer := Some(Job);
            return Maybe.Unwrap;
          end scope; end function;
          begin end."#,
    ] {
        let errors = check_errors(source);
        assert!(
            errors.iter().any(|error| matches!(
                error.code,
                fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE
                    | fpas_diagnostics::codes::SEMA_TASK_HANDLE_ESCAPE
            )),
            "{errors:#?}"
        );
    }
}

#[test]
fn native_callable_values_validate_every_readonly_catalog_entry() {
    use crate::{NativeLowering, NativeReceiver, Ty, types::substitute_type_parameters};
    use std::collections::HashMap;
    let int = Ty::Integer;
    let substitutions = HashMap::from([
        ("t".into(), int.clone()),
        ("u".into(), int.clone()),
        ("k".into(), int.clone()),
        ("v".into(), int.clone()),
        ("v2".into(), int.clone()),
        ("e".into(), Ty::String),
        ("e2".into(), Ty::String),
    ]);
    for operation in crate::native_operations() {
        if operation.receiver.is_factory()
            || matches!(
                operation.lowering,
                NativeLowering::InPlace(_) | NativeLowering::ArrayPop
            )
        {
            continue;
        }
        let (receiver, expression) = match operation.receiver {
            NativeReceiver::ByteBuffer => (Ty::ByteBuffer, "Bytes.Create(2).Unwrap()"),
            NativeReceiver::Error => (Ty::Named("Failure".into()), "Failure.Failed"),
            NativeReceiver::String => (Ty::String, "'ab'"),
            NativeReceiver::Array => (Ty::Array(Box::new(int.clone())), "[1, 2]"),
            NativeReceiver::StringArray => (Ty::Array(Box::new(Ty::String)), "['a', 'b']"),
            NativeReceiver::Dictionary => (
                Ty::Dict(Box::new(Ty::String), Box::new(int.clone())),
                "['a': 1]",
            ),
            NativeReceiver::Option => (Ty::Option(Box::new(int.clone())), "Some(1)"),
            NativeReceiver::Result => (
                Ty::Result(Box::new(int.clone()), Box::new(Ty::String)),
                "Ok(1)",
            ),
            _ => unreachable!(),
        };
        let mut signature = operation.signature_for(Some(&receiver));
        if signature.variadic {
            signature.variadic = false;
            signature
                .params
                .push(crate::ParamTy::value("N", int.clone()));
        }
        let ty = if *signature.return_type == Ty::Unit {
            Ty::Procedure(crate::types::ProcedureTy {
                type_params: vec![],
                params: signature.params,
                variadic: false,
            })
        } else {
            Ty::Function(signature)
        };
        let ty = substitute_type_parameters(&ty, &substitutions);
        let source = format!(
            "program T; uses Std.Bytes as Bytes; type Failure = error Failed; end error;
             begin const Receiver: {receiver} := {expression};
             const F: {ty} := Receiver.{}; end.",
            operation.name
        );
        check_ok(&source.replace("Std.Bytes.", "Bytes."));
    }
}
