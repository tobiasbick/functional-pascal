use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, Expr};
use fpas_sema::PropertyReadInfo;

use super::super::Compiler;

impl Compiler {
    pub(in super::super) fn compile_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let original_name = name;
        if let Some(local_ref) = self.resolve_local(name) {
            for arg in args {
                self.compile_expr(arg)?;
            }
            self.emit_local_ref_read(local_ref, location);
            let arity = Self::checked_u8_at(args.len(), "call arguments", location)?;
            self.emit(Op::CallValue(arity), location);
            return Ok(());
        }

        let qualified_storage = self
            .short_aliases
            .get(&super::super::canonical_name(name))
            .cloned();
        let name = qualified_storage.as_deref().unwrap_or(name);

        if self.compile_std_library_call(name, args, location)? {
            return Ok(());
        }

        if let Some((type_name, variant_info)) = self.find_enum_variant_with_data(name) {
            for arg in args {
                self.compile_expr(arg)?;
            }
            let type_idx = self.add_constant(Value::Str(type_name), location)?;
            let variant_idx = self.add_constant(Value::Str(variant_info.name), location)?;
            self.emit(
                Op::MakeEnum(
                    type_idx,
                    variant_idx,
                    Self::checked_u8_at(args.len(), "enum variant fields", location)?,
                ),
                location,
            );
            return Ok(());
        }

        for arg in args {
            self.compile_expr(arg)?;
        }
        if self.emit_captured_routine_closure(original_name, location)? {
            let arity = Self::checked_u8_at(args.len(), "call arguments", location)?;
            self.emit(Op::CallValue(arity), location);
            return Ok(());
        }
        let name_idx = self.add_constant(Value::Str(name.into()), location)?;
        let arity = Self::checked_u8_at(args.len(), "call arguments", location)?;
        self.emit(Op::Call(name_idx, arity), location);
        Ok(())
    }

    pub(in super::super) fn compile_method_call(
        &mut self,
        designator: &Designator,
        qualified_method: &str,
        receiver_reads: &[PropertyReadInfo],
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let qualified_method = self.qualify_name(qualified_method).to_string();
        if self.compile_std_library_call(&qualified_method, args, location)? {
            return Ok(());
        }

        self.compile_property_receiver_prefix(
            designator,
            designator.parts.len() - 1,
            receiver_reads,
        )?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        let total_args = Self::checked_u8_at(args.len() + 1, "method call arguments", location)?;
        let name_idx = self.add_constant(Value::Str(qualified_method), location)?;
        self.emit(Op::Call(name_idx, total_args), location);
        Ok(())
    }
}
