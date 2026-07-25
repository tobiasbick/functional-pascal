//! Lowering for property getter reads (`B.Text`).
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, DesignatorPart};
use fpas_sema::PropertyReadInfo;

use super::super::Compiler;

impl Compiler {
    /// Emit a property getter call with the receiver already on the stack.
    ///
    /// Stack effect: `[..., receiver]` → `[..., result]`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(in crate::compiler) fn emit_property_get_from_receiver(
        &mut self,
        info: &PropertyReadInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let getter = self.qualify_name(&info.getter_name).to_string();
        let name_idx = self.add_constant(Value::Str(getter.into()), location)?;
        self.emit(Op::Call(name_idx, 1), location);
        Ok(())
    }

    /// Compile a designator that sema resolved as a property read.
    pub(in crate::compiler) fn compile_property_read_designator(
        &mut self,
        d: &Designator,
        infos: &[PropertyReadInfo],
    ) -> Result<(), CompileError> {
        self.compile_property_read_designator_prefix(d, d.parts.len(), infos)
    }

    fn compile_property_read_designator_prefix(
        &mut self,
        d: &Designator,
        part_count: usize,
        infos: &[PropertyReadInfo],
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&d.span);
        let parts = &d.parts[..part_count.min(d.parts.len())];
        let mut ordered = infos.to_vec();
        ordered.sort_by_key(|info| info.receiver_part_count);
        let Some(first) = ordered.first() else {
            return Err(internal_compiler_error(
                "Property-read metadata has no getter entry.",
                "Re-run compilation and report this internal compiler error.",
                d.span.line,
                d.span.column,
            ));
        };
        parts.get(..first.receiver_part_count).ok_or_else(|| {
            internal_compiler_error(
                "Property-read receiver metadata exceeds the designator path.",
                "Re-run compilation and report this internal compiler error.",
                d.span.line,
                d.span.column,
            )
        })?;
        self.compile_designator_prefix_read(d, first.receiver_part_count)?;
        self.emit_property_get_from_receiver(first, location)?;

        let mut cursor = first.receiver_part_count + 1;
        for info in ordered.iter().skip(1) {
            let suffix = parts.get(cursor..info.receiver_part_count).ok_or_else(|| {
                internal_compiler_error(
                    "Property-read metadata is not ordered within the designator path.",
                    "Re-run compilation and report this internal compiler error.",
                    d.span.line,
                    d.span.column,
                )
            })?;
            self.compile_property_receiver_suffix(suffix, location)?;
            self.emit_property_get_from_receiver(info, location)?;
            cursor = info.receiver_part_count + 1;
        }

        let suffix = parts.get(cursor..).ok_or_else(|| {
            internal_compiler_error(
                "Property-read metadata exceeds the designator path.",
                "Re-run compilation and report this internal compiler error.",
                d.span.line,
                d.span.column,
            )
        })?;
        self.compile_property_receiver_suffix(suffix, location)
    }

    /// Compile a receiver prefix that may contain property getter reads.
    pub(in crate::compiler) fn compile_property_receiver_prefix(
        &mut self,
        designator: &Designator,
        part_count: usize,
        reads: &[PropertyReadInfo],
    ) -> Result<(), CompileError> {
        if reads.is_empty() {
            self.compile_designator_prefix_read(designator, part_count)
        } else {
            self.compile_property_read_designator_prefix(designator, part_count, reads)
        }
    }

    fn compile_property_receiver_suffix(
        &mut self,
        parts: &[DesignatorPart],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        for part in parts {
            match part {
                DesignatorPart::Ident(field, _) => {
                    let idx = self.add_constant(Value::Str(field.clone().into()), location)?;
                    self.emit(Op::FieldGet(idx), location);
                }
                DesignatorPart::Index(expr, _) => {
                    self.compile_expr(expr)?;
                    self.emit(Op::IndexGet, location);
                }
            }
        }
        Ok(())
    }
}
