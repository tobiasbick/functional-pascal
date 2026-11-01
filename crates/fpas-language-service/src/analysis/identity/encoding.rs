//! Length-framed SHA-256 shared by build identity and snapshot identity.

use ring::digest::{Context, SHA256};

pub(crate) struct CanonicalHasher(Context);

impl CanonicalHasher {
    pub(crate) fn new(domain: &str) -> Self {
        let mut result = Self(Context::new(&SHA256));
        result.field(domain.as_bytes());
        result
    }

    pub(crate) fn field(&mut self, bytes: &[u8]) {
        self.0.update(&(bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }

    pub(crate) fn text(&mut self, text: &str) {
        self.field(text.as_bytes());
    }
    pub(crate) fn count(&mut self, count: usize) {
        self.text(&count.to_string());
    }

    pub(crate) fn finish(self) -> String {
        self.0
            .finish()
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
