use fpas_lexer::Span;
use fpas_parser::{CaseLabel, DesignatorPart, Expr};

/// A variant call found in a pattern, used for field-count validation.
pub(super) struct PatternCall {
    pub(super) variant_name: String,
    pub(super) arg_count: usize,
    pub(super) span: Span,
}

/// The normalized form of a single-level data-enum case pattern.
///
/// Supports only one-level destructuring: `Shape.Circle(R)` or `Shape.Point`.
/// Each field position must be a plain identifier binding.
///
/// **Documentation:** `docs/pascal/06-pattern-matching.md`
pub(super) struct DataEnumPattern {
    /// Variant name at the root of the pattern (e.g. `"Circle"`).
    pub(super) root_variant_name: Option<String>,
    /// Flat list of `(field_index, binding_name)` for identifier bindings.
    pub(super) bindings: Vec<(u8, String)>,
    /// The variant call for field-count validation, if present.
    pub(super) variant_call: Option<PatternCall>,
}

impl DataEnumPattern {
    /// Analyze a [`CaseLabel`] and extract the root variant name and bindings.
    pub(super) fn analyze(label: &CaseLabel) -> Self {
        let CaseLabel::Value {
            start, end: None, ..
        } = label
        else {
            return Self::empty();
        };

        match start {
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let root_variant_name = variant_name_from_designator(designator);
                let variant_call = root_variant_name.as_ref().map(|name| PatternCall {
                    variant_name: name.clone(),
                    arg_count: args.len(),
                    span: *span,
                });
                let bindings = collect_bindings(args);
                Self {
                    root_variant_name,
                    bindings,
                    variant_call,
                }
            }
            Expr::Designator(designator) => {
                let root_variant_name = variant_name_from_designator(designator);
                Self {
                    root_variant_name,
                    bindings: Vec::new(),
                    variant_call: None,
                }
            }
            _ => Self::empty(),
        }
    }

    fn empty() -> Self {
        Self {
            root_variant_name: None,
            bindings: Vec::new(),
            variant_call: None,
        }
    }
}

/// Collect `(field_index, binding_name)` pairs from an argument list.
///
/// Only plain identifier args (not `_`) become bindings. Any other expression
/// is skipped — sema has already rejected such args before the compiler runs.
fn collect_bindings(args: &[Expr]) -> Vec<(u8, String)> {
    args.iter()
        .enumerate()
        .filter_map(|(i, arg)| {
            if let Expr::Designator(d) = arg
                && d.parts.len() == 1
                && let DesignatorPart::Ident(name, _) = &d.parts[0]
                && name != "_"
            {
                Some((i as u8, name.clone()))
            } else {
                None
            }
        })
        .collect()
}

fn variant_name_from_designator(designator: &fpas_parser::Designator) -> Option<String> {
    designator.parts.last().and_then(|part| match part {
        DesignatorPart::Ident(name, _) => Some(name.clone()),
        _ => None,
    })
}
