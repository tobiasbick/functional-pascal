//! `Std.Test` symbol names and registry group.

pub const STD_TEST_ASSERT_TRUE: &str = std_test!("AssertTrue");
pub const STD_TEST_ASSERT_FALSE: &str = std_test!("AssertFalse");
pub const STD_TEST_ASSERT_EQUALS: &str = std_test!("AssertEquals");
pub const STD_TEST_FAIL: &str = std_test!("Fail");
pub const STD_TEST_SKIP: &str = std_test!("Skip");
pub const STD_TEST_ASSERT_SCREEN_LINE: &str = std_test!("AssertScreenLine");
pub const STD_TEST_ASSERT_SCREEN_CELL: &str = std_test!("AssertScreenCell");
pub const STD_TEST_PUSH_READLN: &str = std_test!("PushReadLn");

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
