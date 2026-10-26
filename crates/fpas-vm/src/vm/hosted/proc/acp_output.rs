//! Incremental lossy UTF-8 decoding and byte-bounded retained terminal suffixes.

#[derive(Default)]
pub(super) struct Decoder {
    pending: Vec<u8>,
}
impl Decoder {
    pub fn push(&mut self, bytes: &[u8], eof: bool) -> String {
        self.pending.extend_from_slice(bytes);
        let mut output = String::new();
        let mut consumed = 0;
        loop {
            match std::str::from_utf8(&self.pending[consumed..]) {
                Ok(text) => {
                    output.push_str(text);
                    consumed = self.pending.len();
                    break;
                }
                Err(error) => {
                    let end = consumed + error.valid_up_to();
                    if let Ok(text) = std::str::from_utf8(&self.pending[consumed..end]) {
                        output.push_str(text);
                    }
                    consumed = end;
                    if let Some(length) = error.error_len() {
                        output.push('\u{fffd}');
                        consumed += length;
                    } else {
                        if eof {
                            output.push('\u{fffd}');
                            consumed = self.pending.len();
                        }
                        break;
                    }
                }
            }
        }
        self.pending.drain(..consumed);
        output
    }
}
pub(super) struct Output {
    pub text: String,
    pub truncated: bool,
    maximum: usize,
}
impl Output {
    pub fn new(maximum: usize) -> Self {
        Self {
            text: String::new(),
            truncated: false,
            maximum,
        }
    }
    pub fn push(&mut self, text: &str) {
        self.text.push_str(text);
        if self.text.len() > self.maximum {
            let mut start = self.text.len() - self.maximum;
            while !self.text.is_char_boundary(start) {
                start += 1;
            }
            self.text.drain(..start);
            self.truncated = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, Output};
    #[test]
    fn fragmented_invalid_utf8_and_suffix_truncation_obey_the_byte_bound() {
        let mut decoder = Decoder::default();
        let mut output = Output::new(6);
        output.push(&decoder.push(&[b'a', 0xf0, 0x9f], false));
        assert_eq!(output.text, "a");
        output.push(&decoder.push(&[0x98, 0x80, b'Z'], false));
        assert_eq!(output.text, "a😀Z");
        output.push(&decoder.push(&[0xff, b'E', b'N', b'D'], false));
        assert_eq!(output.text, "�END");
        assert_eq!(output.text.len(), 6);
        assert!(output.truncated);
        assert_eq!(decoder.push(&[0xe2], false), "");
        assert_eq!(decoder.push(&[], true), "�");
        let mut discard = Output::new(0);
        discard.push("😀");
        assert_eq!(discard.text, "");
        assert!(discard.truncated);
    }
}
