use super::super::lex_with_errors;

#[test]
fn unterminated_brace_comment() {
    let (_, errs) = lex_with_errors("{ not closed");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("{"));
}

#[test]
fn unterminated_paren_comment() {
    let (_, errs) = lex_with_errors("(* not closed");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("(*"));
}
