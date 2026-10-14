//! Enum identity and ordering survive unit reuse; legacy sidecars cannot skip checks.

use super::{BuildOptions, Digest, Fixture, assert_output, decode, encode, fs, write};
use fpas_diagnostics::codes::{SEMA_BACKING_VALUE_IN_DATA_ENUM, SEMA_DUPLICATE_ENUM_BACKING_VALUE};

#[test]
fn enum_values_preserve_backing_identity_in_cold_and_reused_interfaces() {
    let fixture = Fixture::create();
    write(
        &fixture.base,
        "unit Demo.Base;
        public type Phase = enum High = 40; Next; Low = 5; end enum;
        public type Alias = Phase;
        public const Selected: Alias := Alias.Next;
        end unit;",
    );
    write(
        &fixture.root.join("src/consumer.fpas"),
        "unit Demo.Consumer; uses Demo.Base as API;
        public function Run(): integer;
        begin
          const Sorted := [API.Phase.Next, API.Alias.Low, API.Phase.High].Sort();
          if (Sorted[0] <> API.Phase.Low) or (Sorted[1] <> API.Phase.High) or
             (Sorted[2] <> API.Phase.Next) then panic('imported backing order'); end if;
          return case API.Selected of
            when API.Phase.High: 40;
            when API.Phase.Next: 41;
            when API.Phase.Low: 5;
          end case;
        end function; end unit;",
    );
    let cold = fixture.build().expect("cold enum build");
    assert_eq!(cold.counters().compiled, 2);
    assert_output(cold, "41");
    // Keep the enum sidecar but rebuild its consumer to exercise the loaded interface.
    fs::remove_file(fixture.root.join("src/consumer.fpascu")).expect("remove consumer sidecar");
    let reused = fixture
        .build()
        .expect("consumer build with cached enum interface");
    assert_eq!(reused.counters().sidecar_reused, 1);
    assert_eq!(reused.counters().compiled, 1);
    assert_output(reused, "41");
    fs::remove_dir_all(&fixture.root).expect("remove enum fixture");
}

#[test]
fn enum_values_recheck_legacy_sidecars_even_with_matching_source_hashes() {
    for (members, code) in [
        ("First = 1; Second = 1;", SEMA_DUPLICATE_ENUM_BACKING_VALUE),
        (
            "First = 1; Second(Value: integer);",
            SEMA_BACKING_VALUE_IN_DATA_ENUM,
        ),
    ] {
        let fixture = Fixture::create();
        let valid =
            "unit Demo.Base; public type Phase = enum First = 1; Second = 2; end enum; end unit;";
        write(&fixture.base, valid);
        write(
            &fixture.root.join("src/consumer.fpas"),
            "unit Demo.Consumer; uses Demo.Base as API;
             public function Run(): integer; begin return 42; end function; end unit;",
        );
        let mut legacy_options = BuildOptions::default();
        legacy_options.compiler_version.push_str("-before-ap44");
        fixture
            .build_with_options(&legacy_options)
            .expect("legacy cache fixture");

        let source =
            format!("unit Demo.Base; public type Phase = enum {members} end enum; end unit;");
        write(&fixture.base, &source);
        let sidecar = fixture.base.with_extension("fpascu");
        let mut legacy =
            decode(&fs::read(&sidecar).expect("sidecar bytes")).expect("sidecar format");
        // Simulate a source-matching artifact emitted before the stricter compiler.
        legacy.identity.source_hash = Digest::of(source.as_bytes());
        fs::write(&sidecar, encode(&legacy).expect("legacy envelope")).expect("legacy sidecar");
        let failure = match fixture.build() {
            Err(error) => error,
            Ok(_) => panic!("legacy sidecar bypassed enum validation: {members}"),
        };
        assert!(
            failure
                .diagnostics()
                .iter()
                .any(|error| error.diagnostic.code == code),
            "{failure:#?}"
        );
        fs::remove_dir_all(&fixture.root).expect("remove legacy enum fixture");
    }
}
