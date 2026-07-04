//! `Std.Time` symbol names and registry group.

pub const STD_TIME_TIMESTAMP_MILLIS: &str = std_time!("TimestampMillis");
pub const STD_TIME_MONOTONIC_MILLIS: &str = std_time!("MonotonicMillis");
pub const STD_TIME_ELAPSED_MILLIS: &str = std_time!("ElapsedMillis");
pub const STD_TIME_SLEEP: &str = std_time!("Sleep");

pub(in crate::std_units) const STD_TIME_SYMBOLS: &[&str] = &[
    STD_TIME_TIMESTAMP_MILLIS,
    STD_TIME_MONOTONIC_MILLIS,
    STD_TIME_ELAPSED_MILLIS,
    STD_TIME_SLEEP,
];
