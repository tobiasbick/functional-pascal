//! Bound unfinished frames before parsing or extending the retained byte buffer.

use crate::{RawJson, connection::Fault};

/// Incremental newline-delimited UTF-8 JSON decoder.
pub struct Framer {
    unfinished: Vec<u8>,
    maximum: usize,
    failure: Option<Fault>,
}

impl Framer {
    /// Construct a decoder with the complete-message byte ceiling.
    pub fn new(maximum: usize) -> Self {
        Self {
            unfinished: Vec::new(),
            maximum,
            failure: None,
        }
    }

    /// Decode complete messages while retaining only a bounded unfinished line.
    pub fn push(&mut self, mut bytes: &[u8]) -> Result<Vec<RawJson>, Fault> {
        if let Some(fault) = &self.failure {
            return Err(fault.clone());
        }
        let mut frames = Vec::new();
        while !bytes.is_empty() {
            let newline = bytes.iter().position(|byte| *byte == b'\n');
            let count = newline.unwrap_or(bytes.len());
            match self.append(&bytes[..count], newline.is_some()) {
                Ok(Some(frame)) => frames.push(frame),
                Ok(None) => {}
                Err(fault) => {
                    self.failure = Some(fault.clone());
                    return if frames.is_empty() {
                        Err(fault)
                    } else {
                        Ok(frames)
                    };
                }
            }
            let Some(_) = newline else {
                break;
            };
            bytes = &bytes[count + 1..];
        }
        Ok(frames)
    }

    /// EOF is clean only at a frame boundary.
    pub fn finish(&self) -> Result<(), Fault> {
        if let Some(fault) = &self.failure {
            Err(fault.clone())
        } else if self.unfinished.is_empty() {
            Ok(())
        } else {
            Err(Fault::InvalidMessage)
        }
    }

    /// A later invalid line does not erase complete frames already returned by push.
    pub fn failure(&self) -> Option<&Fault> {
        self.failure.as_ref()
    }

    fn append(&mut self, bytes: &[u8], complete: bool) -> Result<Option<RawJson>, Fault> {
        if bytes.len() > self.maximum.saturating_sub(self.unfinished.len()) {
            return Err(Fault::LimitExceeded("MessageBytes", self.maximum));
        }
        self.unfinished.extend_from_slice(bytes);
        if !complete {
            return Ok(None);
        }
        let text = std::str::from_utf8(&self.unfinished).map_err(|_| Fault::InvalidMessage)?;
        let frame = RawJson::parse_bounded(text, self.maximum)?;
        self.unfinished.clear();
        Ok(Some(frame))
    }
}
