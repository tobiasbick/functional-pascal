//! Enum patterns resolve their complete nominal variant identity.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

#[test]
fn payload_patterns_reject_foreign_declarations_and_unknown_qualifiers() {
    for (pattern, code) in [
        ("OtherChoice.Present(Value)", SEMA_TYPE_MISMATCH),
        ("MissingQualifier.Present(Value)", SEMA_UNKNOWN_NAME),
        ("MissingQualifier.Missing", SEMA_UNKNOWN_NAME),
        ("ExpectedChoice[0].Present(Value)", SEMA_TYPE_MISMATCH),
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type ExpectedChoice = enum Present(Value: integer); Missing; end enum;
            type OtherChoice = enum Present(Value: string); Missing; end enum;
            var Item: ExpectedChoice := ExpectedChoice.Present(42);
            begin case Item of
              when {pattern}: null;
              when ExpectedChoice.Present(Value): null;
              when ExpectedChoice.Missing: null;
            end case; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == code),
            "{pattern}: {errors:#?}"
        );
    }
}

#[test]
fn payload_patterns_reject_matching_names_from_another_nominal_type() {
    let errors = check_errors(
        "program Main;\n\ntype FirstChoice = enum\n  Present(Value: integer);\n  Missing;\nend enum;\n\ntype SecondChoice = enum\n  Present(Value: integer);\n  Missing;\nend enum;\n\nvar Item: FirstChoice := FirstChoice.Present(42);\n\nbegin\n  case Item of\n    when SecondChoice.Present(const Value):\n      null;\n    when FirstChoice.Present(const Value):\n      null;\n    when FirstChoice.Missing:\n      null;\n  end case;\nend program;\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("does not belong")),
        "{errors:#?}"
    );
}

#[test]
fn qualified_patterns_use_expected_generic_arguments() {
    check_ok(
        "program Main;\n\ntype Choice of (T) = enum\n  Present(Value: T);\n  Missing;\nend enum;\n\nprocedure Inspect(Item: Choice of (integer));\nbegin\n  case Item of\n    when Choice.Present(const Value):\n      discard Value + 1;\n    when Choice.Missing:\n      null;\n  end case;\nend procedure;\n\nbegin\n  null;\nend program;\n",
    );
}

#[test]
fn concrete_enum_alias_patterns_preserve_their_arguments() {
    check_ok(
        "program Main;\n\ntype Choice of (T) = enum\n  Present(Value: T);\n  Missing;\nend enum;\n\ntype IntegerChoice = Choice of (integer);\n\nvar Item: IntegerChoice := IntegerChoice.Present(42);\n\nbegin\n  case Item of\n    when IntegerChoice.Present(const Value):\n      discard Value + 1;\n    when IntegerChoice.Missing:\n      null;\n  end case;\nend program;\n",
    );
    let errors = check_errors(
        "program Main;\n\ntype Choice of (T) = enum\n  Present(Value: T);\n  Missing;\nend enum;\n\ntype IntegerChoice = Choice of (integer);\n\ntype TextChoice = Choice of (string);\n\nvar Item: IntegerChoice := IntegerChoice.Present(42);\n\nbegin\n  case Item of\n    when TextChoice.Present(const Value):\n      null;\n    when IntegerChoice.Present(const Value):\n      null;\n    when IntegerChoice.Missing:\n      null;\n  end case;\nend program;\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("does not belong")),
        "{errors:#?}"
    );
}

#[test]
fn recursive_generic_payloads_keep_their_instantiated_pattern_type() {
    check_ok(
        "program Main;\n\ntype Chain of (T) = enum\n  Empty;\n  Link(Value: T; Tail: Chain of (T));\nend enum;\n\nprocedure Inspect(Item: Chain of (integer));\nbegin\n  case Item of\n    when Chain.Link(const Value, const Tail):\n      case Tail of\n        when Chain.Link(const NextValue, const NextTail):\n          discard Value + NextValue;\n        when Chain.Empty:\n          null;\n      end case;\n    when Chain.Empty:\n      null;\n  end case;\nend procedure;\n\nbegin\n  null;\nend program;\n",
    );
}
