//! Standard constant guards share lowering values and preserve lexical resolution.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_INVALID_STATIC_OPERATION;

#[test]
fn builtin_constants_select_reached_and_skipped_static_operands() {
    for (guard, expected) in [
        ("M.pI > 3.0", true),
        ("M.Pi < 3.0", false),
        ("[M.Pi] = [M.Pi]", true),
        ("[M.Pi] = [3.0]", false),
        ("M.Pi in [0.0, M.Pi]", true),
        ("Option.Some(M.Pi) = Option.Some(M.Pi)", true),
        ("['pi': M.Pi] = ['pi': M.Pi]", true),
        ("C.Black = 0", true),
        ("[C.Font8x8] = [256]", true),
        ("C.White + C.Blink = 143", true),
    ] {
        for operator in ["and", "or"] {
            let source = format!(
                "program T; uses Std.Math as M; uses Std.Console as C; begin const Value := ({guard}) {operator} (1 div 0 = 0); end program;"
            );
            if (operator == "and") == expected {
                let errors = check_errors(&source);
                assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
                assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
            } else {
                check_ok(&source);
            }
        }
    }
}

#[test]
fn every_registered_builtin_constant_has_a_matching_static_value() {
    for unit in fpas_std::STD_UNITS_INTRINSIC {
        for symbol in crate::intrinsic_std_symbols(unit) {
            if symbol.kind != crate::IntrinsicStdSymbolKind::Constant {
                continue;
            }
            let value = fpas_std::intrinsic_std_constant_value(&symbol.qualified_name)
                .expect("registered intrinsic constant value");
            let literal = match (&symbol.ty, value) {
                (crate::Ty::Integer, fpas_bytecode::Value::Integer(value)) => value.to_string(),
                (crate::Ty::Real, fpas_bytecode::Value::Real(value)) => value.to_string(),
                other => panic!("constant type/value mismatch: {symbol:?}: {other:?}"),
            };
            let name = symbol.qualified_name.rsplit('.').next().unwrap();
            let source = format!(
                "program T; uses {unit} as U; begin const Value := (U.{name} = {literal}) and (1 mod 0 = 0); end program;"
            );
            let errors = check_errors(&source);
            assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
            assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
        }
    }
}

#[test]
fn builtin_constant_lookup_does_not_replace_shadowed_bindings() {
    check_ok(
        "program T; uses Std.Math as Math;
        const Pi: real := Math.Pi;
        begin
          begin const Pi := 0.0; const Skip := (Pi > 3.0) and (1 div 0 = 0); end;
          const Other := (Pi > 3.0) or (1 mod 0 = 0);
        end program;",
    );
    for source in [
        "program T; begin const Value := Math.Pi; end program;",
        "program T; uses Std.Math as Math; begin const Value := Pi; end program;",
        "program T; uses Std.Math as Math; begin const Value := Std.Math.Pi; end program;",
        "program T; uses Std.Math as Math; begin const Math := 0; end program;",
    ] {
        assert!(!check_errors(source).is_empty(), "{source}");
    }
}
