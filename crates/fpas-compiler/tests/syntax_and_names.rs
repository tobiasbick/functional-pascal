//! Execution regressions for named block bodies and alias-only imports.

fn run(source: &str) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let executable =
        fpas_compiler::compile(&program).unwrap_or_else(|errors| panic!("{errors:#?}"));
    fpas_vm::Vm::new(executable).run().unwrap();
}

#[test]
fn branch_lists_and_elsif_evaluate_conditions_once_in_order() {
    run("program P;
        mutable var Calls: integer := 0;
        function Check(Expected: integer): boolean;
        begin Calls := Calls + 1; return Calls = Expected; end function;
        begin
          mutable var Score: integer := 0;
          if Check(9) then panic('first'); elsif Check(2) then
            var X: integer := 3; Score := X;
          elsif Check(3) then panic('third'); else panic('else'); end if;
          if Calls <> 2 then panic('condition count'); end if;
          if Score <> 3 then panic('branch body'); end if;
          var X: string := 'outer';
        end program;");
}

#[test]
fn aliased_intrinsic_tasks_keep_result_shapes_and_argument_overloads() {
    run(
        "program P;\n\nuses Std.Fs as Files;\nuses Std.Math as Numbers;\nuses Std.Tasks as Tasks;\nuses Std.Str as Text;\n\nbegin\n  var ReadJob: task := go Files.ReadText('.temp-data/missing-alias-task-file');\n  case Tasks.Wait(ReadJob) of\n    when Result.Ok(const Value):\n      panic('unexpected file');\n    when Result.Error(const Message):\n      null;\n  end case;\n  var IntJob: task := go Numbers.Abs(-2);\n  var RealJob: task := go Numbers.Abs(-2.5);\n  if Tasks.Wait(IntJob) <> 2 then\n    panic('integer task');\n  end if;\n\n  if Tasks.Wait(RealJob) <> 2.5 then\n    panic('real task');\n  end if;\n  var Formatted: task := go Text.Format('%s-%d', 'a', 2);\n  if Tasks.Wait(Formatted) <> 'a-2' then\n    panic('format task');\n  end if;\nend program;\n",
    );
}

#[test]
fn returning_elsif_chains_and_nested_else_if_execute() {
    run("program P;
        function Pick(I: integer): integer;
        begin if I = 0 then return 10; elsif I = 1 then return 20; else
          if I = 2 then return 30; else return 40; end if;
        end if; end function;
        begin
          for I: integer := 0 to 3 do
            if Pick(I) <> (I + 1) * 10 then panic('return branch'); end if;
          end for;
        end program;");
}

#[test]
fn all_loop_and_case_bodies_execute_multiple_statements() {
    run("program P; begin
        mutable var Sum: integer := 0;
        for I: integer := 1 to 2 do Sum := Sum + I; null; end for;
        for I: integer in [3, 4] do Sum := Sum + I; null; end for;
        while Sum < 12 do Sum := Sum + 1; null; end while;
        repeat Sum := Sum - 1; null; until Sum = 10;
        case Sum of when 9: panic('wrong'); when 10: Sum := Sum + 1; null; else panic('wrong else'); end case;
        if Sum <> 11 then panic('loops'); end if;
        begin var Sum: integer := 99; if Sum <> 99 then panic('block'); end if; end;
        if Sum <> 11 then panic('scope'); end if;
        end program;");
}

#[test]
fn aliases_dispatch_intrinsics_and_recursive_json_variants() {
    run(
        "program P;\n\nuses Std.Str as Text;\nuses Std.Math as Numbers;\nuses Std.Json as Json;\n\nbegin\n  if Text.Trim(' a ') <> 'a' then\n    panic('text alias');\n  end if;\n\n  if Numbers.Abs(-5) <> 5 then\n    panic('number alias');\n  end if;\n  var V: Json.JsonValue := Json.JsonValue.NullValue;\n  case V of\n    when Json.JsonValue.NullValue:\n      null;\n    when Json.JsonValue.Bool(_), Json.JsonValue.Number(_), Json.JsonValue.String(_), Json.JsonValue.ArrayValue(_), Json.JsonValue.Object(_):\n      begin\n        panic('json variant');\n      end;\n  end case;\nend program;\n",
    );
}

#[test]
fn case_arm_locals_do_not_replace_outer_bindings() {
    run(
        "program P;\n\nbegin\n  var X: integer := 10;\n  for I: integer := 0 to 2 do\n    case I of\n      when 0:\n        var X: string := 'zero';\n        if X <> 'zero' then\n          panic('first');\n        end if;\n      when 1:\n        var X: integer := 1;\n        if X <> 1 then\n          panic('second');\n        end if;\n      else\n        var X: boolean := true;\n        if not X then\n          panic('else');\n        end if;\n    end case;\n\n    if X <> 10 then\n      panic('outer');\n    end if;\n  end for;\n\n  case Option.Some(1) of\n    when Option.Some(const Value):\n      null;\n    when Option.None:\n      begin\n        var X: string := 'none';\n      end;\n  end case;\n\n  if X <> 10 then\n    panic('variant outer');\n  end if;\nend program;\n",
    );
}

#[test]
fn forward_types_updates_and_closure_expression_boundaries_execute() {
    run(
        "program P;\n\nfunction Twice(X: Later): Later;\nbegin\n  return X * 2;\nend function;\n\ntype Boxed = record\n  Value: Later;\nend record;\n\ntype Later = integer;\n\nfunction Apply(F: function(X: integer): integer; X: integer): integer;\nbegin\n  return F(X);\nend function;\n\nbegin\n  var B: Boxed := Boxed(Value := Twice(3));\n  var C: Boxed := B with Value := Apply(function(X: integer): integer begin\n    return X + 1;\n  end function, B.Value); end with;\n\n  if C.Value <> 7 then\n    panic('closure/update');\n  end if;\n\n  if B.Value <> 6 then\n    panic('copy');\n  end if;\nend program;\n",
    );
}
