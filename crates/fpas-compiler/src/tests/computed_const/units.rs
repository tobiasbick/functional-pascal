use super::*;

#[test]
fn computed_and_aggregate_constants_link_through_dependency_initializers() {
    let original = compiled_unit(
        "unit Original;
        var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
        public const First: integer := Next(); public const Second: integer := Next();
        public const Fixed: integer := 6 * 7;
        public const Values: array of integer := [1, 2];
        public function Count(): integer; begin return Calls; end function; end unit;",
        &[],
    );
    let facade = compiled_unit(
        "unit Facade; uses Original as OriginalUnit;\n        public const Derived: integer := OriginalUnit.Second + OriginalUnit.Fixed;\n        public const Values: array of integer := OriginalUnit.Values;\n        end unit;",
        std::slice::from_ref(&original.interface),
    );
    let interfaces = vec![original.interface.clone(), facade.interface.clone()];
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            "program Consumer;\n        uses Original as OriginalUnit, Facade as FacadeUnit;\n        const Final: integer := FacadeUnit.Derived + OriginalUnit.First;\n        begin const Original: integer := 0;\n        if Final <> 45 or OriginalUnit.Count() <> 2 or OriginalUnit.First <> 1 then panic('initialization order'); end if;\n        if FacadeUnit.Values[1] <> 2 then panic('aggregate import'); end if;\n        case 42 of when OriginalUnit.Fixed: null; else panic('static import'); end case; end.",
        ),
        &interfaces,
        &interfaces,
    )
    .expect("consumer object");
    let linked = fpas_linker::link_objects(&[original.object, facade.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(linked)
        .run()
        .expect("execution");
}

#[test]
fn writable_exports_and_value_parameters_survive_interface_round_trip() {
    let state = compiled_unit(
        "unit State;
        public var Current: integer := 1;
        public var Values: array of integer := [1];
        public const Fixed: integer := 1;
        public const Frozen: array of integer := [1];
        public function Increment(Value: integer): integer;
        begin var LocalValue: integer := Value;
          LocalValue := LocalValue + 1; return LocalValue;
        end function; end unit;",
        &[],
    );
    let interfaces = [state.interface.clone()];
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            "program Consumer; uses State as StateUnit;\n            begin StateUnit.Current := StateUnit.Current + 3; StateUnit.Values[0] := 9;\n              const F: function(Value: integer): integer := StateUnit.Increment;\n              if F(StateUnit.Current) <> 5 or StateUnit.Current <> 4 or StateUnit.Values[0] <> 9 then panic('interface'); end if;\n            end.",
        ),
        &interfaces,
        &interfaces,
    )
    .expect("consumer object");
    for target in ["StateUnit.Fixed", "StateUnit.Frozen[0]"] {
        let source =
            format!("program Consumer; uses State as StateUnit; begin {target} := 2; end.");
        let errors = crate::compile_program_object_with_support(
            &parse_ok(&source),
            &interfaces,
            &interfaces,
        )
        .expect_err("exported constant stays read-only");
        assert!(
            errors
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT),
            "{errors:#?}"
        );
    }
    let linked = fpas_linker::link_objects(&[state.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(linked)
        .run()
        .expect("execution");
}

#[test]
fn static_record_patterns_link_while_aggregate_values_remain_runtime_globals() {
    let original = compiled_unit("unit Original;
        public type Color = enum Red; Green; end enum;
        public type Flags = record public Enabled: boolean; public Color: Color; end record;
        public type Settings = record public Flags: Flags; public Values: array of integer; end record;
        public const Config: Settings := Settings(Flags := Flags(Enabled := true, Color := Color.Red), Values := [1, 2]);
        end unit;", &[]);
    let facade = compiled_unit("unit Facade; uses Original as O;
        public const Copy: O.Settings := O.Config;
        public const Other: O.Flags := Copy.Flags with Enabled := false; Color := O.Color.Green; end with;
        public const Enabled: boolean := Copy.Flags.Enabled;
        end unit;", std::slice::from_ref(&original.interface));
    let interfaces = [original.interface.clone(), facade.interface.clone()];
    let root = crate::compile_program_object_with_support(&parse_ok(
        "program Consumer; uses Original as O, Facade as F;
        function Score(B: Option of boolean): integer;
        begin case B of when Some(F.Enabled): return 1;
        when Some(F.Other.Enabled): return 2; when None: return 4; end case; end function;
        function ColorScore(C: Option of O.Color): integer;
        begin case C of when Some(F.Copy.Flags.Color): return 8;
        when Some(F.Other.Color): return 16; when None: return 32; end case; end function;
        begin if Score(Some(true)) + Score(Some(false)) + Score(None) <> 7 then panic('imported bool dispatch'); end if;
        if ColorScore(Some(O.Color.Red)) + ColorScore(Some(O.Color.Green)) + ColorScore(None) <> 56 then panic('imported enum dispatch'); end if;
        if F.Copy.Values[1] <> 2 or F.Other.Enabled then panic('record globals'); end if;
        end."
    ), &interfaces, &interfaces).expect("consumer object");
    let linked = fpas_linker::link_objects(&[original.object, facade.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(linked)
        .run()
        .expect("execution");
}

#[test]
fn imported_constant_receivers_link_and_execute_native_calls() {
    let values = compiled_unit(
        "unit Values;
        public const Code: string := '?';
        public const Empty: string := '';
        public const Unicode: string := 'é😀';
        public const Parts: array of string := ['a', '😀'];
        var Calls: integer := 0;
        function Build(): string; begin Calls := Calls + 1; return 'runtime'; end function;
        public const Dynamic: string := Build();
        public var Mutable: string := 'before';
        public function Count(): integer; begin return Calls; end function;
        end unit;",
        &[],
    );
    let interfaces = [values.interface.clone()];
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            "program Consumer; uses Values as Texts, Std.Tasks as Tasks;
            var Order: string := '';
            function PickIndex(): integer; begin Order := Order + 'I'; return 1; end function;
            function Copies(): integer; begin Order := Order + 'A'; return 2; end function;
            begin
              if Texts.Code.RepeatStr(2) <> '??' then panic('literal receiver'); end if;
              if tExTs.cOdE.Length() <> 1 then panic('case insensitive receiver'); end if;
              if Texts.Empty.RepeatStr(3) <> '' or Texts.Code.RepeatStr(0) <> '' then panic('empty repetition'); end if;
              if Texts.Unicode.RepeatStr(2).Length() <> 4 then panic('unicode chain'); end if;
              if Texts.Unicode[1].RepeatStr(2) <> '😀😀' then panic('indexed constant receiver'); end if;
              if Texts.Parts[1].RepeatStr(2) <> '😀😀' then panic('aggregate receiver'); end if;
              if Texts.Parts[Texts.Code.Length()].RepeatStr(2) <> '😀😀' then panic('native call in index'); end if;
              if Texts.Parts[PickIndex()].RepeatStr(Copies()) <> '😀😀' then panic('computed index'); end if;
              if Order <> 'IA' then panic('receiver and argument evaluate once in order'); end if;
              if Texts.Dynamic.RepeatStr(2) <> 'runtimeruntime' or Texts.Count() <> 1 then panic('computed constant'); end if;
              Texts.Mutable := 'after';
              if Texts.Mutable.RepeatStr(2) <> 'afterafter' then panic('mutable global'); end if;
              scope
                const Worker := go Texts.Code.RepeatStr(2);
                const Output := Tasks.Wait(Worker);
                if Output <> '??' then panic('spawned constant receiver'); end if;
              end scope;
            end.",
        ),
        &interfaces,
        &interfaces,
    )
    .expect("native calls on imported constants compile");
    let linked = fpas_linker::link_objects(&[values.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(linked)
        .run()
        .expect("execution");
}

#[test]
fn imported_constant_receivers_remain_read_only_and_type_checked() {
    let values = compiled_unit(
        "unit Values; public const Code: string := '?'; end unit;",
        &[],
    );
    let interfaces = [values.interface];
    for (call, expected) in [
        (
            "Texts.Code.RepeatStrInPlace(2)",
            fpas_diagnostics::codes::SEMA_INVALID_VAR_ARGUMENT,
        ),
        (
            "discard Texts.Code.RepeatStr('two')",
            fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
        ),
    ] {
        let source = format!("program Consumer; uses Values as Texts; begin {call}; end.");
        let errors = crate::compile_program_object_with_support(
            &parse_ok(&source),
            &interfaces,
            &interfaces,
        )
        .expect_err("invalid constant call must be rejected semantically");
        assert!(
            errors.iter().any(|error| error.code == expected),
            "{errors:#?}"
        );
        assert!(
            errors
                .iter()
                .all(|error| error.code
                    != fpas_diagnostics::codes::INTERNAL_COMPILER_INVARIANT_FAILURE),
            "{errors:#?}"
        );
    }
}
