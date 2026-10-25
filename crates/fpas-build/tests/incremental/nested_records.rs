//! A cached transitive consumer must follow changes to nested record layouts.

use super::{Fixture, assert_output, fs, write};

#[test]
fn growing_a_nested_record_rebuilds_transitive_consumers_but_not_unrelated_units() {
    let fixture = Fixture::create();
    let state = |extra_field: &str, extra_value: &str, value: i32| {
        format!(
            "unit Demo.Base;
             public type State = record public Value: integer; {extra_field} end record;
             public function NewState(): State;
             begin return State(Value := {value}{extra_value}); end function;
             end unit;"
        )
    };
    write(&fixture.base, &state("", "", 42));
    write(
        &fixture.root.join("src/wrapper.fpas"),
        "unit Demo.Wrapper; uses Demo.Base as Base;
         public type Session = record public State: option of Base.State; end record;
         public function NewSession(): Session;
         begin return Session(State := Some(Base.NewState())); end function;
         end unit;",
    );
    write(
        &fixture.root.join("src/consumer.fpas"),
        "unit Demo.Consumer; uses Demo.Wrapper as Wrapper;
         public function Run(): integer;
         begin
           const Session := Wrapper.NewSession();
           case Session.State of
             when Some(const State): return State.Value;
             when None: return 0;
           end case;
         end function; end unit;",
    );
    write(
        &fixture.root.join("src/unrelated.fpas"),
        "unit Demo.Unrelated; public const Value: integer := 99; end unit;",
    );
    write(
        &fixture.root.join("src/main.fpas"),
        "program Demo;
         uses Demo.Consumer as Consumer, Demo.Unrelated as Unrelated, Std.Console as Console;
         begin if Unrelated.Value = 99 then Console.WriteLn(Consumer.Run()); end if; end.",
    );

    let cold = fixture.build().expect("cold nested record build");
    assert_output(cold, "42");
    let warm = fixture.build().expect("warm nested record build");
    assert_eq!(warm.counters().sidecar_reused, 4);
    assert_output(warm, "42");

    write(&fixture.base, &state("", "", 43));
    let body_change = fixture.build().expect("implementation-only leaf change");
    assert_eq!(body_change.counters().compiled, 1);
    assert_eq!(body_change.counters().sidecar_reused, 3);
    assert_output(body_change, "43");

    write(
        &fixture.base,
        &state("public Coins: integer;", ", Coins := 0", 43),
    );
    let grown = fixture
        .build()
        .expect("changed nested record must link with fresh consumer projections");
    assert_eq!(grown.counters().compiled, 3);
    assert_eq!(grown.counters().sidecar_reused, 1);
    assert_output(grown, "43");
    let warm = fixture.build().expect("warm expanded record build");
    assert_eq!(warm.counters().sidecar_reused, 4);
    assert_output(warm, "43");
    fs::remove_dir_all(&fixture.root).ok();
}
