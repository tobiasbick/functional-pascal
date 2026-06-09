use super::check_ok;

#[test]
fn std_test_assertions_resolve() {
    check_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(4, 2 + 2);
  AssertTrue(1 + 1 = 2);
  AssertFalse(1 = 2);
  AssertEquals('ok', 'o' + 'k')
end.",
    );
}
