use super::check_ok;

#[test]
fn std_test_assertions_resolve() {
    check_ok(
        "program T;\nuses Std.Test as Test;\nbegin\n  Test.AssertEquals(4, 2 + 2);\n  Test.AssertTrue(1 + 1 = 2);\n  Test.AssertFalse(1 = 2);\n  Test.AssertEquals('ok', 'o' + 'k');\nend.",
    );
}
