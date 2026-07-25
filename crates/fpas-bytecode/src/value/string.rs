//! Shared string value storage with cached character counts.

use std::ops::Deref;
use std::sync::Arc;

/// UTF-8 payload plus a cached Unicode scalar count for O(1) [`SharedStr::char_len`].
#[derive(Debug, Clone)]
struct StrBody {
    data: String,
    char_len: usize,
}

impl PartialEq for StrBody {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for StrBody {}

impl PartialOrd for StrBody {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrBody {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl std::hash::Hash for StrBody {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

/// Shared immutable storage for FPAS string values.
///
/// Cloning a string shares its UTF-8 buffer and cached character length, avoiding a deep copy
/// until an owning consumer needs to mutate the string. [`SharedStr::char_len`] is O(1).
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SharedStr(Arc<StrBody>);

impl SharedStr {
    /// Unicode scalar count (`Std.Str.Length`), cached at construction and concat time.
    ///
    /// **Documentation:** `docs/pascal/std/text/str/case-trim.md` (Length); contributor map in
    /// `docs/pascal/std/text/str/README.md`.
    pub fn char_len(&self) -> usize {
        self.0.char_len
    }

    /// Concatenate two shared strings, summing cached character lengths.
    pub fn concat(left: &Self, right: &Self) -> Self {
        let mut data = String::with_capacity(left.len() + right.len());
        data.push_str(left);
        data.push_str(right);
        Self(Arc::new(StrBody {
            data,
            char_len: left.char_len() + right.char_len(),
        }))
    }

    fn from_parts(data: String, char_len: usize) -> Self {
        Self(Arc::new(StrBody { data, char_len }))
    }
}

fn count_chars(value: &str) -> usize {
    if value.is_ascii() {
        value.len()
    } else {
        value.chars().count()
    }
}

impl From<String> for SharedStr {
    fn from(value: String) -> Self {
        let char_len = count_chars(&value);
        Self::from_parts(value, char_len)
    }
}

impl From<&str> for SharedStr {
    fn from(value: &str) -> Self {
        Self::from(value.to_owned())
    }
}

impl From<SharedStr> for String {
    fn from(value: SharedStr) -> Self {
        Arc::unwrap_or_clone(value.0).data
    }
}

impl FromIterator<char> for SharedStr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let chars: Vec<char> = iter.into_iter().collect();
        let char_len = chars.len();
        Self::from_parts(chars.into_iter().collect(), char_len)
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.data.as_str()
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl std::fmt::Display for SharedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_shares_utf8_storage() {
        let original = SharedStr::from("hello");
        let cloned = original.clone();

        assert!(Arc::ptr_eq(&original.0, &cloned.0));
        assert_eq!(String::from(cloned), "hello");
    }

    #[test]
    fn char_len_handles_ascii_and_unicode() {
        assert_eq!(SharedStr::from("hello").char_len(), 5);
        assert_eq!(SharedStr::from("café").char_len(), 4);
    }

    #[test]
    fn concat_sums_cached_char_len() {
        let left = SharedStr::from("café");
        let right = SharedStr::from("!");
        let joined = SharedStr::concat(&left, &right);
        assert_eq!(joined.as_ref(), "café!");
        assert_eq!(joined.char_len(), 5);
    }
}
