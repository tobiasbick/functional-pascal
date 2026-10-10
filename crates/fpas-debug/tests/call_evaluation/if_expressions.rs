//! `if` expressions in debugger evaluation evaluate only the selected branch.
//! See `docs/pascal/language/control-flow/if-then-else.md` and `docs/pascal/tools/debugger.md`.

use super::named_arguments::{evaluate, server};

#[test]
fn if_expressions_select_one_branch_and_require_boolean_conditions() {
    let mut server = server(
        "program DebugIf;
function Crash(): integer;
begin
  panic('unselected branch must not run');
  return 0;
end function;
begin end.",
    );
    for (expression, expected) in [
        ("if 1 < 2 then 'a' else 'b' end if", "'a'"),
        ("if 1 > 2 then 1 elsif 2 > 1 then 2 else 3 end if", "2"),
        ("1 + if false then 10 else 20 end if", "21"),
        ("if true then 7 else Crash() end if", "7"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["result"], expected,
            "{expression}: {response}"
        );
    }
    let response = evaluate(&mut server, "if 1 then 1 else 2 end if");
    assert_eq!(response["success"], false, "{response}");
    assert!(
        response.to_string().contains("condition must be boolean"),
        "{response}"
    );
}
