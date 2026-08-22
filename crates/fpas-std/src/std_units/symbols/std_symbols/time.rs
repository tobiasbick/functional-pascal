//! `Std.Time` symbol names and registry group.

std_symbol!(STD_TIME_TIMESTAMP_MILLIS = std_time!("TimestampMillis"));
std_symbol!(STD_TIME_MONOTONIC_MILLIS = std_time!("MonotonicMillis"));
std_symbol!(STD_TIME_ELAPSED_MILLIS = std_time!("ElapsedMillis"));
std_symbol!(STD_TIME_SLEEP = std_time!("Sleep"));

pub(in crate::std_units) const STD_TIME_SYMBOLS: &[&str] = &[
    STD_TIME_TIMESTAMP_MILLIS,
    STD_TIME_MONOTONIC_MILLIS,
    STD_TIME_ELAPSED_MILLIS,
    STD_TIME_SLEEP,
];
