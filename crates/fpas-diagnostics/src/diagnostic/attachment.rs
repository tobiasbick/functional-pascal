//! Non-rendered ownership shared by cloned diagnostics.

use std::{any::Any, fmt, sync::Arc};

#[derive(Clone, Default)]
/// Shared producer ownership, independent of diagnostic meaning.
pub(super) struct Attachment(Option<Arc<dyn Any + Send + Sync>>);

impl Attachment {
    /// Read producer context without changing its lifetime.
    pub(super) fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.0.as_deref()?.downcast_ref()
    }

    /// Replace the context retained by this diagnostic and its subsequent clones.
    pub(super) fn set<T: Any + Send + Sync>(&mut self, value: T) {
        self.0 = Some(Arc::new(value));
    }
}

impl fmt::Debug for Attachment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(if self.0.is_some() { "Attached" } else { "None" })
    }
}

// Attachments carry ownership, not diagnostic meaning or rendered content.
impl PartialEq for Attachment {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for Attachment {}
