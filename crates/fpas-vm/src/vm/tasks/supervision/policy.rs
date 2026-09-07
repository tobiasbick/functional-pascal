//! Fixed-delay retry admission and input bounds.

/// Remaining retries and fixed pause between attempts.
pub(in crate::vm) struct RetryPolicy {
    pub(super) remaining: u16,
    pub(super) backoff_millis: u64,
}

impl RetryPolicy {
    /// Validate public bounds before allocating a task or retaining its captures.
    pub(in crate::vm) fn new(retries: i64, backoff_millis: i64) -> Result<Self, String> {
        if !(0..=1023).contains(&retries) {
            return Err("RetryLimit must be in 0..1023".into());
        }
        if !(0..=60000).contains(&backoff_millis) {
            return Err("BackoffMillis must be in 0..60000".into());
        }
        Ok(Self {
            remaining: retries as u16,
            backoff_millis: backoff_millis as u64,
        })
    }
}
