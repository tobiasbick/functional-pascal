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

/// Coverage view of one checked pattern.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Pat {
    /// Matches every value: `_`, `const Name`, or a label that could not be checked.
    Wild,
    /// Finite-type constructor with its payload patterns, keyed in lowercase.
    Ctor(String, Vec<Pat>),
    /// Concrete comparison in an open domain; it cannot complete coverage.
    Value(ConstantValue),
    /// Comparison whose value cannot be folded, or a scalar range.
    Other,
}

/// One constructor of a finite type: lowercase key, display name, and payload types.
struct Constructor {
    key: String,
    display: String,
    fields: Vec<Ty>,
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
                let field_tys = self.constructor_fields(ty, key, args.len());
                let specialized = specialize(rows, key, args.len());
                let mut next = args.clone();
                next.extend_from_slice(rest);
                self.pattern_row_is_useful(&specialized, &next, &concat(&field_tys, rest_tys))
            }
            Pat::Value(value) => {
                self.pattern_row_is_useful(&value_rows(rows, value), rest, rest_tys)
            }
            Pat::Other => self.pattern_row_is_useful(&default_rows(rows), rest, rest_tys),
            Pat::Wild => match self.complete_constructors(rows, ty) {
                Some(constructors) => constructors.into_iter().any(|constructor| {
                    let arity = constructor.fields.len();
                    let mut next = vec![Pat::Wild; arity];
                    next.extend_from_slice(rest);
                    self.pattern_row_is_useful(
                        &specialize(rows, &constructor.key, arity),
                        &next,
                        &concat(&constructor.fields, rest_tys),
                    )
                }),
                None => self.pattern_row_is_useful(&default_rows(rows), rest, rest_tys),
            },
        }
    }

    /// Lists one missing pattern per uncovered top-level constructor of `ty`.
    pub(super) fn missing_patterns(&self, rows: &[Vec<Pat>], ty: &Ty) -> Vec<String> {
        let Some(constructors) = self.constructors(ty) else {
            return Vec::new();
        };
        constructors
            .into_iter()
            .filter_map(|constructor| {
                let arity = constructor.fields.len();
                let witness = self.witness(
                    &specialize(rows, &constructor.key, arity),
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
        if let Some(constructors) = self.complete_constructors(rows, ty) {
            return constructors.into_iter().find_map(|constructor| {
                let arity = constructor.fields.len();
                let mut witness = self.witness(
                    &specialize(rows, &constructor.key, arity),
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
                rows.iter().any(
                    |row| matches!(row.first(), Some(Pat::Ctor(key, _)) if *key == constructor.key),
                )
            })
            .then_some(constructors)
    }

    fn constructor_fields(&self, ty: &Ty, key: &str, arity: usize) -> Vec<Ty> {
        self.constructors(ty)
            .and_then(|constructors| {
                constructors
                    .into_iter()
                    .find(|constructor| constructor.key == key)
                    .map(|constructor| constructor.fields)
            })
            .filter(|fields| fields.len() == arity)
            .unwrap_or_else(|| vec![Ty::Error; arity])
    }

    fn constructors(&self, ty: &Ty) -> Option<Vec<Constructor>> {
        if let Some(enum_ty) = self.resolve_enum_ty(ty) {
            return Some(
                enum_ty
                    .variants
                    .iter()
                    .map(|variant| Constructor {
                        key: variant.name.to_ascii_lowercase(),
                        display: format!("{}.{}", enum_ty.name, variant.name),
                        fields: variant.fields.iter().map(|(_, ty)| ty.clone()).collect(),
                    })
                    .collect(),
            );
        }
        let constructor = |key: &str, display: &str, fields: Vec<Ty>| Constructor {
            key: key.to_string(),
            display: display.to_string(),
            fields,
        };
        match ty {
            Ty::Option(inner) => Some(vec![
                constructor("some", "Some", vec![(**inner).clone()]),
                constructor("none", "None", Vec::new()),
            ]),
            Ty::Result(ok, error) => Some(vec![
                constructor("ok", "Ok", vec![(**ok).clone()]),
                constructor("error", "Error", vec![(**error).clone()]),
            ]),
            Ty::Boolean => Some(vec![
                constructor("true", "true", Vec::new()),
                constructor("false", "false", Vec::new()),
            ]),
            _ => None,
        }
    }
}

/// Rows whose head matches constructor `key`, with its payload expanded in place.
fn specialize(rows: &[Vec<Pat>], key: &str, arity: usize) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| {
            let (head, rest) = row.split_first()?;
            let mut next = match head {
                Pat::Ctor(head_key, args) if head_key == key && args.len() == arity => args.clone(),
                Pat::Wild => vec![Pat::Wild; arity],
                _ => return None,
            };
            next.extend_from_slice(rest);
            Some(next)
        })
        .collect()
}

/// Rows whose head matches every value, without that column.
fn default_rows(rows: &[Vec<Pat>]) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| match row.split_first() {
            Some((Pat::Wild, rest)) => Some(rest.to_vec()),
            _ => None,
        })
        .collect()
}

/// Rows that match this concrete open-domain value, without its column.
fn value_rows(rows: &[Vec<Pat>], value: &ConstantValue) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| match row.split_first() {
            Some((Pat::Wild, rest)) => Some(rest.to_vec()),
            Some((Pat::Value(previous), rest)) if previous == value => Some(rest.to_vec()),
            _ => None,
        })
        .collect()
}

fn concat(first: &[Ty], rest: &[Ty]) -> Vec<Ty> {
    first.iter().chain(rest).cloned().collect()
}

fn render(display: &str, fields: &[String]) -> String {
    if fields.is_empty() {
        display.to_string()
    } else {
        format!("{display}({})", fields.join(", "))
    }
}
