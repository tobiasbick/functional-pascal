//! Import-free checking of every catalog signature and native argument diagnostics.
use super::{check_errors, check_ok};

#[test]
fn every_catalog_entry_accepts_positional_and_fully_named_explicit_arguments() {
    let cases = [
        (
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Length(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Length(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.IsEmpty(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.IsEmpty(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.Contains('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.Contains(Sub := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.StartsWith('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.StartsWith(Pre := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.EndsWith('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.EndsWith(Suf := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.IndexOf('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.IndexOf(Sub := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Trim(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Trim(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.ToUpper(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.ToUpper(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.ToLower(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.ToLower(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Replace('a', 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Replace(New := 'a', Old := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: array of string := Receiver.Split('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: array of string := Receiver.Split(Delim := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Map(function(C: string): string begin return 'a'; end function); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Map(F := function(C: string): string begin return 'a'; end function); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Filter(function(C: string): boolean begin return true; end function); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Filter(F := function(C: string): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Reduce(1, function(Acc: integer; C: string): integer begin return 1; end function); end.",
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Reduce(F := function(Acc: integer; C: string): integer begin return 1; end function, Init := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Slice(1, 1); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Slice(Len := 1, Start := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.LastIndexOf('a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.LastIndexOf(Sub := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.IsNumeric(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: boolean := Receiver.IsNumeric(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.RepeatStr(1); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.RepeatStr(N := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadLeft(1, 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadLeft(PadChar := 'a', Width := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadRight(1, 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadRight(PadChar := 'a', Width := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadCenter(1, 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.PadCenter(PadChar := 'a', Width := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.FromChar(1); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.FromChar(N := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.CharAt(1); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.CharAt(Index := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.SetCharAt(1, 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.SetCharAt(C := 'a', Index := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Ord(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: integer := Receiver.Ord(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Insert(1, 'a'); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Insert(Sub := 'a', Index := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Delete(1, 1); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Delete(Len := 1, Index := 1); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Reverse(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Reverse(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.TrimLeft(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.TrimLeft(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.TrimRight(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.TrimRight(); end.",
        ),
        (
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Format(); end.",
            "program T; begin var Receiver: string := 'a'; const Output: string := Receiver.Format(); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Length(); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Length(); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.IsEmpty(); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.IsEmpty(); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.Contains(1); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.Contains(Value := 1); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.IndexOf(1); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.IndexOf(Value := 1); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Map(function(X: integer): integer begin return 1; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Map(F := function(X: integer): integer begin return 1; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Filter(function(X: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Filter(F := function(X: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Reduce(1, function(Acc: integer; integer: integer): integer begin return 1; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Reduce(F := function(Acc: integer; integer: integer): integer begin return 1; end function, Init := 1); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: Option of integer := Receiver.Find(function(X: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: Option of integer := Receiver.Find(F := function(X: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.FindIndex(function(X: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.FindIndex(F := function(X: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.Any(function(X: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.Any(F := function(X: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.All(function(X: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: boolean := Receiver.All(F := function(X: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Sort(); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Sort(); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Reverse(); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Reverse(); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Slice(1, 1); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Slice(Len := 1, Start := 1); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Concat([1]); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.Concat(B := [1]); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.FlatMap(function(X: integer): array of integer begin return [1]; end function); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: array of integer := Receiver.FlatMap(F := function(X: integer): array of integer begin return [1]; end function); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; Receiver.ForEach(procedure(X: integer) begin end procedure); end.",
            "program T; begin var Receiver: array of integer := [1]; Receiver.ForEach(F := procedure(X: integer) begin end procedure); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; Receiver.Push(1); end.",
            "program T; begin var Receiver: array of integer := [1]; Receiver.Push(Value := 1); end.",
        ),
        (
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Pop(); end.",
            "program T; begin var Receiver: array of integer := [1]; const Output: integer := Receiver.Pop(); end.",
        ),
        (
            "program T; begin var Receiver: array of string := ['a']; const Output: string := Receiver.Join('a'); end.",
            "program T; begin var Receiver: array of string := ['a']; const Output: string := Receiver.Join(Delim := 'a'); end.",
        ),
        (
            "program T; begin const Output: string := string.Chr(1); end.",
            "program T; begin const Output: string := string.Chr(N := 1); end.",
        ),
        (
            "program T; begin const Output: array of integer := array.Fill(1, 1); end.",
            "program T; begin const Output: array of integer := array.Fill(Count := 1, Value := 1); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: integer := Receiver.Length(); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: integer := Receiver.Length(); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: boolean := Receiver.IsEmpty(); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: boolean := Receiver.IsEmpty(); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: boolean := Receiver.ContainsKey('a'); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: boolean := Receiver.ContainsKey(Key := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: Option of integer := Receiver.Get('a'); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: Option of integer := Receiver.Get(Key := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: array of string := Receiver.Keys(); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: array of string := Receiver.Keys(); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: array of integer := Receiver.Values(); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: array of integer := Receiver.Values(); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to string := Receiver.Map(function(integer: integer): string begin return 'a'; end function); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to string := Receiver.Map(F := function(integer: integer): string begin return 'a'; end function); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Filter(function(string: string; integer: integer): boolean begin return true; end function); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Filter(F := function(string: string; integer: integer): boolean begin return true; end function); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: integer := Receiver.Reduce(1, function(Acc: integer; Key: string; Value: integer): integer begin return 1; end function); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: integer := Receiver.Reduce(F := function(Acc: integer; Key: string; Value: integer): integer begin return 1; end function, Init := 1); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Merge(['a': 1]); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Merge(D2 := ['a': 1]); end.",
        ),
        (
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Remove('a'); end.",
            "program T; begin var Receiver: dict of string to integer := ['a': 1]; const Output: dict of string to integer := Receiver.Remove(Key := 'a'); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: boolean := Receiver.IsSome(); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: boolean := Receiver.IsSome(); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: boolean := Receiver.IsNone(); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: boolean := Receiver.IsNone(); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.Map(function(integer: integer): integer begin return 1; end function); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.Map(F := function(integer: integer): integer begin return 1; end function); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.AndThen(function(integer: integer): Option of integer begin return Some(1); end function); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.AndThen(F := function(integer: integer): Option of integer begin return Some(1); end function); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.OrElse(function(): Option of integer begin return Some(1); end function); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: Option of integer := Receiver.OrElse(F := function(): Option of integer begin return Some(1); end function); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: integer := Receiver.Unwrap(); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: integer := Receiver.Unwrap(); end.",
        ),
        (
            "program T; begin var Receiver: Option of integer := Some(1); const Output: integer := Receiver.UnwrapOr(1); end.",
            "program T; begin var Receiver: Option of integer := Some(1); const Output: integer := Receiver.UnwrapOr(Default := 1); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: boolean := Receiver.IsOk(); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: boolean := Receiver.IsOk(); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: boolean := Receiver.IsError(); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: boolean := Receiver.IsError(); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, string := Receiver.Map(function(integer: integer): integer begin return 1; end function); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, string := Receiver.Map(F := function(integer: integer): integer begin return 1; end function); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, string := Receiver.AndThen(function(integer: integer): Result of integer, string begin return Ok(1); end function); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, string := Receiver.AndThen(F := function(integer: integer): Result of integer, string begin return Ok(1); end function); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, integer := Receiver.OrElse(function(Err: string): Result of integer, integer begin return Ok(1); end function); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: Result of integer, integer := Receiver.OrElse(F := function(Err: string): Result of integer, integer begin return Ok(1); end function); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: integer := Receiver.Unwrap(); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: integer := Receiver.Unwrap(); end.",
        ),
        (
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: integer := Receiver.UnwrapOr(1); end.",
            "program T; begin var Receiver: Result of integer, string := Ok(1); const Output: integer := Receiver.UnwrapOr(Default := 1); end.",
        ),
    ];
    assert_eq!(cases.len(), crate::native_operations().count());
    for (positional, named) in cases {
        check_ok(positional);
        check_ok(named);
    }
}
#[test]
fn native_arguments_reject_names_counts_types_and_receiver_markers_without_fallback() {
    for call in [
        "Text.Slice(Start := 0)",
        "Text.Slice(Start := 0, START := 1)",
        "Text.Slice(S := Text, Start := 0, Len := 1)",
        "Text.Slice(Width := 0, Len := 1)",
        "Text.Slice('bad', 1)",
        "Text.Slice()",
        "Text.Format(Value := 1)",
        "Text.Substring(0, 1)",
        "Text.Size()",
        "Text.Count()",
    ] {
        let source = format!(
            "program T; function Slice(A: string; B: string): string; begin return A; end function; begin const Text: string := 'abc'; discard {call}; end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code
                == fpas_diagnostics::codes::SEMA_INVALID_NAMED_ARGUMENT
                || error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                || error.code == fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT
                || error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME
                || error.code == fpas_diagnostics::codes::SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED),
            "{call}: {errors:#?}"
        );
        if call.contains("Substring") {
            assert!(
                errors[0]
                    .help
                    .as_deref()
                    .is_some_and(|hint| hint.contains("Slice"))
            );
        }
    }
    let (_, diagnostics) =
        fpas_parser::parse("program T; begin const S: string := 'abc'.Slice(0, Len := 1); end.");
    assert!(
        diagnostics.iter().any(|error| error.as_diagnostic().code
            == fpas_diagnostics::codes::PARSE_MIXED_CALL_ARGUMENTS)
    );
}

#[test]
fn known_generic_container_shapes_keep_native_operations() {
    check_ok(
        "program T; function Count<T>(A: array of T): integer; begin return A.Length(); end function; function Empty<K: Comparable, V>(D: dict of K to V): boolean; begin return D.IsEmpty(); end function; function Has<T>(O: Option of T): boolean; begin return O.IsSome(); end function; function Good<T, E>(R: Result of T, E): boolean; begin return R.IsOk(); end function; begin const N: integer := Count([1]); end.",
    );
    let errors = check_errors(
        "program T; function Count<T>(A: T): integer; begin return A.Length(); end function; begin end.",
    );
    assert!(
        errors[0].message.contains("has no dot operation"),
        "{errors:#?}"
    );
}

#[test]
fn factories_validate_inferred_elements_annotations_and_exclusive_owners() {
    check_ok(
        "program T; begin const A: array of integer := array.Fill(Count := 0, Value := 7); const B: array of string := array.Fill('x', 0); const C: string := string.Chr(n := 65); end.",
    );
    for expression in [
        "array.Fill('x', 0)",
        "array.Fill(Value := 1)",
        "array.Fill(Value := 1, Value := 2)",
        "array.Fill(A := 1, Count := 2)",
        "array.Chr(65)",
        "string.Fill(1, 2)",
        "(65).Chr()",
        "[1].Fill(1, 2)",
    ] {
        let errors = check_errors(&format!(
            "program T; begin const A: array of integer := {expression}; end."
        ));
        assert!(!errors.is_empty(), "{expression}");
    }
    for expression in ["array.Fill<integer>(1, 2)", "(array of integer).Fill(1, 2)"] {
        assert!(
            !fpas_parser::parse_expression(expression).1.is_empty(),
            "{expression}"
        );
    }
}

#[test]
fn former_units_calls_and_references_are_removed_but_own_names_remain_ordinary() {
    for call in [
        "Length('x')",
        "Std.Str.Length('x')",
        "Std.Arrays.Length([1])",
        "Std.Dictionaries.Get(['a': 1], 'a')",
        "Std.Options.Unwrap(Some(1))",
        "Std.Results.Unwrap(Ok(1))",
    ] {
        let errors = check_errors(&format!("program T; begin discard {call}; end."));
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("without imports")),
            "{call}: {errors:#?}"
        );
    }
    for reference in ["Length", "Std.Str.Length"] {
        let errors = check_errors(&format!(
            "program T; begin const F: function(S: string): integer := {reference}; end."
        ));
        assert!(!errors.is_empty(), "{reference}");
    }
    check_ok(
        "program T; function Map(S: string): integer; begin return 9; end function; begin const N: integer := Map('x'); const L: integer := 'x'.Length(); end.",
    );
}
