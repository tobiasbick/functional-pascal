//! Lexical debugger scopes and source sequence points.

use fpas_ir::{DebugScope, SequencePoint, SourceSpan};

use super::LoweringContext;

impl LoweringContext {
    pub(super) fn begin_debug_scope(&mut self) {
        let parent = self.debug_scope;
        let id = u32::try_from(self.debug.scopes.len()).unwrap_or(u32::MAX);
        self.debug.scopes.push(DebugScope {
            id,
            parent: Some(parent),
        });
        self.debug_scope_stack.push(parent);
        self.debug_scope = id;
    }

    pub(super) fn end_debug_scope(&mut self) {
        self.debug_scope = self.debug_scope_stack.pop().unwrap_or(0);
    }

    pub(super) fn record_sequence_point(&mut self, instruction: usize, source: SourceSpan) {
        self.debug.sequence_points.push(SequencePoint {
            block: self.current,
            instruction,
            source,
            scope: self.debug_scope,
        });
    }
}
