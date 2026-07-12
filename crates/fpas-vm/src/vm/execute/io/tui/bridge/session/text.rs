//! Memo and text viewer session state.

use super::*;

impl TurboVisionSession {
    pub fn insert_detached_memo(
        &mut self,
        memo: Box<dyn View>,
        local_bounds: Rect,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::Memo);
        self.memo_texts.insert(handle, text);
        self.detached_memos
            .insert(handle, DetachedMemo { memo, local_bounds });
        handle
    }

    /// Replaces a detached memo view after `Memo.SetText`.
    pub fn replace_detached_memo(&mut self, handle: u32, memo: Box<dyn View>) {
        if let Some(detached) = self.detached_memos.get_mut(&handle) {
            detached.memo = memo;
        }
    }

    /// Removes a detached memo for parent attach.
    pub fn take_detached_memo(&mut self, handle: u32) -> Option<DetachedMemo> {
        self.detached_memos.remove(&handle)
    }

    /// Returns host-side memo text.
    #[must_use]
    /// Read host-side memo text (unit tests).
    #[cfg(test)]
    pub fn memo_text(&self, handle: u32) -> Option<&str> {
        self.memo_texts.get(&handle).map(String::as_str)
    }

    /// Updates host-side memo text.
    pub fn set_memo_text(&mut self, handle: u32, text: String) {
        self.memo_texts.insert(handle, text);
    }

    /// Returns detached memo bounds when still awaiting attach.
    #[must_use]
    pub fn detached_memo_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_memos
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }

    /// Inserts a detached text viewer and returns its FPAS handle.
    pub fn insert_detached_text_viewer(
        &mut self,
        text_viewer: Box<dyn View>,
        local_bounds: Rect,
        text: String,
    ) -> u32 {
        let handle = self.registry.allocate(0, ViewKind::TextViewer);
        self.text_viewer_texts.insert(handle, text);
        self.detached_text_viewers.insert(
            handle,
            DetachedTextViewer {
                text_viewer,
                local_bounds,
            },
        );
        handle
    }

    /// Replaces a detached text viewer after `TextViewer.SetText`.
    pub fn replace_detached_text_viewer(&mut self, handle: u32, text_viewer: Box<dyn View>) {
        if let Some(detached) = self.detached_text_viewers.get_mut(&handle) {
            detached.text_viewer = text_viewer;
        }
    }

    /// Removes a detached text viewer for parent attach.
    pub fn take_detached_text_viewer(&mut self, handle: u32) -> Option<DetachedTextViewer> {
        self.detached_text_viewers.remove(&handle)
    }

    /// Returns host-side text viewer text.
    #[must_use]
    /// Read host-side text viewer text (unit tests).
    #[cfg(test)]
    pub fn text_viewer_text(&self, handle: u32) -> Option<&str> {
        self.text_viewer_texts.get(&handle).map(String::as_str)
    }

    /// Updates host-side text viewer text.
    pub fn set_text_viewer_text(&mut self, handle: u32, text: String) {
        self.text_viewer_texts.insert(handle, text);
    }

    /// Returns detached text viewer bounds when still awaiting attach.
    #[must_use]
    pub fn detached_text_viewer_bounds(&self, handle: u32) -> Option<Rect> {
        self.detached_text_viewers
            .get(&handle)
            .map(|detached| detached.local_bounds)
    }
}
