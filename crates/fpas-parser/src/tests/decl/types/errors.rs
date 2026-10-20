//! Contextual Error syntax and recovery preserve the reserved Result constructor.
use super::*;

#[test]
fn native_error_syntax_rejects_alternative_forms_and_nonliteral_messages() {
    for declaration in [
        "type Failure = error end error;",
        "type Failure = error Failed = 1; end error;",
        "type Failure = error Failed = ''; end error;",
        "type Failure = error Failed = 'a' + 'b'; end error;",
        "type Failure = error Failed(Value: integer := 1); end error;",
        "type Failure = error Failed; end type;",
        "type Failure = error function Message(): string; begin return ''; end function; end error;",
        "type Root = Result of (integer, Error);",
    ] {
        let source = format!("program T; {declaration} begin end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(!errors.is_empty(), "accepted {declaration}");
    }
}
