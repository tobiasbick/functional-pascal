//! Iterative equality for runtime representations and FPAS value data.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use super::Value;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Representation,
    Language,
}

struct DictionaryComparison<'a> {
    left: &'a [(Value, Value)],
    right: &'a [(Value, Value)],
    index: usize,
    used: Vec<bool>,
}

enum Frame<'a> {
    Compare(&'a Value, &'a Value),
    Sequence {
        pairs: Vec<(&'a Value, &'a Value)>,
        index: usize,
    },
    Dictionary(DictionaryComparison<'a>),
    FindKey {
        dictionary: DictionaryComparison<'a>,
        candidate: usize,
        awaiting: bool,
    },
}

/// Compare stored representations, preserving NaN bits and dictionary order.
pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    compare(a, b, Mode::Representation)
}

/// Compare value data with IEEE real equality and dictionary mapping equality.
pub(super) fn language_values_equal(a: &Value, b: &Value) -> bool {
    compare(a, b, Mode::Language)
}

fn compare<'a>(a: &'a Value, b: &'a Value, mode: Mode) -> bool {
    let mut pending = vec![Frame::Compare(a, b)];
    let mut equal = true;
    while let Some(frame) = pending.pop() {
        match frame {
            Frame::Compare(a, b) => {
                let mut pairs = Vec::new();
                equal = true;
                match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) if x == y => {}
                    (Value::Real(x), Value::Real(y))
                        if x == y
                            || (mode == Mode::Representation && x.to_bits() == y.to_bits()) => {}
                    (Value::Integer(x), Value::Real(y))
                        if mode == Mode::Language && (*x as f64) == *y => {}
                    (Value::Real(x), Value::Integer(y))
                        if mode == Mode::Language && *x == (*y as f64) => {}
                    (Value::Boolean(x), Value::Boolean(y)) if x == y => {}
                    (Value::Str(x), Value::Str(y)) if x == y => {}
                    (Value::Enum(a), Value::Enum(b)) => {
                        let a = a.body();
                        let b = b.body();
                        if a.layout != b.layout || a.values.len() != b.values.len() {
                            equal = false;
                            continue;
                        }
                        push_pairs(&mut pairs, &a.values, &b.values);
                    }
                    (Value::Array(a), Value::Array(b)) => {
                        if a.len() != b.len() {
                            equal = false;
                            continue;
                        }
                        push_pairs(&mut pairs, a, b);
                    }
                    (Value::Dict(a), Value::Dict(b)) if mode == Mode::Language => {
                        if a.len() != b.len() {
                            equal = false;
                            continue;
                        }
                        pending.push(Frame::Dictionary(DictionaryComparison {
                            left: a,
                            right: b,
                            index: 0,
                            used: vec![false; b.len()],
                        }));
                        continue;
                    }
                    (Value::Dict(a), Value::Dict(b)) => {
                        if a.len() != b.len() {
                            equal = false;
                            continue;
                        }
                        for ((left_key, left_value), (right_key, right_value)) in
                            a.iter().zip(b.iter()).rev()
                        {
                            pairs.push((left_value, right_value));
                            pairs.push((left_key, right_key));
                        }
                    }
                    (Value::Record(a), Value::Record(b)) => {
                        let a = a.body();
                        let b = b.body();
                        if a.layout != b.layout || a.values.len() != b.values.len() {
                            equal = false;
                            continue;
                        }
                        push_pairs(&mut pairs, &a.values, &b.values);
                    }
                    (Value::Unit, Value::Unit) | (Value::OptionNone, Value::OptionNone) => {}
                    (Value::ResultOk(a), Value::ResultOk(b))
                    | (Value::ResultError(a), Value::ResultError(b))
                    | (Value::OptionSome(a), Value::OptionSome(b)) => pairs.push((a, b)),
                    (Value::Function(a), Value::Function(b)) if mode == Mode::Representation => {
                        if a.function != b.function
                            || a.name != b.name
                            || a.task_bound != b.task_bound
                            || a.owner_task != b.owner_task
                            || a.captures.len() != b.captures.len()
                        {
                            equal = false;
                            continue;
                        }
                        push_pairs(&mut pairs, &a.captures, &b.captures);
                    }
                    (Value::Cell(a), Value::Cell(b))
                        if mode == Mode::Representation && std::sync::Arc::ptr_eq(a, b) => {}
                    (Value::Reference(a), Value::Reference(b))
                        if mode == Mode::Representation && std::sync::Arc::ptr_eq(a, b) => {}
                    (Value::Task(a), Value::Task(b)) if mode == Mode::Representation && a == b => {}
                    (Value::OpaqueHandle(a), Value::OpaqueHandle(b))
                        if mode == Mode::Representation && a == b => {}
                    _ => {
                        equal = false;
                        continue;
                    }
                }
                if !pairs.is_empty() {
                    pending.push(Frame::Sequence { pairs, index: 0 });
                }
            }
            Frame::Sequence { pairs, index } => {
                if index > 0 && !equal {
                    continue;
                }
                if let Some(&(a, b)) = pairs.get(index) {
                    pending.push(Frame::Sequence {
                        pairs,
                        index: index + 1,
                    });
                    pending.push(Frame::Compare(a, b));
                } else {
                    equal = true;
                }
            }
            Frame::Dictionary(dictionary) => {
                if dictionary.index > 0 && !equal {
                    continue;
                }
                if dictionary.index == dictionary.left.len() {
                    equal = true;
                    continue;
                }
                pending.push(Frame::FindKey {
                    dictionary,
                    candidate: 0,
                    awaiting: false,
                });
            }
            Frame::FindKey {
                mut dictionary,
                mut candidate,
                awaiting,
            } => {
                if awaiting && equal {
                    let left = &dictionary.left[dictionary.index].1;
                    let right = &dictionary.right[candidate].1;
                    dictionary.used[candidate] = true;
                    dictionary.index += 1;
                    pending.push(Frame::Dictionary(dictionary));
                    pending.push(Frame::Compare(left, right));
                    continue;
                }
                if awaiting {
                    candidate += 1;
                }
                while candidate < dictionary.right.len() && dictionary.used[candidate] {
                    candidate += 1;
                }
                if candidate == dictionary.right.len() {
                    equal = false;
                    continue;
                }
                let left = &dictionary.left[dictionary.index].0;
                let right = &dictionary.right[candidate].0;
                pending.push(Frame::FindKey {
                    dictionary,
                    candidate,
                    awaiting: true,
                });
                pending.push(Frame::Compare(left, right));
            }
        }
    }
    equal
}

fn push_pairs<'a>(
    pending: &mut Vec<(&'a Value, &'a Value)>,
    left: &'a [Value],
    right: &'a [Value],
) {
    pending.extend(left.iter().zip(right).rev());
}
