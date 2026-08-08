//! Deterministic scalar constants, strings, and sparse source runs.

use fpas_bytecode::{
    Constant, ConstantId, InstructionAddress, SourceId, SourceMap, SourceRun, StringId, StringTable,
};
use fpas_ir::SourceSpan;

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct MetadataBuilder {
    constants: Vec<Constant>,
    strings: Vec<String>,
    runs: Vec<SourceRun>,
    source_path: StringId,
    last_location: Option<(u32, u32)>,
}

impl MetadataBuilder {
    pub fn new(function_name: &str) -> Result<(Self, StringId), CompileError> {
        let mut builder = Self {
            constants: Vec::new(),
            strings: Vec::new(),
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
        if let Some(index) = self
            .constants
            .iter()
            .position(|existing| *existing == value)
        {
            return ConstantId::try_from_index(index)
                .map(Some)
                .map_err(|error| metadata_error(&error.to_string()));
        }
        let id = ConstantId::try_from_index(self.constants.len())
            .map_err(|error| metadata_error(&error.to_string()))?;
        self.constants.push(value);
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
        if let Some(index) = self.strings.iter().position(|existing| existing == value) {
            return StringId::try_from_index(index)
                .map_err(|error| metadata_error(&error.to_string()));
        }
        let id = StringId::try_from_index(self.strings.len())
            .map_err(|error| metadata_error(&error.to_string()))?;
        self.strings.push(value.to_string());
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
