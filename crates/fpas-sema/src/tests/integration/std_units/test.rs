use super::check_ok;

#[test]
fn std_test_assertions_resolve() {
    check_ok(
        r#"program T;
uses Std.Test as Test;
begin
  Test.AssertEquals(4, 2 + 2);
  Test.AssertTrue(1 + 1 = 2);
  Test.AssertFalse(1 = 2);
  Test.AssertEquals('ok', 'o' + 'k');
end program;"#,
    );
}
