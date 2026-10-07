use super::*;
use fpas_unit::interface::{decode_interface, encode_interface};

fn unit(
    source: &str,
    interfaces: &[fpas_unit::interface::UnitInterface],
) -> crate::CompiledUnitObject {
    let (parsed, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let fpas_parser::CompilationUnit::Unit(unit) = parsed else {
        panic!("unit expected");
    };
    let mut compiled = crate::compile_unit_object(&unit, interfaces).expect("compiled unit");
    compiled.interface =
        decode_interface(&encode_interface(&compiled.interface).expect("encode")).expect("decode");
    compiled
}

#[test]
fn computed_and_aggregate_constants_link_through_dependency_initializers() {
    let original = unit(
        "unit Original;
        var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
        public const First: integer := Next(); public const Second: integer := Next();
        public const Fixed: integer := 6 * 7;
        public const Values: array of integer := [1, 2];
        public function Count(): integer; begin return Calls; end function; end unit;",
        &[],
    );
    let facade = unit(
        "unit Facade; uses Original;
        public const Derived: integer := Original.Second + Original.Fixed;
        public const Values: array of integer := Original.Values;
        end unit;",
        std::slice::from_ref(&original.interface),
    );
    let interfaces = vec![original.interface.clone(), facade.interface.clone()];
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            "program Consumer;
        uses Original, Facade;
        const Final: integer := Facade.Derived + Original.First;
        begin const Original: integer := 0;
        if Final <> 45 or Original.Count() <> 2 or Original.First <> 1 then panic('initialization order'); end if;
        if Facade.Values[1] <> 2 then panic('aggregate import'); end if;
        case 42 of when Original.Fixed: null; else panic('static import'); end case; end.",
        ),
        &interfaces,
        &interfaces,
    )
    .expect("consumer object");
    let linked = fpas_linker::link_objects(&[original.object, facade.object], &root).expect("link");
    fpas_vm::Vm::new(linked).run().expect("execution");
}

#[test]
fn writable_exports_and_value_parameters_survive_interface_round_trip() {
    let state = unit(
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
            "program Consumer; uses State;
            begin State.Current := State.Current + 3; State.Values[0] := 9;
              const F: function(Value: integer): integer := Increment;
              if F(State.Current) <> 5 or State.Current <> 4 or State.Values[0] <> 9 then panic('interface'); end if;
            end.",
        ),
        &interfaces,
        &interfaces,
    )
    .expect("consumer object");
    for target in ["State.Fixed", "State.Frozen[0]"] {
        let source = format!("program Consumer; uses State; begin {target} := 2; end.");
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
    fpas_vm::Vm::new(linked).run().expect("execution");
}
