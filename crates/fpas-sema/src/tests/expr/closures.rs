use crate::analyze_with_types;

#[test]
fn nested_closure_parameter_does_not_capture_shadowed_outer_binding() {
    let (program, parse_errors) = fpas_parser::parse(
        "program T;
function Make(): function(): integer;
begin
  mutable var Count: integer := 10;
  return function(): integer
  begin
    var Read: function(Count: integer): integer :=
      function(Count: integer): integer
      begin
        return Count
      end;
    return Read(5)
  end
end;
begin
end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let (errors, _, _, _, _, closures, _, _, _, _, _, _, _) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(closures.len(), 2);
    assert!(
        closures.values().all(|info| info.captures.is_empty()),
        "shadowed parameter created captures: {closures:#?}"
    );
}

#[test]
fn closure_block_local_does_not_capture_shadowed_outer_binding() {
    let (program, parse_errors) = fpas_parser::parse(
        "program T;
function Make(): function(): integer;
begin
  mutable var Count: integer := 10;
  return function(): integer
  begin
    begin
      var Count: integer := 5;
      var Copy: integer := Count
    end;
    return 0
  end
end;
begin
end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let (errors, _, _, _, _, closures, _, _, _, _, _, _, _) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(closures.len(), 1);
    assert!(
        closures.values().all(|info| info.captures.is_empty()),
        "shadowed block local created captures: {closures:#?}"
    );
}
