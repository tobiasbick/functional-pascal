//! Loop-carried handle origins reach a stable state before analysis continues.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use super::walker::Walker;

impl Walker<'_, '_> {
    /// Repeat a loop's lexical walk until enclosing bindings gain no more origins.
    pub(super) fn walk_loop(&mut self, walk: impl Fn(&mut Self)) {
        loop {
            let before = self.binding_origin_count();
            self.in_frame(&walk);
            if before == self.binding_origin_count() {
                break;
            }
        }
    }

    fn binding_origin_count(&self) -> usize {
        self.frames
            .iter()
            .flat_map(|frame| frame.bindings.values())
            .map(|binding| binding.owners.len())
            .sum()
    }
}
