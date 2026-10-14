//! Mandatory qualifiers across imports, values, construction and patterns.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME;

#[test]
fn native_call_repairs_preserve_the_written_alias() {
    for (call, repair) in [
        ("Numbers.Abs()", "Numbers.Abs(-5)"),
        ("Numbers.Min(1)", "Numbers.Min(A, B)"),
        ("Numbers.Sign()", "Numbers.Sign(-5)"),
        ("Numbers.Clamp(1)", "Numbers.Clamp(X, Lo, Hi)"),
    ] {
        let errors = check_errors(&format!(
            "program P; uses Std.Math as Numbers; begin discard {call}; end."
        ));
        assert!(
            errors.iter().any(|error| error
                .help
                .as_ref()
                .is_some_and(|hint| hint.contains(repair) && !hint.contains("Std.Math."))),
            "{errors:?}"
        );
    }
    let errors = check_errors(
        "program P; uses Std.Tasks as Jobs; procedure Work(); begin end procedure; begin scope const H: task := go Work(); const N: integer := Jobs.Wait(H); end scope; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.help.as_ref().is_some_and(|hint| hint
                .contains("Jobs.Wait(Handle);")
                && !hint.contains("`Wait(Handle);`"))),
        "{errors:?}"
    );
}

#[test]
fn imported_names_and_enum_members_require_their_source_qualifier() {
    for (source, repair) in [
        (
            "program P; uses Std.Console as Console; begin WriteLn('x'); end.",
            "Console.WriteLn",
        ),
        (
            "program P; uses Std.Tasks as Tasks; begin const Group: TaskGroup := Tasks.CreateTaskGroup(); end.",
            "Tasks.TaskGroup",
        ),
        (
            "program P; type Color = enum Red; Blue; end enum; begin const C: Color := Red; end.",
            "Color.Red",
        ),
        (
            "program P; type Data = enum Item(Value: integer); Empty; end enum; begin const D: Data := Item(1); end.",
            "Data.Item",
        ),
        (
            "program P; type Color = enum Red; Blue; end enum; begin const C := Color.Red; case C of when Red: null; when Color.Blue: null; end case; end.",
            "Color.Red",
        ),
        (
            "program P; type Data = enum Item(Value: integer); Empty; end enum; begin const D := Data.Item(1); discard D is Item(_); end.",
            "Data.Item",
        ),
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_UNQUALIFIED_NAME
                    && error
                        .help
                        .as_ref()
                        .is_some_and(|help| help.contains(repair))),
            "{source}: {errors:#?}"
        );
    }
}

#[test]
fn qualified_variants_remain_stable_across_collisions_aliases_and_generic_patterns() {
    check_ok(
        "program P; uses Std.Console as Console;
        type Color = enum Red; Blue; end enum;
        type Other = enum Red; end enum;
        type Red = integer;
        type LocalColor = Color;
        type Payload of T = enum Item(Value: T); Empty; end enum;
        procedure WriteLn(Value: integer); begin end procedure;
        begin const C := LocalColor.Red;
        const P: Payload of integer := Payload.Item(Value := 42);
        WriteLn(1); Console.WriteLn('qualified');
        case Some(P) of
          when Some(Payload.Item(Value := const Number)): discard Number;
          when Some(Payload.Empty), None: null;
        end case;
        discard C = Color.Red;
        discard P is Payload.Item(Value := _);
        end.",
    );
}
