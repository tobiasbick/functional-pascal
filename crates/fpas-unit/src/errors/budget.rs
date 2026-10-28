//! Shared admission budget for generated source-less error metadata.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

/// Limits generated metadata across all concrete instances in one link operation.
#[derive(Debug)]
pub struct ErrorInstantiationBudget {
    nodes: usize,
    text_bytes: usize,
}

impl Default for ErrorInstantiationBudget {
    fn default() -> Self {
        Self {
            nodes: fpas_code::limits::MAX_ERROR_EXPANSION_NODES,
            text_bytes: fpas_code::limits::MAX_STRING_BYTES,
        }
    }
}

impl ErrorInstantiationBudget {
    pub(super) fn nodes(&mut self, count: usize) -> Result<(), &'static str> {
        self.nodes = self
            .nodes
            .checked_sub(count)
            .ok_or("error instantiation exceeds the shared metadata budget")?;
        Ok(())
    }

    pub(super) fn text(&mut self, text: &str) -> Result<(), &'static str> {
        self.text_bytes = self
            .text_bytes
            .checked_sub(text.len())
            .ok_or("error instantiation exceeds the shared metadata budget")?;
        Ok(())
    }
}
