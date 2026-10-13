//! Initialization safety survives independently encoded callback interfaces.

use super::{analyze_unit, parse_unit};
use fpas_diagnostics::codes::SEMA_UNINITIALIZED_DEPENDENCY;
use fpas_unit::interface::{decode_interface, encode_interface};

#[test]
fn imported_callback_dependencies_survive_encoding_and_aliases() {
    let dependency = parse_unit(
        "unit Demo.Callbacks;
        public function Apply(F: function(): integer): integer;
        begin return Wrapper(F); end function;
        function Wrapper(F: function(): integer): integer; begin return F(); end function;
        end unit;",
    );
    let analysis = analyze_unit(&dependency, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.unwrap();
    let restored = decode_interface(&encode_interface(&interface).unwrap()).unwrap();
    for interface in [&interface, &restored] {
        for (values, rejected) in [
            (
                "const First: integer := API.Apply(ReadLater); const Later: integer := 7;",
                true,
            ),
            (
                "const Later: integer := 7; const First: integer := API.Apply(ReadLater);",
                false,
            ),
        ] {
            let consumer = parse_unit(&format!("unit Demo.Consumer; uses Demo.Callbacks as API;
                {values} function ReadLater(): integer; begin return Later; end function; end unit;"));
            let errors = analyze_unit(&consumer, std::slice::from_ref(interface))
                .unwrap()
                .metadata
                .errors;
            if rejected {
                assert!(
                    errors
                        .iter()
                        .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY
                            && error.message.contains("Later")),
                    "{errors:#?}"
                );
            } else {
                assert!(errors.is_empty(), "{errors:#?}");
            }
        }
    }
}
