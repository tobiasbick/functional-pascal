use super::super::NameRewriter;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl NameRewriter<'_> {
    pub(in super::super) fn rewrite_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Integer(_, _)
            | Expr::Real(_, _)
            | Expr::Str(_, _)
            | Expr::Bool(_, _)
            | Expr::Error(_) => {}
            Expr::Designator(designator) => self.rewrite_designator(designator),
            Expr::Call {
                designator, args, ..
            } => {
                self.rewrite_designator(designator);
                for arg in args {
                    self.rewrite_expr(arg);
                }
            }
            Expr::UnaryOp { operand, .. } => self.rewrite_expr(operand),
            Expr::BinaryOp { left, right, .. } => {
                self.rewrite_expr(left);
                self.rewrite_expr(right);
            }
            Expr::Paren(inner, _) => self.rewrite_expr(inner),
            Expr::ArrayLiteral(values, _) => {
                for value in values {
                    self.rewrite_expr(value);
                }
            }
            Expr::DictLiteral(pairs, _) => {
                for (key, value) in pairs {
                    self.rewrite_expr(key);
                    self.rewrite_expr(value);
                }
            }
            Expr::RecordLiteral { fields, .. } => {
                for field in fields {
                    self.rewrite_expr(&mut field.value);
                }
            }
            Expr::New {
                type_expr, fields, ..
            } => {
                self.rewrite_type_expr(type_expr);
                for field in fields {
                    self.rewrite_expr(&mut field.value);
                }
            }
            Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) | Expr::OptionSome(inner, _) => {
                self.rewrite_expr(inner);
            }
            Expr::Try(inner, _) | Expr::Go(inner, _) => self.rewrite_expr(inner),
            Expr::OptionNone(_) => {}
            Expr::RecordUpdate { base, fields, .. } => {
                self.rewrite_expr(base);
                for field in fields {
                    self.rewrite_expr(&mut field.value);
                }
            }
        }
    }

    pub(super) fn rewrite_case_pattern_expr(&mut self, expr: &mut Expr, allow_binding_name: bool) {
        match expr {
            Expr::Integer(_, _)
            | Expr::Real(_, _)
            | Expr::Str(_, _)
            | Expr::Bool(_, _)
            | Expr::Error(_) => {}
            Expr::Designator(designator) => {
                if allow_binding_name && Self::is_pattern_binding_designator(designator) {
                    return;
                }
                self.rewrite_designator(designator);
            }
            Expr::Call {
                designator, args, ..
            } => {
                self.rewrite_designator(designator);
                for arg in args {
                    self.rewrite_case_pattern_expr(arg, true);
                }
            }
            Expr::UnaryOp { operand, .. } => {
                self.rewrite_case_pattern_expr(operand, allow_binding_name);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.rewrite_case_pattern_expr(left, allow_binding_name);
                self.rewrite_case_pattern_expr(right, allow_binding_name);
            }
            Expr::Paren(inner, _) => self.rewrite_case_pattern_expr(inner, allow_binding_name),
            Expr::ArrayLiteral(values, _) => {
                for value in values {
                    self.rewrite_case_pattern_expr(value, allow_binding_name);
                }
            }
            Expr::DictLiteral(pairs, _) => {
                for (key, value) in pairs {
                    self.rewrite_case_pattern_expr(key, allow_binding_name);
                    self.rewrite_case_pattern_expr(value, allow_binding_name);
                }
            }
            Expr::RecordLiteral { fields, .. } => {
                for field in fields {
                    self.rewrite_case_pattern_expr(&mut field.value, allow_binding_name);
                }
            }
            Expr::New {
                type_expr, fields, ..
            } => {
                self.rewrite_type_expr(type_expr);
                for field in fields {
                    self.rewrite_case_pattern_expr(&mut field.value, allow_binding_name);
                }
            }
            Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) | Expr::OptionSome(inner, _) => {
                self.rewrite_case_pattern_expr(inner, allow_binding_name);
            }
            Expr::Try(inner, _) | Expr::Go(inner, _) => {
                self.rewrite_case_pattern_expr(inner, allow_binding_name);
            }
            Expr::OptionNone(_) => {}
            Expr::RecordUpdate { base, fields, .. } => {
                self.rewrite_case_pattern_expr(base, allow_binding_name);
                for field in fields {
                    self.rewrite_case_pattern_expr(&mut field.value, allow_binding_name);
                }
            }
        }
    }

    pub(in super::super) fn rewrite_designator(&mut self, designator: &mut Designator) {
        for part in &mut designator.parts {
            if let DesignatorPart::Index(expr, _) = part {
                self.rewrite_expr(expr);
            }
        }

        let Some(DesignatorPart::Ident(first_name, first_span)) = designator.parts.first() else {
            return;
        };

        if self.is_local_value(first_name) {
            return;
        }

        let Some(qualified) =
            self.resolve_import_name(first_name, first_span.line, first_span.column)
        else {
            self.normalize_qualified_unit_prefix(designator);
            return;
        };

        let mut rewritten = qualified
            .split('.')
            .map(|segment| DesignatorPart::Ident(segment.to_string(), *first_span))
            .collect::<Vec<_>>();
        rewritten.extend(designator.parts.iter().skip(1).cloned());
        designator.parts = rewritten;
    }

    /// When a user writes a qualified designator like `app.lib.GetValue`, the unit
    /// prefix may use non-canonical casing. Normalize the prefix segments to their
    /// canonical form so the sema and compiler see the correct names.
    fn normalize_qualified_unit_prefix(&self, designator: &mut Designator) {
        if designator.parts.len() < 2 {
            return;
        }

        let ident_count = designator
            .parts
            .iter()
            .take_while(|p| matches!(p, DesignatorPart::Ident(_, _)))
            .count();

        if ident_count < 2 {
            return;
        }

        let first_span = match &designator.parts[0] {
            DesignatorPart::Ident(_, span) => *span,
            _ => return,
        };

        for prefix_len in (1..ident_count).rev() {
            let candidate: String = designator.parts[..prefix_len]
                .iter()
                .filter_map(|p| match p {
                    DesignatorPart::Ident(name, _) => Some(name.to_ascii_lowercase()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(".");

            if let Some(canonical_segments) = self.canonical_units.get(&candidate) {
                let mut new_parts: Vec<DesignatorPart> = canonical_segments
                    .iter()
                    .map(|s| DesignatorPart::Ident(s.clone(), first_span))
                    .collect();
                new_parts.extend(designator.parts.iter().skip(prefix_len).cloned());
                designator.parts = new_parts;
                return;
            }
        }
    }
}
