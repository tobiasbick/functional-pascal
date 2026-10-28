//! Callable values for scalar parsing, formatting, and integer-to-real conversion.
//! See `docs/pascal/std/text/parse.md`, `docs/pascal/std/text/format.md`, and
//! `docs/pascal/std/numeric/math.md`.

use fpas_code::{ConvIntrinsic, Intrinsic, ParseIntrinsic};
use fpas_ir::FunctionId;
use fpas_parser::{DesignatorPart, Expr};
use fpas_sema::{AnalysisMetadata, Ty};

use crate::CompileError;
use crate::lowering::context::{BoundMethodTarget, unsupported};
use crate::lowering::types;

use super::{ClosureRegistry, IntrinsicRoutine};

impl ClosureRegistry<'_> {
    /// Registers a checked conversion reference using its imported identity and callable type.
    pub(super) fn register_conversion_value(
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
        let Some((unit, _)) = metadata
            .import_aliases
            .iter()
            .find(|(_, written)| written.eq_ignore_ascii_case(alias))
        else {
            return Ok(());
        };
        let Some(intrinsic) = conversion_intrinsic(unit, member) else {
            return Ok(());
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
            name: format!("$conversion_value_{}", id.get()),
            intrinsic: Some(intrinsic),
            native: None,
            parameters,
            result,
            span,
            owner,
        });
        Ok(())
    }
}

fn conversion_intrinsic(unit: &str, member: &str) -> Option<Intrinsic> {
    let qualified = format!("{unit}.{member}");
    [
        Intrinsic::Parse(ParseIntrinsic::Int),
        Intrinsic::Parse(ParseIntrinsic::Real),
        Intrinsic::Parse(ParseIntrinsic::Bool),
        Intrinsic::Parse(ParseIntrinsic::HexInt),
        Intrinsic::Conv(ConvIntrinsic::IntToStr),
        Intrinsic::Conv(ConvIntrinsic::RealToStr),
        Intrinsic::Conv(ConvIntrinsic::BoolToStr),
        Intrinsic::Conv(ConvIntrinsic::IntToHex),
        Intrinsic::Conv(ConvIntrinsic::IntToReal),
    ]
    .into_iter()
    .find(|intrinsic| intrinsic.debugger_name().eq_ignore_ascii_case(&qualified))
}
