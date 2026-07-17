use super::{check_errors, check_ok};

#[test]
fn field_type_on_returned_record() {
    check_ok(
        "program T; \
         type Point = record X: integer; Y: integer; end; \
         function Make(): Point; begin return record X := 1; Y := 2; end end; \
         var V: integer := Make().X; \
         begin end.",
    );
}

#[test]
fn index_result_for_returned_array() {
    check_ok(
        "program T; \
         function Make(): array of integer; begin return [10, 20, 30] end; \
         var V: integer := Make()[1]; \
         begin end.",
    );
}

#[test]
fn index_result_for_returned_dict() {
    check_ok(
        "program T; \
         function Make(): dict of string to integer; begin return ['a': 1] end; \
         var V: integer := Make()['a']; \
         begin end.",
    );
}

#[test]
fn index_result_for_returned_string() {
    check_ok(
        "program T; \
         function Make(): string; begin return 'ab' end; \
         var V: string := Make()[0]; \
         begin end.",
    );
}

#[test]
fn instance_method_argument_and_return_propagation() {
    check_ok(
        "program T; \
         type Num = record \
           V: integer; \
           function Scale(Self: Num; Factor: integer): Num; \
           begin return record V := Self.V * Factor; end end; \
           function Next(Self: Num): Num; \
           begin return record V := Self.V + 1; end end; \
         end; \
         function Create(): Num; begin return record V := 2; end end; \
         var Out: integer := Create().Scale(3).Next().V; \
         begin end.",
    );
}

#[test]
fn type_alias_on_intermediate_record() {
    check_ok(
        "program T; \
         type Point = record X: integer; Y: integer; end; \
         type Alias = Point; \
         function Make(): Alias; begin return record X := 4; Y := 5; end end; \
         var V: integer := Make().Y; \
         begin end.",
    );
}

#[test]
fn unknown_field_on_postfix() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; end; \
         function Make(): Point; begin return record X := 1; end end; \
         var V: integer := Make().Missing; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("no field `Missing`")),
        "{errors:#?}"
    );
}

#[test]
fn invalid_suffix_does_not_cascade_into_later_suffixes() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; end; \
         function Make(): Point; begin return record X := 1; end end; \
         var V: integer := Make().Missing.Another; \
         begin end.",
    );
    assert_eq!(errors.len(), 1, "unexpected cascading errors: {errors:#?}");
    assert!(
        errors[0].message.contains("no field `Missing`"),
        "{errors:#?}"
    );
}

#[test]
fn unknown_method_on_postfix() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; end; \
         function Make(): Point; begin return record X := 1; end end; \
         var V: integer := Make().Missing(); \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("has no method `Missing`")),
        "{errors:#?}"
    );
}

#[test]
fn non_record_member_access() {
    let errors = check_errors(
        "program T; \
         function Make(): integer; begin return 1 end; \
         var V: integer := Make().X; \
         begin end.",
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
        "program T; \
         function Make(): array of integer; begin return [1] end; \
         var V: integer := Make()['x']; \
         begin end.",
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
        "program T; \
         type Point = record X: integer; end; \
         function Make(): Point; begin return record X := 1; end end; \
         var V: integer := Make()[0]; \
         begin end.",
    );
    assert!(
        errors.iter().any(|e| e.message.contains("not an array")),
        "{errors:#?}"
    );
}

#[test]
fn static_function_through_returned_value() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; \
           static function Create(X: integer): Point; \
           begin return record X := X; end end; \
         end; \
         function Make(): Point; begin return Point.Create(1) end; \
         var V: Point := Make().Create(2); \
         begin end.",
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
        "program T; \
         type Point = record \
           X: integer; \
           procedure Touch(Self: Point); begin end; \
         end; \
         function Make(): Point; begin return record X := 1; end end; \
         var V: integer := Make().Touch(); \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("does not return a value")),
        "{errors:#?}"
    );
}

#[test]
fn generic_instance_function_in_chain() {
    check_ok(
        "program T; \
         type Box = record \
           Value: integer; \
           function Map<T>(Self: Box; Fn: function(X: integer): T): T; \
           begin return Fn(Self.Value) end; \
         end; \
         function Create(): Box; begin return record Value := 7; end end; \
         function Identity(N: integer): integer; begin return N end; \
         var V: integer := Create().Map(Identity); \
         begin end.",
    );
}

#[test]
fn generic_instance_function_result_continues_chain() {
    check_ok(
        "program T; \
         type Value = record Number: integer; end; \
         type Box = record \
           Number: integer; \
           function Map<T>(Self: Box; Fn: function(X: integer): T): T; \
           begin return Fn(Self.Number) end; \
         end; \
         function Create(): Box; begin return record Number := 7; end end; \
         function Wrap(N: integer): Value; begin return record Number := N; end end; \
         var V: integer := Create().Map(Wrap).Number; \
         begin end.",
    );
}

#[test]
fn generic_free_function_result_continues_chain() {
    check_ok(
        "program T; \
         type Value = record Number: integer; end; \
         function Identity<T>(Input: T): T; begin return Input end; \
         function Create(): Value; begin return record Number := 9; end end; \
         var V: integer := Identity(Create()).Number; \
         begin end.",
    );
}

#[test]
fn generic_static_function_result_continues_chain() {
    check_ok(
        "program T; \
         type Value = record Number: integer; end; \
         type Factory = record \
           static function Identity<T>(Input: T): T; begin return Input end; \
         end; \
         function Create(): Value; begin return record Number := 11; end end; \
         var V: integer := Factory.Identity(Create()).Number; \
         begin end.",
    );
}
