//! Warm dependency reuse preserves imported callback initialization checks.

use super::{Fixture, assert_output, fs, write};

#[test]
fn warm_sidecars_preserve_forward_calls_and_callback_dependency_checks() {
    let fixture = Fixture::create();
    write(
        &fixture.base,
        "unit Demo.Base;
        public function Apply(F: function(): integer): integer;
        begin return Invoke(F); end function;
        function Invoke(F: function(): integer): integer; begin return F(); end function;
        end unit;",
    );
    let consumer = fixture.root.join("src/consumer.fpas");
    write(
        &consumer,
        "unit Demo.Consumer; uses Demo.Base as API;
        const Later: integer := 7;
        const Initial: integer := API.Apply(ReadLater);
        function ReadLater(): integer; begin return Later; end function;
        public function Run(): integer; begin return Initial; end function;
        end unit;",
    );
    let cold = fixture.build().expect("cold callback build");
    assert_eq!(cold.counters().compiled, 2);
    assert_output(cold, "7");
    let warm = fixture.build().expect("warm callback build");
    assert_eq!(warm.counters().parsed, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_output(warm, "7");

    write(
        &consumer,
        "unit Demo.Consumer; uses Demo.Base as API;
        const Initial: integer := API.Apply(ReadLater);
        const Later: integer := 7;
        function ReadLater(): integer; begin return Later; end function;
        public function Run(): integer; begin return Initial; end function;
        end unit;",
    );
    let failure = match fixture.build() {
        Err(error) => error,
        Ok(_) => panic!("premature callback dependency was accepted"),
    };
    assert!(
        failure
            .diagnostics()
            .iter()
            .any(|error| error.diagnostic.code
                == fpas_diagnostics::codes::SEMA_UNINITIALIZED_DEPENDENCY
                && error.diagnostic.message.contains("Later")),
        "{failure:#?}"
    );
    fs::remove_dir_all(&fixture.root).ok();
}
