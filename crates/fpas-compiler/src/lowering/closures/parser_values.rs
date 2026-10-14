//! Callable values for the four public Result parsers.
//! See `docs/pascal/std/text/parse.md`.

use fpas_bytecode::{Intrinsic, ParseIntrinsic};
use fpas_ir::FunctionId;
use fpas_parser::{DesignatorPart, Expr};
use fpas_sema::{AnalysisMetadata, Ty};

use crate::CompileError;
use crate::lowering::context::{BoundMethodTarget, unsupported};
use crate::lowering::types;

use super::{ClosureRegistry, IntrinsicRoutine};

impl ClosureRegistry<'_> {
    /// Registers a checked parser reference using its imported identity and callable type.
    pub(super) fn register_parser_value(
        &mut self,
        expression: &Expr,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let Expr::Designator(designator) = expression else {
            return Ok(());
        };
        let [
            DesignatorPart::Ident(alias, _),
            DesignatorPart::Ident(member, _),
        ] = designator.parts.as_slice()
        else {
            return Ok(());
        };
        if !metadata.import_aliases.iter().any(|(unit, written)| {
            unit.eq_ignore_ascii_case("Std.Parse") && written.eq_ignore_ascii_case(alias)
        }) {
            return Ok(());
        }
        let parser = match member.to_ascii_lowercase().as_str() {
            "int" => ParseIntrinsic::Int,
            "real" => ParseIntrinsic::Real,
            "bool" => ParseIntrinsic::Bool,
            "hexint" => ParseIntrinsic::HexInt,
            _ => return Ok(()),
        };
        let key = fpas_sema::expr_lookup_key(expression);
        if self.intrinsic_targets.contains_key(&key) {
            return Ok(());
        }
        let Some(Ty::Function(signature)) = metadata.expr_types.get(&key) else {
            return Ok(());
        };
        let span = expression.span();
        let parameters = signature
            .params
            .iter()
            .map(|parameter| types.intern(&parameter.ty, span.line, span.column))
            .collect::<Result<Vec<_>, _>>()?;
        let result = types.intern(&signature.return_type, span.line, span.column)?;
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        self.intrinsic_targets.insert(
            key,
            BoundMethodTarget {
                function: id,
                value_type,
            },
        );
        self.intrinsic_routines.push(IntrinsicRoutine {
            id,
            name: format!("$parser_value_{}", id.get()),
            intrinsic: Some(Intrinsic::Parse(parser)),
            native: None,
            parameters,
            result,
            span,
            owner,
        });
        Ok(())
    }
}
