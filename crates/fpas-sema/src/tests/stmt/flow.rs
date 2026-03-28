use super::super::check_ok;

#[test]
fn case_ordinal_valid() {
    check_ok(
        "program T; begin \
         case 1 of \
           1: return; \
           2: return \
         end \
         end.",
    );
}
