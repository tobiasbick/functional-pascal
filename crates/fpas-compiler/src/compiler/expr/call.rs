use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, Expr};

use super::super::Compiler;

impl Compiler {
    pub(in super::super) fn compile_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
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

        if let Some(local_ref) = self.resolve_local(name) {
            for arg in args {
                self.compile_expr(arg)?;
            }
            match local_ref {
                super::super::LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
                super::super::LocalRef::Enclosing(depth, slot) => {
                    self.emit(Op::GetEnclosing(depth, slot), location)
                }
            };
            let arity = Self::checked_u8_at(args.len(), "call arguments", location)?;
            self.emit(Op::CallValue(arity), location);
            return Ok(());
        }

        for arg in args {
            self.compile_expr(arg)?;
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
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let receiver = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };
        self.compile_designator_read(&receiver)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        let total_args = Self::checked_u8_at(args.len() + 1, "method call arguments", location)?;
        let name_idx = self.add_constant(Value::Str(qualified_method.into()), location)?;
        self.emit(Op::Call(name_idx, total_args), location);
        Ok(())
    }
}
