//! Flat postfix wire identities avoid recursive deserialization of hostile types.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{IdentityParameter, NominalKind, TypeIdentity};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Token {
    Unit,
    Boolean,
    Integer,
    Real,
    String,
    ByteBuffer,
    Subrange(i64, i64),
    Nominal(NominalKind, String, usize),
    Array,
    Channel,
    Dictionary,
    Option,
    Result,
    Task,
    Callable(Vec<(String, bool)>, bool, bool),
    Parameter(u16),
}

impl Serialize for TypeIdentity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Self::validate_all(std::slice::from_ref(self), u16::MAX)
            .map_err(serde::ser::Error::custom)?;
        let mut pending = vec![(self, false)];
        let mut tokens = Vec::new();
        while let Some((ty, ready)) = pending.pop() {
            if !ready {
                pending.push((ty, true));
                match ty {
                    Self::Nominal { arguments, .. } => {
                        pending.extend(arguments.iter().rev().map(|ty| (ty, false)))
                    }
                    Self::Array(ty) | Self::Channel(ty) | Self::Option(ty) | Self::Task(ty) => {
                        pending.push((ty, false))
                    }
                    Self::Dictionary(left, right) | Self::Result(left, right) => {
                        pending.push((right, false));
                        pending.push((left, false));
                    }
                    Self::Callable {
                        parameters, result, ..
                    } => {
                        if let Some(result) = result {
                            pending.push((result, false));
                        }
                        pending.extend(
                            parameters
                                .iter()
                                .rev()
                                .map(|parameter| (&parameter.ty, false)),
                        );
                    }
                    _ => {}
                }
                continue;
            }
            tokens.push(match ty {
                Self::Unit => Token::Unit,
                Self::Boolean => Token::Boolean,
                Self::Integer => Token::Integer,
                Self::Real => Token::Real,
                Self::String => Token::String,
                Self::ByteBuffer => Token::ByteBuffer,
                Self::Subrange { lower, upper } => Token::Subrange(*lower, *upper),
                Self::Nominal {
                    kind,
                    declaration,
                    arguments,
                } => Token::Nominal(*kind, declaration.clone(), arguments.len()),
                Self::Array(_) => Token::Array,
                Self::Channel(_) => Token::Channel,
                Self::Dictionary(_, _) => Token::Dictionary,
                Self::Option(_) => Token::Option,
                Self::Result(_, _) => Token::Result,
                Self::Task(_) => Token::Task,
                Self::Callable {
                    parameters,
                    result,
                    variadic,
                } => Token::Callable(
                    parameters
                        .iter()
                        .map(|p| (p.name.clone(), p.mutable))
                        .collect(),
                    result.is_some(),
                    *variadic,
                ),
                Self::Parameter(index) => Token::Parameter(*index),
            });
        }
        tokens.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TypeIdentity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let tokens = bounded_list(deserializer, crate::limits::MAX_LAYOUT_FIELDS)?;
        decode(tokens).map_err(serde::de::Error::custom)
    }
}

pub(super) fn arguments<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<TypeIdentity>, D::Error> {
    bounded_list(deserializer, crate::limits::MAX_LAYOUT_FIELDS)
}

fn bounded_list<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
    maximum: usize,
) -> Result<Vec<T>, D::Error> {
    struct List<T> {
        maximum: usize,
        marker: std::marker::PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for List<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded semantic identity list")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = sequence.next_element()? {
                if values.len() >= self.maximum {
                    return Err(serde::de::Error::custom(
                        "error identity list exceeds its resource limit",
                    ));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(List {
        maximum,
        marker: std::marker::PhantomData,
    })
}

fn decode(tokens: Vec<Token>) -> Result<TypeIdentity, &'static str> {
    if tokens.len() > crate::limits::MAX_LAYOUT_FIELDS {
        return Err("error type identity exceeds its graph limit");
    }
    let mut stack: Vec<(TypeIdentity, usize)> = Vec::new();
    for token in tokens {
        let count = match &token {
            Token::Nominal(_, _, count) => *count,
            Token::Callable(parameters, returns, _) => parameters.len() + usize::from(*returns),
            Token::Array | Token::Channel | Token::Option | Token::Task => 1,
            Token::Dictionary | Token::Result => 2,
            _ => 0,
        };
        if count > stack.len() {
            return Err("error type identity has missing operands");
        }
        let depth = stack[stack.len() - count..]
            .iter()
            .map(|(_, depth)| depth + 1)
            .max()
            .unwrap_or(0);
        if depth >= 64 {
            return Err("error type identity exceeds its graph limit");
        }
        let children = stack
            .split_off(stack.len() - count)
            .into_iter()
            .map(|(ty, _)| ty)
            .collect::<Vec<_>>();
        let mut children = children.into_iter();
        let mut next = || {
            children
                .next()
                .ok_or("error type identity has missing operands")
        };
        let ty = match token {
            Token::Unit => TypeIdentity::Unit,
            Token::Boolean => TypeIdentity::Boolean,
            Token::Integer => TypeIdentity::Integer,
            Token::Real => TypeIdentity::Real,
            Token::String => TypeIdentity::String,
            Token::ByteBuffer => TypeIdentity::ByteBuffer,
            Token::Subrange(lower, upper) => TypeIdentity::Subrange { lower, upper },
            Token::Nominal(kind, declaration, _) => TypeIdentity::Nominal {
                kind,
                declaration,
                arguments: children.collect(),
            },
            Token::Array => TypeIdentity::Array(Box::new(next()?)),
            Token::Channel => TypeIdentity::Channel(Box::new(next()?)),
            Token::Dictionary => TypeIdentity::Dictionary(Box::new(next()?), Box::new(next()?)),
            Token::Option => TypeIdentity::Option(Box::new(next()?)),
            Token::Result => TypeIdentity::Result(Box::new(next()?), Box::new(next()?)),
            Token::Task => TypeIdentity::Task(Box::new(next()?)),
            Token::Parameter(index) => TypeIdentity::Parameter(index),
            Token::Callable(parameters, returns, variadic) => {
                let parameters = parameters
                    .into_iter()
                    .map(|(name, mutable)| {
                        Ok(IdentityParameter {
                            name,
                            mutable,
                            ty: next()?,
                        })
                    })
                    .collect::<Result<Vec<_>, &'static str>>()?;
                TypeIdentity::Callable {
                    parameters,
                    variadic,
                    result: if returns {
                        Some(Box::new(next()?))
                    } else {
                        None
                    },
                }
            }
        };
        stack.push((ty, depth));
    }
    if stack.len() != 1 {
        return Err("error type identity has trailing operands");
    }
    let ty = stack.pop().ok_or("empty error type identity")?.0;
    TypeIdentity::validate_all(std::slice::from_ref(&ty), u16::MAX)?;
    Ok(ty)
}
