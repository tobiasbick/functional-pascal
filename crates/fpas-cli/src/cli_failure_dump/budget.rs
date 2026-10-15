//! Count escaped UTF-8 JSON bytes before accepting optional report data.

use super::value::Policy;
use serde::Serialize;
use std::io::{self, Write};

/// Complete document bound, including the trailing newline.
pub(super) const MAX_BYTES: usize = 8 * 1024 * 1024;

/// Byte allowance for content; structural changes and completeness markers have a reserve.
pub(super) struct Budget {
    remaining: usize,
}

impl Budget {
    /// Reserve the base document plus room for completeness strings and a trailing newline.
    pub(super) fn new(base: &impl Serialize) -> Self {
        Self {
            remaining: MAX_BYTES
                .saturating_sub(size(base).unwrap_or(MAX_BYTES))
                .saturating_sub(4096),
        }
    }

    /// Charge an entire JSON component before retaining its borrowed planning entry.
    pub(super) fn take(&mut self, value: &impl Serialize, reserve: usize) -> bool {
        let Some(size) = size(value).and_then(|size| size.checked_add(reserve + 1)) else {
            return false;
        };
        if size > self.remaining {
            return false;
        }
        self.remaining -= size;
        true
    }

    /// Keep a bounded value prefix when its complete serialized form does not fit.
    pub(super) fn partial<T: Serialize>(&mut self, make: impl Fn(Policy) -> T) -> Option<Policy> {
        let fits = |policy| size(&make(policy)).is_some_and(|size| size + 1 <= self.remaining);
        let mut policy = Policy {
            children: 0,
            chars: 4096,
        };
        let (mut low, mut high, children): (usize, usize, bool) = if fits(policy) {
            (0, 256, true)
        } else {
            policy.chars = 0;
            if !fits(policy) {
                return None;
            }
            (0, 4096, false)
        };
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            let mut candidate = policy;
            if children {
                candidate.children = middle;
            } else {
                candidate.chars = middle;
            }
            if fits(candidate) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        if children {
            policy.children = low;
        } else {
            policy.chars = low;
        }
        self.take(&make(policy), 0).then_some(policy)
    }
}

fn size(value: &impl Serialize) -> Option<usize> {
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).ok()?;
    Some(counter.0)
}

struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_BYTES.saturating_sub(self.0) {
            return Err(io::Error::other(
                "JSON component exceeds the document bound",
            ));
        }
        self.0 += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode a pre-budgeted plan through a final hard bound; never publish partial JSON.
pub(super) fn encode(value: &impl Serialize) -> io::Result<Vec<u8>> {
    struct Output(Vec<u8>);
    impl Write for Output {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() >= MAX_BYTES.saturating_sub(self.0.len()) {
                return Err(io::Error::other("Failure report exceeds 8 MiB"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut output = Output(Vec::new());
    serde_json::to_writer(&mut output, value).map_err(io::Error::other)?;
    output.0.push(b'\n');
    Ok(output.0)
}
