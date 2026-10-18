//! Bounded capture of worker-native pipes and acknowledged debug output records.

use serde_json::{Value, json};
use std::{
    io::Read,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub(super) struct Output {
    pub records: Vec<Value>,
    pub overflow: bool,
    bytes: usize,
    limit: usize,
}
impl Output {
    pub fn new(limit: usize) -> Self {
        Self {
            records: Vec::new(),
            overflow: false,
            bytes: 0,
            limit,
        }
    }
    pub fn push(&mut self, stream: &str, text: &str) {
        let available = self.limit.saturating_sub(self.bytes);
        let mut end = available.min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let truncated = end < text.len();
        self.overflow |= truncated;
        self.bytes += end;
        if end > 0 {
            self.records
                .push(json!({"stream":stream,"text":&text[..end],"truncated":truncated}));
        }
    }
}

pub(super) fn drain(
    mut input: impl Read + Send + 'static,
    stream: &'static str,
    captured: Arc<Mutex<Output>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut bytes = [0; 4096];
        let mut decoder = super::utf8::Utf8Output::default();
        loop {
            match input.read(&mut bytes) {
                Ok(0) | Err(_) => {
                    captured
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .push(stream, &decoder.decode(&[], true));
                    break;
                }
                Ok(count) => captured
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push(stream, &decoder.decode(&bytes[..count], false)),
            }
        }
    })
}
