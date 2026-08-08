use fpas_ir::ValueId;
use fpas_lexer::Span;
use fpas_parser::DesignatorPart;

use crate::CompileError;

use super::LoweringContext;

impl LoweringContext {
    /// Lowers an index-only global write to the direct path operation when its encoding fits.
    pub(super) fn lower_global_index_path_write(
        &mut self,
        name: &str,
        parts: &[DesignatorPart],
        replacement: ValueId,
        span: Span,
    ) -> Result<bool, CompileError> {
        if parts.is_empty()
            || !self.global_index_path_uses_u16_slot(name)
            || parts
                .iter()
                .any(|part| !matches!(part, DesignatorPart::Index(_, _)))
            || u8::try_from(parts.len()).is_err()
        {
            return Ok(false);
        }

        let root = self.read_global(name, span)?;
        let mut indexes = Vec::with_capacity(parts.len());
        for part in parts {
            let DesignatorPart::Index(index, _) = part else {
                return Ok(false);
            };
            indexes.push(self.lower_expression(index)?);
        }
        self.write_global_index_path(name, root, indexes, replacement, span)?;
        Ok(true)
    }
}
