//! Callable invocation and explicit discard formatting.

mod common;

#[test]
fn arbitrary_invocation_and_discard_preserve_comments_and_round_trip() {
    for source in [
        "program P; begin discard Make()(1, 2); // result\n end program;",
        "program P; begin discard (function(X: integer): integer begin // body\n return X; end function)(42); end program;",
        "program P; begin ([Action])[0](); discard Callbacks[Index()](// argument\n Value()); end program;",
        "program P; begin discard MakeAFunctionWithALongName()(MakeAnArgumentWithALongName(), MakeAnotherArgumentWithALongName()); end program;",
    ] {
        common::assert_round_trip("callable target", source);
    }
}
