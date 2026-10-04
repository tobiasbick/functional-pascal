//! Pattern-matrix specialization for finite variants and open scalar domains.

use super::{Checker, RowPattern, Tag};
use crate::PatternVariant;
use crate::types::Ty;

impl Checker {
    /// Determine whether a pattern product accepts values absent from earlier rows.
    pub(super) fn pattern_row_useful(
        &self,
        matrix: &[Vec<RowPattern>],
        row: &[RowPattern],
        types: &[Ty],
    ) -> bool {
        let Some((head, tail)) = row.split_first() else {
            return matrix.is_empty();
        };
        if matrix.is_empty() {
            return true;
        }
        if matrix
            .iter()
            .any(|row| row.iter().all(|pattern| matches!(pattern, RowPattern::Any)))
        {
            return false;
        }
        let Some((ty, remaining_types)) = types.split_first() else {
            return true;
        };
        match head {
            RowPattern::Constructor(Tag::Integer(low, high), _) => {
                if low > high {
                    return false;
                }
                let mut boundaries = vec![i128::from(*low), i128::from(*high) + 1];
                for previous in matrix {
                    if let Some(RowPattern::Constructor(Tag::Integer(a, b), _)) = previous.first() {
                        for bound in [i128::from(*a), i128::from(*b) + 1] {
                            if bound > i128::from(*low) && bound <= i128::from(*high) {
                                boundaries.push(bound);
                            }
                        }
                    }
                }
                boundaries.sort_unstable();
                boundaries.dedup();
                boundaries.windows(2).any(|bounds| {
                    let point = bounds[0];
                    let specialized = matrix
                        .iter()
                        .filter_map(|previous| match previous.first()? {
                            RowPattern::Any => Some(previous[1..].to_vec()),
                            RowPattern::Constructor(Tag::Integer(a, b), _)
                                if i128::from(*a) <= point && point <= i128::from(*b) =>
                            {
                                Some(previous[1..].to_vec())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    self.pattern_row_useful(&specialized, tail, remaining_types)
                })
            }
            RowPattern::Constructor(Tag::String(low, high), _) => {
                if low > high {
                    return false;
                }
                let lower = (low.clone(), false);
                let upper = (high.clone(), true);
                let mut boundaries = vec![lower.clone(), upper.clone()];
                for previous in matrix {
                    if let Some(RowPattern::Constructor(Tag::String(a, b), _)) = previous.first() {
                        for boundary in [(a.clone(), false), (b.clone(), true)] {
                            if boundary > lower && boundary < upper {
                                boundaries.push(boundary);
                            }
                        }
                    }
                }
                boundaries.sort();
                boundaries.dedup();
                boundaries.windows(2).any(|bounds| {
                    let specialized = matrix
                        .iter()
                        .filter_map(|previous| match previous.first()? {
                            RowPattern::Any => Some(previous[1..].to_vec()),
                            RowPattern::Constructor(Tag::String(a, b), _)
                                if (a.clone(), false) <= bounds[0]
                                    && bounds[1] <= (b.clone(), true) =>
                            {
                                Some(previous[1..].to_vec())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    self.pattern_row_useful(&specialized, tail, remaining_types)
                })
            }
            RowPattern::Constructor(tag, arguments) => {
                let payloads = self
                    .pattern_constructors(ty)
                    .into_iter()
                    .find(|(candidate, _)| candidate == tag)
                    .map(|(_, payloads)| payloads)
                    .unwrap_or_default();
                let matrix = specialize(matrix, tag, arguments.len());
                let mut row = arguments.clone();
                row.extend_from_slice(tail);
                let mut types = payloads;
                types.extend_from_slice(remaining_types);
                self.pattern_row_useful(&matrix, &row, &types)
            }
            RowPattern::Any => {
                let constructors = self.pattern_constructors(ty);
                if !constructors.is_empty()
                    && constructors.iter().all(|(tag, _)| {
                        matrix.iter().any(|row|
                    matches!(row.first(), Some(RowPattern::Constructor(head, _)) if head == tag))
                    })
                {
                    constructors.into_iter().any(|(tag, payloads)| {
                        let matrix = specialize(matrix, &tag, payloads.len());
                        let mut row = vec![RowPattern::Any; payloads.len()];
                        row.extend_from_slice(tail);
                        let mut types = payloads;
                        types.extend_from_slice(remaining_types);
                        self.pattern_row_useful(&matrix, &row, &types)
                    })
                } else {
                    let matrix = matrix
                        .iter()
                        .filter(|row| matches!(row.first(), Some(RowPattern::Any)))
                        .map(|row| row[1..].to_vec())
                        .collect::<Vec<_>>();
                    self.pattern_row_useful(&matrix, tail, remaining_types)
                }
            }
        }
    }

    fn pattern_constructors(&self, ty: &Ty) -> Vec<(Tag, Vec<Ty>)> {
        match self.resolve_visible_type(ty) {
            Ty::Boolean => vec![(Tag::Boolean(false), vec![]), (Tag::Boolean(true), vec![])],
            Ty::Option(inner) => vec![
                (Tag::Variant(PatternVariant::None), vec![]),
                (Tag::Variant(PatternVariant::Some), vec![*inner]),
            ],
            Ty::Result(ok, error) => vec![
                (Tag::Variant(PatternVariant::Ok), vec![*ok]),
                (Tag::Variant(PatternVariant::Error), vec![*error]),
            ],
            Ty::Enum(enumeration) => enumeration
                .variants
                .iter()
                .enumerate()
                .map(|(index, variant)| {
                    (
                        Tag::Variant(PatternVariant::Enum(index)),
                        variant.fields.iter().map(|(_, ty)| ty.clone()).collect(),
                    )
                })
                .collect(),
            _ => vec![],
        }
    }
}

fn specialize(matrix: &[Vec<RowPattern>], tag: &Tag, arity: usize) -> Vec<Vec<RowPattern>> {
    matrix
        .iter()
        .filter_map(|row| {
            let (head, tail) = row.split_first()?;
            let mut result = match head {
                RowPattern::Any => vec![RowPattern::Any; arity],
                RowPattern::Constructor(candidate, arguments) if candidate == tag => {
                    arguments.clone()
                }
                _ => return None,
            };
            result.extend_from_slice(tail);
            Some(result)
        })
        .collect()
}
