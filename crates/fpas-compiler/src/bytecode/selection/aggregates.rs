//! Aggregate and dense-global instruction selection.

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::{Operation, ValueId};

use crate::CompileError;

use super::{Selector, abc, abx, narrow, selection_error};
use crate::bytecode::metadata::MetadataBuilder;

impl Selector<'_> {
    pub(super) fn select_aggregate(
        &self,
        operation: &Operation,
        result: Option<ValueId>,
        metadata: &mut MetadataBuilder,
    ) -> Result<Option<Vec<Instruction>>, CompileError> {
        let selected = match operation {
            Operation::LoadGlobal(global) => vec![abx(
                Opcode::LoadGlobal,
                self.result_register(result)?,
                global.get(),
            )?],
            Operation::StoreGlobal { global, value } => vec![abx(
                Opcode::StoreGlobal,
                self.allocation.value(*value)?.get(),
                global.get(),
            )?],
            Operation::MakeArray(values) => {
                let mut instructions = self.prepare_window(values)?;
                instructions.push(abc(
                    Opcode::MakeArray,
                    self.result_register(result)?,
                    self.allocation.call_window().get(),
                    narrow(values.len(), "array element count")?,
                )?);
                instructions
            }
            Operation::ArrayPush { local, value } => {
                let local = self.allocation.local(*local)?.get();
                vec![
                    abc(
                        Opcode::ArrayPush,
                        local,
                        local,
                        self.allocation.value(*value)?.get(),
                    )?,
                    abc(Opcode::LoadUnit, self.result_register(result)?, 0, 0)?,
                ]
            }
            Operation::MakeDictionary(pairs) => {
                let values = pairs
                    .iter()
                    .flat_map(|(key, value)| [*key, *value])
                    .collect::<Vec<_>>();
                let mut instructions = self.prepare_window(&values)?;
                instructions.push(abc(
                    Opcode::MakeDictionary,
                    self.result_register(result)?,
                    self.allocation.call_window().get(),
                    narrow(pairs.len(), "dictionary pair count")?,
                )?);
                instructions
            }
            Operation::IndexGet { collection, index } => vec![abc(
                Opcode::IndexGet,
                self.result_register(result)?,
                self.allocation.value(*collection)?.get(),
                self.allocation.value(*index)?.get(),
            )?],
            Operation::IndexSet {
                collection,
                index,
                value,
            } => {
                let destination = self.result_register(result)?;
                vec![
                    abc(
                        Opcode::Move,
                        destination,
                        self.allocation.value(*collection)?.get(),
                        0,
                    )?,
                    abc(
                        Opcode::IndexSet,
                        destination,
                        self.allocation.value(*index)?.get(),
                        self.allocation.value(*value)?.get(),
                    )?,
                ]
            }
            Operation::Contains { value, collection } => vec![abc(
                Opcode::Contains,
                self.result_register(result)?,
                self.allocation.value(*value)?.get(),
                self.allocation.value(*collection)?.get(),
            )?],
            Operation::MakeRecord { layout, fields } => {
                let mut instructions = self.prepare_window(fields)?;
                instructions.push(abc(
                    Opcode::MakeRecord,
                    self.result_register(result)?,
                    narrow(layout.get(), "record layout")?,
                    self.allocation.call_window().get(),
                )?);
                instructions
            }
            Operation::LoadField { record, field, .. } => vec![abc(
                Opcode::LoadField,
                self.result_register(result)?,
                self.allocation.value(*record)?.get(),
                narrow(field.get(), "record field")?,
            )?],
            Operation::StoreField {
                record,
                field,
                value,
                ..
            } => vec![abc(
                Opcode::StoreField,
                self.allocation.value(*record)?.get(),
                narrow(field.get(), "record field")?,
                self.allocation.value(*value)?.get(),
            )?],
            Operation::UpdateRecord { record, fields, .. } => {
                let destination = self.result_register(result)?;
                let mut instructions = vec![abc(
                    Opcode::Move,
                    destination,
                    self.allocation.value(*record)?.get(),
                    0,
                )?];
                instructions.extend(self.prepare_overrides(fields, metadata)?);
                instructions.push(abc(
                    Opcode::UpdateRecord,
                    destination,
                    self.allocation.call_window().get(),
                    narrow(fields.len(), "record override count")?,
                )?);
                instructions
            }
            Operation::MakeOk(value) => self.wrap(Opcode::MakeOk, *value, result)?,
            Operation::MakeError(value) => self.wrap(Opcode::MakeError, *value, result)?,
            Operation::MakeSome(value) => self.wrap(Opcode::MakeSome, *value, result)?,
            Operation::MakeNone => {
                vec![abc(Opcode::MakeNone, self.result_register(result)?, 0, 0)?]
            }
            Operation::IsResultOk(value) => self.wrap(Opcode::IsResultOk, *value, result)?,
            Operation::IsOptionSome(value) => self.wrap(Opcode::IsOptionSome, *value, result)?,
            Operation::UnwrapOk(value) => self.wrap(Opcode::UnwrapOk, *value, result)?,
            Operation::UnwrapError(value) => self.wrap(Opcode::UnwrapError, *value, result)?,
            Operation::UnwrapSome(value) => self.wrap(Opcode::UnwrapSome, *value, result)?,
            Operation::MakeEnum {
                layout,
                variant,
                fields,
            } => {
                let mut instructions = self.prepare_window(fields)?;
                instructions.push(abc(
                    Opcode::MakeEnum,
                    self.result_register(result)?,
                    self.enum_variant(*layout, *variant)?,
                    self.allocation.call_window().get(),
                )?);
                instructions
            }
            Operation::TestVariant {
                value,
                layout,
                variant,
            } => vec![abc(
                Opcode::TestVariant,
                self.result_register(result)?,
                self.allocation.value(*value)?.get(),
                self.enum_variant(*layout, *variant)?,
            )?],
            Operation::LoadEnumField { value, field, .. } => vec![abc(
                Opcode::LoadEnumField,
                self.result_register(result)?,
                self.allocation.value(*value)?.get(),
                narrow(field.get(), "enum field")?,
            )?],
            _ => return Ok(None),
        };
        Ok(Some(selected))
    }

    fn wrap(
        &self,
        opcode: Opcode,
        value: ValueId,
        result: Option<ValueId>,
    ) -> Result<Vec<Instruction>, CompileError> {
        Ok(vec![abc(
            opcode,
            self.result_register(result)?,
            self.allocation.value(value)?.get(),
            0,
        )?])
    }

    fn prepare_overrides(
        &self,
        fields: &[(fpas_ir::FieldId, ValueId)],
        metadata: &mut MetadataBuilder,
    ) -> Result<Vec<Instruction>, CompileError> {
        let base = self.allocation.call_window().get();
        let mut instructions = Vec::with_capacity(fields.len() * 2);
        for (index, (field, value)) in fields.iter().enumerate() {
            let offset = narrow(
                index
                    .checked_mul(2)
                    .ok_or_else(|| selection_error("record override window overflow"))?,
                "record override offset",
            )?;
            let field_register = base
                .checked_add(offset)
                .ok_or_else(|| selection_error("record override window exceeds u16"))?;
            let value_register = field_register
                .checked_add(1)
                .ok_or_else(|| selection_error("record override window exceeds u16"))?;
            let constant = metadata
                .constant(&fpas_ir::Constant::Integer(i64::from(field.get())))?
                .ok_or_else(|| selection_error("record field constant is missing"))?;
            instructions.push(abx(Opcode::LoadConstant, field_register, constant.get())?);
            instructions.push(abc(
                Opcode::Move,
                value_register,
                self.allocation.value(*value)?.get(),
                0,
            )?);
        }
        Ok(instructions)
    }

    fn enum_variant(
        &self,
        layout: fpas_ir::EnumLayoutId,
        variant: fpas_ir::VariantId,
    ) -> Result<u16, CompileError> {
        let preceding = self
            .program
            .enum_layouts
            .iter()
            .take_while(|item| item.id != layout)
            .map(|item| item.variants.len())
            .sum::<usize>();
        narrow(
            preceding
                .checked_add(
                    usize::try_from(variant.get())
                        .map_err(|_| selection_error("enum variant does not fit this host"))?,
                )
                .ok_or_else(|| selection_error("enum variant index overflow"))?,
            "enum variant",
        )
    }
}
