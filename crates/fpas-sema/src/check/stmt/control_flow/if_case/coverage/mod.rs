//! Pattern-matrix coverage for exhaustiveness and unreachable labels.
//!
//! Patterns are reduced to constructors of finite types (enum variants,
//! `Ok`/`Error`, `Some`/`None`, `true`/`false`) and wildcards. A literal of an
//! open type never completes coverage on its own.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::Checker;
use crate::types::Ty;
use fpas_unit::interface::ConstantValue;

mod constructors;
mod intervals;
mod rows;

use constructors::Constructor;
use rows::{concat, default_rows, render, specialize, value_rows};

/// Coverage view of one checked pattern.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Pat {
    /// Matches every value: `_`, `const Name`, or a label that could not be checked.
    Wild,
    /// Finite-type constructor with its payload patterns, keyed in lowercase.
    Ctor(String, Vec<Pat>),
    /// Concrete comparison in an open domain; it cannot complete coverage.
    Value(ConstantValue),
    /// Inclusive constant range; finite domains are checked without enumerating values.
    Range(ConstantValue, ConstantValue),
    /// Comparison whose value cannot be folded.
    Other,
}

impl Checker {
    /// Returns `true` when `row` matches a value that no earlier row matches.
    pub(super) fn pattern_row_is_useful(&self, rows: &[Vec<Pat>], row: &[Pat], tys: &[Ty]) -> bool {
        let Some((head, rest)) = row.split_first() else {
            return rows.is_empty();
        };
        let (ty, rest_tys) = (&tys[0], &tys[1..]);
        match head {
            Pat::Ctor(key, args) => {
                let constructor = self.constructor(ty, key, args.len());
                let specialized = specialize(rows, key, args.len(), constructor.value.as_ref());
                let mut next = args.clone();
                next.extend_from_slice(rest);
                self.pattern_row_is_useful(
                    &specialized,
                    &next,
                    &concat(&constructor.fields, rest_tys),
                )
            }
            Pat::Value(value) => {
                if !self.value_in_domain(ty, value) {
                    return false;
                }
                self.pattern_row_is_useful(&value_rows(rows, value), rest, rest_tys)
            }
            Pat::Range(start, end) => self
                .range_values(rows, ty, start, end)
                .into_iter()
                .any(|value| self.pattern_row_is_useful(&value_rows(rows, &value), rest, rest_tys)),
            Pat::Other => self.pattern_row_is_useful(&default_rows(rows), rest, rest_tys),
            Pat::Wild => {
                if let Ty::Subrange(range) = self.resolve_visible_type(ty) {
                    self.range_values(
                        rows,
                        ty,
                        &ConstantValue::Integer(range.lower),
                        &ConstantValue::Integer(range.upper),
                    )
                    .into_iter()
                    .any(|value| {
                        self.pattern_row_is_useful(&value_rows(rows, &value), rest, rest_tys)
                    })
                } else {
                    match self.complete_constructors(rows, ty) {
                        Some(constructors) => constructors.into_iter().any(|constructor| {
                            let arity = constructor.fields.len();
                            let mut next = vec![Pat::Wild; arity];
                            next.extend_from_slice(rest);
                            self.pattern_row_is_useful(
                                &specialize(
                                    rows,
                                    &constructor.key,
                                    arity,
                                    constructor.value.as_ref(),
                                ),
                                &next,
                                &concat(&constructor.fields, rest_tys),
                            )
                        }),
                        None => self.pattern_row_is_useful(&default_rows(rows), rest, rest_tys),
                    }
                }
            }
        }
    }

    /// Lists one missing pattern per uncovered top-level constructor of `ty`.
    pub(super) fn missing_patterns(&self, rows: &[Vec<Pat>], ty: &Ty) -> Vec<String> {
        if matches!(self.resolve_visible_type(ty), Ty::Subrange(_)) {
            return self
                .witness(rows, std::slice::from_ref(ty))
                .unwrap_or_default();
        }
        let Some(constructors) = self.constructors(ty) else {
            return Vec::new();
        };
        constructors
            .into_iter()
            .filter_map(|constructor| {
                let arity = constructor.fields.len();
                let witness = self.witness(
                    &specialize(rows, &constructor.key, arity, constructor.value.as_ref()),
                    &constructor.fields,
                )?;
                Some(render(&constructor.display, &witness[..arity]))
            })
            .collect()
    }

    /// Returns a value vector that no row matches, rendered per column.
    fn witness(&self, rows: &[Vec<Pat>], tys: &[Ty]) -> Option<Vec<String>> {
        let Some((ty, rest_tys)) = tys.split_first() else {
            return rows.is_empty().then(Vec::new);
        };
        if let Ty::Subrange(range) = self.resolve_visible_type(ty) {
            return self
                .range_values(
                    rows,
                    ty,
                    &ConstantValue::Integer(range.lower),
                    &ConstantValue::Integer(range.upper),
                )
                .into_iter()
                .find_map(|value| {
                    let mut witness = self.witness(&value_rows(rows, &value), rest_tys)?;
                    let ConstantValue::Integer(value) = value else {
                        return None;
                    };
                    witness.insert(0, value.to_string());
                    Some(witness)
                });
        }
        if let Some(constructors) = self.complete_constructors(rows, ty) {
            return constructors.into_iter().find_map(|constructor| {
                let arity = constructor.fields.len();
                let mut witness = self.witness(
                    &specialize(rows, &constructor.key, arity, constructor.value.as_ref()),
                    &concat(&constructor.fields, rest_tys),
                )?;
                let head = render(&constructor.display, &witness[..arity]);
                witness.drain(..arity);
                witness.insert(0, head);
                Some(witness)
            });
        }
        let mut witness = self.witness(&default_rows(rows), rest_tys)?;
        let head = self
            .constructors(ty)
            .and_then(|constructors| {
                constructors.into_iter().find(|constructor| {
                    !rows.iter().any(
                        |row| matches!(row.first(), Some(Pat::Ctor(key, _)) if *key == constructor.key),
                    )
                })
            })
            .map_or_else(
                || "_".to_string(),
                |constructor| render(&constructor.display, &vec!["_".to_string(); constructor.fields.len()]),
            );
        witness.insert(0, head);
        Some(witness)
    }

    /// Constructors of `ty` when every one of them heads some row.
    fn complete_constructors(&self, rows: &[Vec<Pat>], ty: &Ty) -> Option<Vec<Constructor>> {
        let constructors = self.constructors(ty)?;
        constructors
            .iter()
            .all(|constructor| {
                rows.iter().any(|row| match row.first() {
                    Some(Pat::Ctor(key, _)) => *key == constructor.key,
                    Some(Pat::Range(start, end)) => constructor
                        .value
                        .as_ref()
                        .is_some_and(|value| intervals::contains(start, end, value)),
                    _ => false,
                })
            })
            .then_some(constructors)
    }
}
