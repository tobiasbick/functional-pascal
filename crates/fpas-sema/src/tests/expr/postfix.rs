use super::{check_errors, check_ok};

#[test]
fn field_type_on_returned_record() {
    check_ok(
        r#"program T;  type Point = record X: integer; Y: integer; end record; function Make(): Point; begin return record X := 1; Y := 2; end record; end function;  var V: integer := Make().X; begin null; end program;"#,
    );
}

#[test]
fn index_result_for_returned_array() {
    check_ok(
        r#"program T; function Make(): array of integer; begin return [10, 20, 30]; end function;  var V: integer := Make()[1]; begin null; end program;"#,
    );
}

#[test]
fn index_result_for_returned_dict() {
    check_ok(
        r#"program T; function Make(): dict of string to integer; begin return ['a': 1]; end function;  var V: integer := Make()['a']; begin null; end program;"#,
    );
}

#[test]
fn index_result_for_returned_string() {
    check_ok(
        r#"program T; function Make(): string; begin return 'ab'; end function;  var V: string := Make()[0]; begin null; end program;"#,
    );
}

#[test]
fn instance_method_argument_and_return_propagation() {
    check_ok(
        r#"program T;  type Num = record V: integer; function Scale(Self: Num; Factor: integer): Num; begin return record V := Self.V * Factor; end record; end function; function Next(Self: Num): Num; begin return record V := Self.V + 1; end record; end function; end record; function Create(): Num; begin return record V := 2; end record; end function;  var Out: integer := Create().Scale(3).Next().V; begin null; end program;"#,
    );
}

#[test]
fn type_alias_on_intermediate_record() {
    check_ok(
        r#"program T;  type Point = record X: integer; Y: integer; end record;  type Alias = Point; function Make(): Alias; begin return record X := 4; Y := 5; end record; end function;  var V: integer := Make().Y; begin null; end program;"#,
    );
}

#[test]
fn unknown_field_on_postfix() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; end record; function Make(): Point; begin return record X := 1; end record; end function;  var V: integer := Make().Missing; begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("no field, property, event, or method `Missing`")),
        "{errors:#?}"
    );
}

#[test]
fn invalid_suffix_does_not_cascade_into_later_suffixes() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; end record; function Make(): Point; begin return record X := 1; end record; end function;  var V: integer := Make().Missing.Another; begin null; end program;"#,
    );
    assert_eq!(errors.len(), 1, "unexpected cascading errors: {errors:#?}");
    assert!(
        errors[0]
            .message
            .contains("no field, property, event, or method `Missing`"),
        "{errors:#?}"
    );
}

#[test]
fn unknown_method_on_postfix() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; end record; function Make(): Point; begin return record X := 1; end record; end function;  var V: integer := Make().Missing(); begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("No lexical `Missing` accepts receiver type `Point`")),
        "{errors:#?}"
    );
}

#[test]
fn non_record_member_access() {
    let errors = check_errors(
        r#"program T; function Make(): integer; begin return 1; end function;  var V: integer := Make().X; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("requires a record value")),
        "{errors:#?}"
    );
}

#[test]
fn wrong_index_type_on_returned_array() {
    let errors = check_errors(
        r#"program T; function Make(): array of integer; begin return [1]; end function;  var V: integer := Make()['x']; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Array index must be integer")),
        "{errors:#?}"
    );
}

#[test]
fn non_indexable_receiver() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; end record; function Make(): Point; begin return record X := 1; end record; end function;  var V: integer := Make()[0]; begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|e| e.message.contains("not an array")),
        "{errors:#?}"
    );
}

#[test]
fn static_function_through_returned_value() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; static function Create(X: integer): Point; begin return record X := X; end record; end function; end record; function Make(): Point; begin return Point.Create(1); end function;  var V: Point := Make().Create(2); begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("is a static function")),
        "{errors:#?}"
    );
}

#[test]
fn procedure_method_in_expression() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; procedure Touch(Self: Point); begin null; end procedure; end record; function Make(): Point; begin return record X := 1; end record; end function;  var V: integer := Make().Touch(); begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("does not return a value")),
        "{errors:#?}"
    );
}

#[test]
fn procedure_method_may_finish_postfix_statement() {
    check_ok(
        r#"program T;  type Point = record X: integer; procedure Touch(Self: Point); begin null; end procedure; static function Create(): Point; begin return record X := 1; end record; end function; end record; begin Point.Create().Touch(); end program;"#,
    );
}

#[test]
fn procedure_method_may_not_continue_postfix_statement() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; procedure Touch(Self: Point); begin null; end procedure; function Next(Self: Point): Point; begin return Self; end function; static function Create(): Point; begin return record X := 1; end record; end function; end record; begin Point.Create().Touch().Next(); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("does not return a value")),
        "{errors:#?}"
    );
}

#[test]
fn postfix_value_may_not_be_used_as_statement() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; static function Create(): Point; begin return record X := 1; end record; end function; end record; begin Point.Create().X; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("must end with a method call")),
        "{errors:#?}"
    );
}

#[test]
fn generic_instance_function_in_chain() {
    check_ok(
        r#"program T;  type Box = record Value: integer; function Map<T>(Self: Box; Fn: function(X: integer): T): T; begin return Fn(Self.Value); end function; end record; function Create(): Box; begin return record Value := 7; end record; end function; function Identity(N: integer): integer; begin return N; end function;  var V: integer := Create().Map(Identity); begin null; end program;"#,
    );
}

#[test]
fn generic_instance_function_result_continues_chain() {
    check_ok(
        r#"program T;  type Value = record Number: integer; end record;  type Box = record Number: integer; function Map<T>(Self: Box; Fn: function(X: integer): T): T; begin return Fn(Self.Number); end function; end record; function Create(): Box; begin return record Number := 7; end record; end function; function Wrap(N: integer): Value; begin return record Number := N; end record; end function;  var V: integer := Create().Map(Wrap).Number; begin null; end program;"#,
    );
}

#[test]
fn generic_free_function_result_continues_chain() {
    check_ok(
        r#"program T;  type Value = record Number: integer; end record; function Identity<T>(Input: T): T; begin return Input; end function; function Create(): Value; begin return record Number := 9; end record; end function;  var V: integer := Identity(Create()).Number; begin null; end program;"#,
    );
}

#[test]
fn generic_static_function_result_continues_chain() {
    check_ok(
        r#"program T;  type Value = record Number: integer; end record;  type Factory = record static function Identity<T>(Input: T): T; begin return Input; end function; end record; function Create(): Value; begin return record Number := 11; end record; end function;  var V: integer := Factory.Identity(Create()).Number; begin null; end program;"#,
    );
}
