//! Incremental UTF-8 decoding for debuggee terminal and native pipe chunks.

/// Retains incomplete scalars between reads; malformed bytes use replacement characters.
#[derive(Default)]
pub struct Utf8Output {
    pending: Vec<u8>,
}

impl Utf8Output {
    /// Decode complete characters, retaining at most three trailing bytes until EOF.
    pub fn decode(&mut self, bytes: &[u8], eof: bool) -> String {
        self.pending.extend_from_slice(bytes);
        let mut result = String::new();
        let mut consumed = 0;
        while consumed < self.pending.len() {
            match std::str::from_utf8(&self.pending[consumed..]) {
                Ok(text) => {
                    result.push_str(text);
                    consumed = self.pending.len();
                }
                Err(error) => {
                    let valid = error.valid_up_to();
                    result.push_str(
                        std::str::from_utf8(&self.pending[consumed..consumed + valid]).unwrap(),
                    );
                    consumed += valid;
                    match error.error_len() {
                        Some(length) => {
                            result.push('\u{fffd}');
                            consumed += length;
                        }
                        None if eof => {
                            result.push('\u{fffd}');
                            consumed = self.pending.len();
                        }
                        None => break,
                    }
                }
            }
        }
        self.pending.drain(..consumed);
        result
    }
}
