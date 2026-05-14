use super::{ViewId, ViewRegistry};

impl ViewRegistry {
    /// Append `id` to the focus chain if it is not already present.
    pub fn push_child(&mut self, id: ViewId) {
        if !self.children.contains(&id) {
            self.children.push(id);
        }
    }

    /// Remove `id` from the focus chain.
    pub fn remove_child(&mut self, id: ViewId) {
        self.remove_from_focus_chain(id);
    }

    /// Return the currently focused child view id, if one exists.
    #[must_use]
    pub fn focused_id(&self) -> Option<ViewId> {
        self.focused.map(|index| self.children[index])
    }

    /// Return `true` when the focus chain has at least one entry.
    #[must_use]
    pub fn has_focusable_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Focus the first child that belongs to `scope`.
    ///
    /// Returns `(changed, had_previous)`.
    pub fn focus_first_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        let Some(target) = self.children.iter().position(|id| scope.contains(id)) else {
            return (false, false);
        };
        if self.focused == Some(target) {
            return (false, false);
        }

        let had_previous = self.focused.is_some();
        self.focused = Some(target);
        (true, had_previous)
    }

    /// Advance focus forward through the full focus chain.
    pub fn focus_next(&mut self) -> (bool, bool) {
        self.focus_step(1)
    }

    /// Advance focus forward within `scope`.
    pub fn focus_next_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, true)
    }

    /// Retreat focus backward through the full focus chain.
    pub fn focus_prev(&mut self) -> (bool, bool) {
        self.focus_step(self.children.len().saturating_sub(1))
    }

    /// Retreat focus backward within `scope`.
    pub fn focus_prev_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, false)
    }

    fn focus_step(&mut self, step: usize) -> (bool, bool) {
        match self.children.len() {
            0 => (false, false),
            1 => {
                if self.focused.is_none() {
                    self.focused = Some(0);
                    (true, false)
                } else {
                    (false, false)
                }
            }
            len => {
                let had_previous = self.focused.is_some();
                let new_index = self.focused.map_or(0, |index| (index + step) % len);
                self.focused = Some(new_index);
                (true, had_previous)
            }
        }
    }

    fn focus_step_in_scope(&mut self, scope: &[ViewId], forward: bool) -> (bool, bool) {
        let scoped_indices: Vec<usize> = self
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, id)| scope.contains(id).then_some(index))
            .collect();

        match scoped_indices.len() {
            0 => (false, false),
            1 => {
                let target = scoped_indices[0];
                if self.focused == Some(target) {
                    (false, false)
                } else {
                    let had_previous = self.focused.is_some();
                    self.focused = Some(target);
                    (true, had_previous)
                }
            }
            len => {
                let had_previous = self.focused.is_some();
                let target = match self
                    .focused
                    .and_then(|focused| scoped_indices.iter().position(|&index| index == focused))
                {
                    Some(position) => {
                        if forward {
                            scoped_indices[(position + 1) % len]
                        } else {
                            scoped_indices[(position + len - 1) % len]
                        }
                    }
                    None => {
                        if forward {
                            scoped_indices[0]
                        } else {
                            scoped_indices[len - 1]
                        }
                    }
                };

                if self.focused == Some(target) {
                    (false, false)
                } else {
                    self.focused = Some(target);
                    (true, had_previous)
                }
            }
        }
    }

    pub(super) fn remove_from_focus_chain(&mut self, id: ViewId) {
        let Some(position) = self.children.iter().position(|view_id| *view_id == id) else {
            return;
        };

        self.children.remove(position);
        self.focused = match self.focused {
            None => None,
            Some(index) if index > position => Some(index - 1),
            Some(index) if index == position => {
                if self.children.is_empty() {
                    None
                } else {
                    Some(index.saturating_sub(1))
                }
            }
            Some(index) => Some(index),
        };
    }
}