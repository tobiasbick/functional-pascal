//! `Std.Test` symbol names and registry group.

std_symbol!(STD_TEST_ASSERT_TRUE = std_test!("AssertTrue"));
std_symbol!(STD_TEST_ASSERT_FALSE = std_test!("AssertFalse"));
std_symbol!(STD_TEST_ASSERT_EQUALS = std_test!("AssertEquals"));
std_symbol!(STD_TEST_FAIL = std_test!("Fail"));
std_symbol!(STD_TEST_SKIP = std_test!("Skip"));
std_symbol!(STD_TEST_ASSERT_SCREEN_LINE = std_test!("AssertScreenLine"));
std_symbol!(STD_TEST_ASSERT_SCREEN_CELL = std_test!("AssertScreenCell"));
std_symbol!(STD_TEST_PUSH_READLN = std_test!("PushReadLn"));

pub(in crate::std_units) const STD_TEST_SYMBOLS: &[&str] = &[
    STD_TEST_ASSERT_TRUE,
    STD_TEST_ASSERT_FALSE,
    STD_TEST_ASSERT_EQUALS,
    STD_TEST_FAIL,
    STD_TEST_SKIP,
    STD_TEST_ASSERT_SCREEN_LINE,
    STD_TEST_ASSERT_SCREEN_CELL,
    STD_TEST_PUSH_READLN,
];
