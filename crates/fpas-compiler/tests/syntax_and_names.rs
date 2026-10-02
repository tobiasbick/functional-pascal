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
    run("program P; uses Std.Fs as Files; uses Std.Math as Numbers;
        uses Std.Tasks as Tasks; uses Std.Str as Text;
        begin
          var ReadJob: task := go Files.ReadText('.temp-data/missing-alias-task-file');
          case Tasks.Wait(ReadJob) of when Ok(Value): panic('unexpected file');
            when Error(Message): null; end case;
          var IntJob: task := go Numbers.Abs(-2);
          var RealJob: task := go Numbers.Abs(-2.5);
          if Tasks.Wait(IntJob) <> 2 then panic('integer task'); end if;
          if Tasks.Wait(RealJob) <> 2.5 then panic('real task'); end if;
          var Formatted: task := go Text.Format('%s-%d', 'a', 2);
          if Tasks.Wait(Formatted) <> 'a-2' then panic('format task'); end if;
        end program;");
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
        "program P; uses Std.Str as Text; uses Std.Math as Numbers; uses Std.Json as Json;
        begin
          if Text.Trim(' a ') <> 'a' then panic('text alias'); end if;
          if Numbers.Abs(-5) <> 5 then panic('number alias'); end if;
          var V: Json.JsonValue := Json.JsonValue.NullValue;
          case V of when Json.JsonValue.NullValue: null; else panic('json variant'); end case;
        end program;",
    );
}

#[test]
fn case_arm_locals_do_not_replace_outer_bindings() {
    run("program P; begin
        var X: integer := 10;
        for I: integer := 0 to 2 do
          case I of
            when 0: var X: string := 'zero'; if X <> 'zero' then panic('first'); end if;
            when 1: var X: integer := 1; if X <> 1 then panic('second'); end if;
            else var X: boolean := true; if not X then panic('else'); end if;
          end case;
          if X <> 10 then panic('outer'); end if;
        end for;
        case Some(1) of
          when Some(Value): null;
          else var X: string := 'none';
        end case;
        if X <> 10 then panic('variant outer'); end if;
        end program;");
}

#[test]
fn forward_types_updates_and_closure_expression_boundaries_execute() {
    run("program P;
        function Twice(X: Later): Later; begin return X * 2; end function;
        type Boxed = record Value: Later; end record;
        type Later = integer;
        function Apply(F: function(X: integer): integer; X: integer): integer; begin return F(X); end function;
        begin
          var B: Boxed := record Value := Twice(3); end record;
          var C: Boxed := B with Value := Apply(function(X: integer): integer begin return X + 1; end function, B.Value); end with;
          if C.Value <> 7 then panic('closure/update'); end if;
          if B.Value <> 6 then panic('copy'); end if;
        end program;");
}
