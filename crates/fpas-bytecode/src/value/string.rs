//! Shared string value storage with cached character counts.

mod char_index;

use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use super::managed_heap::{managed_string_buffer, recycle_string};

/// UTF-8 payload plus a cached Unicode scalar count for O(1) [`SharedStr::char_len`].
#[derive(Debug)]
struct StrBody {
    data: String,
    char_len: usize,
    // Built on the first character lookup into a long non-ASCII string.
    char_offsets: OnceLock<Box<[usize]>>,
}

impl Clone for StrBody {
    fn clone(&self) -> Self {
        let mut data = managed_string_buffer(self.data.len());
        data.push_str(&self.data);
        Self {
            data,
            char_len: self.char_len,
            char_offsets: OnceLock::new(),
        }
    }
}

impl Drop for StrBody {
    fn drop(&mut self) {
        recycle_string(&mut self.data);
    }
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
        let mut data = managed_string_buffer(left.len() + right.len());
        data.push_str(left);
        data.push_str(right);
        Self::from_parts(data, left.char_len() + right.char_len())
    }

    /// Append `other`, reusing this buffer in place when no other value shares it.
    pub fn append(&mut self, other: &Self) {
        if let Some(body) = Arc::get_mut(&mut self.0) {
            body.data.push_str(other);
            body.char_len += other.char_len();
            body.char_offsets = OnceLock::new();
        } else {
            *self = Self::concat(self, other);
        }
    }

    /// Byte offset of Unicode scalar `index`; `index == char_len()` yields the byte length.
    ///
    /// ASCII strings answer in O(1); longer non-ASCII strings use a sampled offset index that is
    /// built once per string, so a lookup walks fewer than 64 scalars.
    pub fn byte_offset(&self, index: usize) -> Option<usize> {
        let body = &*self.0;
        if index >= body.char_len {
            return (index == body.char_len).then_some(body.data.len());
        }
        if body.char_len == body.data.len() {
            return Some(index);
        }
        if body.char_len <= char_index::STRIDE {
            return body
                .data
                .char_indices()
                .nth(index)
                .map(|(offset, _)| offset);
        }
        let samples = body
            .char_offsets
            .get_or_init(|| char_index::sample(&body.data));
        char_index::byte_offset(&body.data, samples, index)
    }

    /// Unicode scalar at `index`, or `None` when `index` is not below [`Self::char_len`].
    ///
    /// **Documentation:** `docs/pascal/std/text/str/format-chars.md` (CharAt).
    pub fn char_at(&self, index: usize) -> Option<char> {
        if index >= self.char_len() {
            return None;
        }
        self.0.data[self.byte_offset(index)?..].chars().next()
    }

    fn from_parts(data: String, char_len: usize) -> Self {
        Self(Arc::new(StrBody {
            data,
            char_len,
            char_offsets: OnceLock::new(),
        }))
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
        let mut data = managed_string_buffer(value.len());
        data.push_str(value);
        Self::from_parts(data, count_chars(value))
    }
}

impl From<SharedStr> for String {
    fn from(value: SharedStr) -> Self {
        match Arc::try_unwrap(value.0) {
            Ok(mut body) => std::mem::take(&mut body.data),
            Err(body) => {
                let mut data = managed_string_buffer(body.data.len());
                data.push_str(&body.data);
                data
            }
        }
    }
}

impl FromIterator<char> for SharedStr {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut data = managed_string_buffer(0);
        let mut char_len = 0;
        for character in iter {
            data.push(character);
            char_len += 1;
        }
        Self::from_parts(data, char_len)
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
    fn character_lookup_handles_ascii_short_and_sampled_unicode() {
        let ascii = SharedStr::from("hello");
        assert_eq!(ascii.char_at(1), Some('e'));
        assert_eq!(ascii.char_at(5), None);
        assert_eq!(ascii.byte_offset(5), Some(5));

        let short = SharedStr::from("aé日😀");
        assert_eq!(short.char_at(3), Some('😀'));
        assert_eq!(short.byte_offset(2), Some(3));

        let long: SharedStr = "aé日😀".repeat(40).into();
        for (index, character) in long.chars().enumerate() {
            assert_eq!(long.char_at(index), Some(character));
        }
        assert_eq!(long.char_at(long.char_len()), None);
        assert_eq!(long.byte_offset(long.char_len()), Some(long.len()));
    }

    #[test]
    fn append_reuses_unique_buffers_and_preserves_shared_ones() {
        let mut unique = SharedStr::from("aé");
        let before = Arc::as_ptr(&unique.0);
        unique.append(&SharedStr::from("日"));
        assert_eq!(Arc::as_ptr(&unique.0), before);
        assert_eq!(unique.as_ref(), "aé日");
        assert_eq!(unique.char_len(), 3);
        assert_eq!(unique.char_at(2), Some('日'));

        let mut appended = unique.clone();
        appended.append(&SharedStr::from("!"));
        assert_eq!(unique.as_ref(), "aé日");
        assert_eq!(appended.as_ref(), "aé日!");
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
