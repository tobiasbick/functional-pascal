use crate::tests::parse_with_errors;

#[test]
fn if_missing_then() {
    let (_, errs) = parse_with_errors("program T; begin if X > 0 Y := 1 end.");
    assert!(!errs.is_empty(), "expected error for missing 'then'");
}

#[test]
fn if_missing_condition() {
    let (_, errs) = parse_with_errors("program T; begin if then Y := 1 end.");
    assert!(!errs.is_empty(), "expected error for missing condition");
}

#[test]
fn if_empty_then_branch() {
    let (_, errs) = parse_with_errors("program T; begin if X then else Y := 1 end.");
    assert!(!errs.is_empty(), "expected error for empty then-branch");
}
