use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Designator, DesignatorPart, Expr};

/// A variant call found in a pattern, tracking the variant name, number of
/// arguments supplied, and the source span for diagnostics.
pub(super) struct PatternCall {
    pub(super) variant_name: String,
    pub(super) arg_count: usize,
    pub(super) span: Span,
}

pub(super) struct DataEnumPattern<'a> {
    pub(super) root_variant_name: Option<String>,
    pub(super) nested_variant_checks: Vec<(Vec<u8>, String)>,
    pub(super) value_checks: Vec<(Vec<u8>, &'a Expr)>,
    pub(super) bindings: Vec<(Vec<u8>, String)>,
    /// Every variant destructuring call (including nested) with its arg count.
    pub(super) variant_calls: Vec<PatternCall>,
}

impl<'a> DataEnumPattern<'a> {
    pub(super) fn analyze(label: &'a CaseLabel) -> Self {
        let root_variant_name = extract_variant_name_from_label(label);
        let mut check_paths = Vec::new();
        let mut check_names = Vec::new();
        let mut value_checks = Vec::new();
        let mut bindings = Vec::new();
        let mut variant_calls = Vec::new();

        if let CaseLabel::Value {
            start, end: None, ..
        } = label
        {
            collect_pattern_info(
                start,
                &[],
                &mut check_paths,
                &mut check_names,
                &mut value_checks,
                &mut bindings,
                &mut variant_calls,
            );
        }

        let nested_variant_checks = check_paths
            .into_iter()
            .zip(check_names)
            .filter(|(field_path, _)| !field_path.is_empty())
            .collect();

        Self {
            root_variant_name,
            nested_variant_checks,
            value_checks,
            bindings,
            variant_calls,
        }
    }
}

fn collect_pattern_info<'a>(
    expr: &'a Expr,
    prefix: &[u8],
    check_paths: &mut Vec<Vec<u8>>,
    check_names: &mut Vec<String>,
    value_checks: &mut Vec<(Vec<u8>, &'a Expr)>,
    bindings: &mut Vec<(Vec<u8>, String)>,
    variant_calls: &mut Vec<PatternCall>,
) {
    match expr {
        Expr::Call {
            designator,
            args,
            span,
        } => {
            if let Some(variant_name) = extract_variant_name_from_designator(designator) {
                check_paths.push(prefix.to_vec());
                check_names.push(variant_name.clone());
                variant_calls.push(PatternCall {
                    variant_name,
                    arg_count: args.len(),
                    span: *span,
                });
            }
            for (index, arg) in args.iter().enumerate() {
                let mut child_path = prefix.to_vec();
                child_path.push(index as u8);
                match arg {
                    Expr::Designator(designator) if designator.parts.len() == 1 => {
                        if let DesignatorPart::Ident(name, _) = &designator.parts[0]
                            && name != "_"
                        {
                            bindings.push((child_path, name.clone()));
                        }
                    }
                    Expr::Call { .. } => {
                        collect_pattern_info(
                            arg,
                            &child_path,
                            check_paths,
                            check_names,
                            value_checks,
                            bindings,
                            variant_calls,
                        );
                    }
                    Expr::Designator(designator) if designator.parts.len() > 1 => {
                        if let Some(variant_name) = extract_variant_name_from_expr(arg) {
                            check_paths.push(child_path);
                            check_names.push(variant_name);
                        }
                    }
                    Expr::Integer(..) | Expr::Real(..) | Expr::Str(..) | Expr::Bool(..) => {
                        value_checks.push((child_path, arg));
                    }
                    _ => {}
                }
            }
        }
        Expr::Designator(designator) => {
            if designator.parts.len() > 1
                && let Some(variant_name) = extract_variant_name_from_expr(expr)
            {
                check_paths.push(prefix.to_vec());
                check_names.push(variant_name);
            }
        }
        _ => {}
    }
}

fn extract_variant_name_from_label(label: &CaseLabel) -> Option<String> {
    match label {
        CaseLabel::Value {
            start, end: None, ..
        } => extract_variant_name_from_expr(start),
        _ => None,
    }
}

fn extract_variant_name_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Call { designator, .. } => extract_variant_name_from_designator(designator),
        Expr::Designator(designator) => extract_variant_name_from_designator(designator),
        _ => None,
    }
}

fn extract_variant_name_from_designator(designator: &Designator) -> Option<String> {
    designator.parts.last().and_then(|part| match part {
        DesignatorPart::Ident(name, _) => Some(name.clone()),
        _ => None,
    })
}
