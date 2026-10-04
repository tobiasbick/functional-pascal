use crate::analyze_with_types;

#[test]
fn closure_capturing_a_stateful_named_routine_stays_task_bound() {
    let errors = crate::tests::check_errors(
        "program Main;
        procedure Make();
          procedure Increment(); begin Count := Count + 1; end procedure;
        begin
          var Count := 0;
          const Run := procedure() begin Increment(); end procedure;
          go Run();
        end procedure;
        begin Make(); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
}

#[test]
fn nested_closure_parameter_does_not_capture_shadowed_outer_binding() {
    let (program, parse_errors) = fpas_parser::parse(
        r#"program T;
function Make(): function(): integer;
begin  var Count: integer := 10;
  return function(): integer
  begin
    const Invoke: function(Count: integer): integer :=
      function(Count: integer): integer
      begin
        return Count;
      end function;
    return Invoke(5);
  end function;
end function;
begin null; end program;"#,
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    let closures = metadata.closure_infos;
    assert_eq!(closures.len(), 2);
    assert!(
        closures.values().all(|info| info.captures.is_empty()),
        "shadowed parameter created captures: {closures:#?}"
    );
}

#[test]
fn closure_block_local_does_not_capture_shadowed_outer_binding() {
    let (program, parse_errors) = fpas_parser::parse(
        r#"program T;
function Make(): function(): integer;
begin  var Count: integer := 10;
  return function(): integer
  begin
    begin
      const Count: integer := 5;
      const Copy: integer := Count;
    end;
    return 0;
  end function;
end function;
begin null; end program;"#,
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    let closures = metadata.closure_infos;
    assert_eq!(closures.len(), 1);
    assert!(
        closures.values().all(|info| info.captures.is_empty()),
        "shadowed block local created captures: {closures:#?}"
    );
}

#[test]
fn closure_scalar_case_guard_binding_does_not_capture_shadowed_outer() {
    let (program, parse_errors) = fpas_parser::parse(
        r#"program T;

begin
   var M: integer := 0;
  const N: integer := 1;
  const F: procedure() := procedure() begin
    case N of
      when const M if M > 0:
        return;
    end case;
  end procedure;

  go F();
end program;
"#,
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    let closures = metadata.closure_infos;
    assert_eq!(closures.len(), 1);
    let info = closures.values().next().expect("closure info");
    assert!(
        info.captures
            .iter()
            .all(|capture| !capture.name.eq_ignore_ascii_case("M")),
        "scalar guard binding spuriously captured outer M: {info:#?}"
    );
    assert!(
        !info.task_bound,
        "closure should not be task-bound from a shadowed mutable: {info:#?}"
    );
}

#[test]
fn nested_closure_capturing_task_bound_callable_is_task_bound() {
    let (program, parse_errors) = fpas_parser::parse(
        r#"program T;
begin
   var Count: integer := 0;
  const Inc: procedure() :=
    procedure()
    begin
      Count := Count + 1;
    end procedure;
  const Outer: procedure() :=
    procedure()
    begin
      Inc();
    end procedure;
  go Outer();
end program;"#,
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let metadata = analyze_with_types(&program);
    assert!(
        metadata
            .errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE }),
        "expected task-bound spawn error, got: {:#?}",
        metadata.errors
    );
    let outer = metadata
        .closure_infos
        .values()
        .find(|info| {
            info.captures
                .iter()
                .any(|capture| capture.name.eq_ignore_ascii_case("Inc"))
        })
        .expect("outer closure should capture Inc");
    assert!(
        outer.task_bound,
        "outer closure must be task-bound via nested capture: {outer:#?}"
    );
}
