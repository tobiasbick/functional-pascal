//! Cached unit interfaces preserve finite coverage after aliases and bound changes.

use super::{Fixture, assert_output, fs, write};

#[test]
fn warm_unit_interfaces_preserve_subrange_bounds_and_distinct_boolean_coverage() {
    let fixture = Fixture::create();
    let base = |upper| {
        format!(
            "unit Demo.Base;
        public type Digit = 1..{upper};
        public type Flag = distinct boolean;
        public const Enabled: Flag := Flag(true);
        public const Disabled: Flag := Flag(false);
        end unit;"
        )
    };
    write(&fixture.base, &base(2));
    write(
        &fixture.root.join("src/consumer.fpas"),
        "unit Demo.Consumer; uses Demo.Base as API;
        type Alias = API.Digit;
        public function Run(): integer;
        begin
            const Number: integer := case Alias(2) of when 1..2: 7; end case;
            return case API.Enabled of
                when API.Enabled: Number;
                when API.Disabled: 0;
            end case;
        end function; end unit;",
    );
    let cold = fixture.build().expect("cold finite coverage build");
    assert_eq!(cold.counters().compiled, 2);
    assert_output(cold, "7");
    let warm = fixture.build().expect("warm finite coverage build");
    assert_eq!(warm.counters().parsed, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_output(warm, "7");

    write(&fixture.base, &base(3));
    let failure = match fixture.build() {
        Err(error) => error,
        Ok(_) => panic!("extended subrange did not expose missing coverage"),
    };
    assert!(
        failure
            .diagnostics()
            .iter()
            .any(|error| error.diagnostic.code
                == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE
                && error.diagnostic.message.contains("missing 3")),
        "{failure:#?}"
    );
    fs::remove_dir_all(&fixture.root).ok();
}
