//! Deterministic scalar constants, strings, and sparse source runs.

use std::collections::HashMap;

use fpas_bytecode::{
    Constant, ConstantId, InstructionAddress, SourceId, SourceMap, SourceRun, StringId, StringTable,
};
use fpas_ir::SourceSpan;

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct MetadataBuilder {
    constants: Vec<Constant>,
    constant_ids: HashMap<Constant, ConstantId>,
    strings: Vec<String>,
    string_ids: HashMap<String, StringId>,
    runs: Vec<SourceRun>,
    source_path: StringId,
    last_location: Option<(u32, u32)>,
}

impl MetadataBuilder {
    pub fn new(function_name: &str) -> Result<(Self, StringId), CompileError> {
        let mut builder = Self {
            constants: Vec::new(),
            constant_ids: HashMap::new(),
            strings: Vec::new(),
            string_ids: HashMap::new(),
            runs: Vec::new(),
            source_path: StringId::new(0),
            last_location: None,
        };
        let name = builder.intern_string(function_name)?;
        builder.source_path = builder.intern_string("<memory>")?;
        Ok((builder, name))
    }

    pub fn function_name(&mut self, function_name: &str) -> Result<StringId, CompileError> {
        self.intern_string(function_name)
    }

    pub fn begin_function(&mut self) {
        self.last_location = None;
    }

    pub fn constant(
        &mut self,
        value: &fpas_ir::Constant,
    ) -> Result<Option<ConstantId>, CompileError> {
        let value = match value {
            fpas_ir::Constant::Unit => return Ok(None),
            fpas_ir::Constant::Boolean(value) => Constant::Boolean(*value),
            fpas_ir::Constant::Integer(value) => Constant::Integer(*value),
            fpas_ir::Constant::Real(value) => Constant::Real(value.to_bits()),
            fpas_ir::Constant::String(value) => Constant::String(self.intern_string(value)?),
        };
        if let Some(id) = self.constant_ids.get(&value) {
            return Ok(Some(*id));
        }
        let id = ConstantId::try_from_index(self.constants.len())
            .map_err(|error| metadata_error(&error.to_string()))?;
        self.constants.push(value);
        self.constant_ids.insert(value, id);
        Ok(Some(id))
    }

    pub fn record_source(
        &mut self,
        address: usize,
        source: Option<SourceSpan>,
    ) -> Result<(), CompileError> {
        let location = source.map_or((1, 1), |span| (span.line(), span.column()));
        if self.last_location == Some(location) && address != 0 {
            return Ok(());
        }
        let instruction_start = InstructionAddress::try_from_index(address)
            .map_err(|error| metadata_error(&error.to_string()))?;
        self.runs.push(SourceRun {
            instruction_start,
            source: SourceId::new(0),
            line: location.0,
            column: location.1,
        });
        self.last_location = Some(location);
        Ok(())
    }

    pub fn finish(self) -> (Vec<Constant>, StringTable, SourceMap) {
        (
            self.constants,
            StringTable::new(self.strings),
            SourceMap {
                sources: vec![self.source_path],
                runs: self.runs,
            },
        )
    }

    pub fn intern_string(&mut self, value: &str) -> Result<StringId, CompileError> {
        if let Some(id) = self.string_ids.get(value) {
            return Ok(*id);
        }
        let id = StringId::try_from_index(self.strings.len())
            .map_err(|error| metadata_error(&error.to_string()))?;
        let value = value.to_string();
        self.strings.push(value.clone());
        self.string_ids.insert(value, id);
        Ok(id)
    }
}

fn metadata_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register metadata limit exceeded: {message}."),
        "Split the program into smaller functions or report this compiler invariant failure.",
        1,
        1,
    )
}

#[cfg(test)]
mod tests {
    use fpas_bytecode::Constant;
    use fpas_ir::Constant as IrConstant;

    use super::MetadataBuilder;
    use crate::CompileError;

    #[test]
    fn strings_reuse_ids_and_preserve_first_seen_order() -> Result<(), CompileError> {
        let (mut metadata, _) = MetadataBuilder::new("root")?;
        let alpha = metadata.intern_string("alpha")?;
        let beta = metadata.intern_string("beta")?;
        let repeated_alpha = metadata.intern_string("alpha")?;
        let (_, strings, _) = metadata.finish();

        assert_eq!(
            (
                alpha,
                beta,
                repeated_alpha,
                strings.iter().collect::<Vec<_>>()
            ),
            (
                fpas_bytecode::StringId::new(2),
                fpas_bytecode::StringId::new(3),
                fpas_bytecode::StringId::new(2),
                vec!["root", "<memory>", "alpha", "beta"]
            )
        );
        Ok(())
    }

    #[test]
    fn constants_reuse_ids_and_preserve_bit_exact_order() -> Result<(), CompileError> {
        let (mut metadata, _) = MetadataBuilder::new("root")?;
        let positive_zero = metadata.constant(&IrConstant::Real(0.0))?;
        let negative_zero = metadata.constant(&IrConstant::Real(-0.0))?;
        let repeated_positive_zero = metadata.constant(&IrConstant::Real(0.0))?;
        let (constants, _, _) = metadata.finish();

        assert_eq!(
            (
                positive_zero,
                negative_zero,
                repeated_positive_zero,
                constants
            ),
            (
                Some(fpas_bytecode::ConstantId::new(0)),
                Some(fpas_bytecode::ConstantId::new(1)),
                Some(fpas_bytecode::ConstantId::new(0)),
                vec![
                    Constant::Real(0.0_f64.to_bits()),
                    Constant::Real((-0.0_f64).to_bits())
                ]
            )
        );
        Ok(())
    }
}
