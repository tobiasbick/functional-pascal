//! VM-local tiktoken tables; encoding and decoding never require network or file I/O.

use super::super::{VmError, worker::Worker};
use crate::{Intrinsic, SourceLocation, TokenizerIntrinsic as Op, Value};
use std::sync::OnceLock;
use tiktoken_rs::{CoreBPE, Rank};

#[cfg(test)]
mod tests;

const ENCODINGS: [&str; 6] = [
    "o200k_base",
    "cl100k_base",
    "r50k_base",
    "p50k_base",
    "p50k_edit",
    "o200k_harmony",
];
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOKENS: usize = 1_048_576;

/// Own tables per VM rather than retaining process-global tokenizer singletons.
#[derive(Default)]
pub(super) struct Cache([OnceLock<Result<CoreBPE, ()>>; ENCODINGS.len()]);

impl Cache {
    fn get(&self, index: usize) -> Result<&CoreBPE, Failure> {
        self.0[index]
            .get_or_init(|| {
                std::panic::catch_unwind(|| match index {
                    0 => tiktoken_rs::o200k_base(),
                    1 => tiktoken_rs::cl100k_base(),
                    2 => tiktoken_rs::r50k_base(),
                    3 => tiktoken_rs::p50k_base(),
                    4 => tiktoken_rs::p50k_edit(),
                    5 => tiktoken_rs::o200k_harmony(),
                    _ => unreachable!("validated encoding index"),
                })
                .ok()
                .and_then(Result::ok)
                .ok_or(())
            })
            .as_ref()
            .map_err(|()| Failure::Backend)
    }
}

#[derive(Debug, PartialEq)]
enum Failure {
    Encoding(String),
    Token(usize, i64),
    Utf8,
    Limit(&'static str, usize),
    Backend,
}

fn encoding(name: &str) -> Result<usize, Failure> {
    ENCODINGS
        .iter()
        .position(|candidate| *candidate == name)
        .ok_or_else(|| Failure::Encoding(name.to_owned()))
}

fn text_limit(text: &str) -> Result<(), Failure> {
    if text.len() > MAX_BYTES {
        Err(Failure::Limit("TextBytes", MAX_BYTES))
    } else {
        Ok(())
    }
}

fn decode(bpe: &CoreBPE, tokens: &[Rank]) -> Result<String, Failure> {
    let mut bytes = Vec::new();
    for (index, token) in tokens.iter().copied().enumerate() {
        let part = bpe
            .decode_bytes(&[token])
            .map_err(|_| Failure::Token(index, i64::from(token)))?;
        if part.len() > MAX_BYTES - bytes.len() {
            return Err(Failure::Limit("DecodedBytes", MAX_BYTES));
        }
        bytes.extend(part);
    }
    String::from_utf8(bytes).map_err(|_| Failure::Utf8)
}

fn token_ids(values: &[Value]) -> Result<Vec<Rank>, Failure> {
    if values.len() > MAX_TOKENS {
        return Err(Failure::Limit("Tokens", MAX_TOKENS));
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let Value::Integer(value) = value else {
                unreachable!("validated integer token array");
            };
            Rank::try_from(*value).map_err(|_| Failure::Token(index, *value))
        })
        .collect()
}

impl Worker {
    pub(super) fn execute_tokenizer_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Tokenizer(operation) = intrinsic else {
            return Ok(None);
        };
        if operation == Op::Encodings && arguments.is_empty() {
            return Ok(Some(Some(Value::Array(
                ENCODINGS
                    .iter()
                    .map(|name| Value::Str((*name).into()))
                    .collect(),
            ))));
        }
        // Invalid bytecode/host values use VM diagnostics rather than panicking in native code.
        let input = match (operation, arguments) {
            (Op::Count | Op::Encode, [Value::Str(text), Value::Str(name)]) => {
                (name.as_ref(), Some(text.as_ref()), None)
            }
            (Op::Decode, [Value::Array(tokens), Value::Str(name)])
                if tokens
                    .iter()
                    .all(|value| matches!(value, Value::Integer(_))) =>
            {
                (name.as_ref(), None, Some(tokens.as_ref()))
            }
            _ => {
                return Err(self.runtime_error(
                    fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    "Tokenizer intrinsic arguments do not match its signature",
                    "Pass text or an integer token array and an explicit encoding name.",
                ));
            }
        };
        let result = (|| {
            let index = encoding(input.0)?;
            if let Some(text) = input.1 {
                text_limit(text)?;
                let bpe = self.hosted.tokenizers.get(index)?;
                let tokens = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    bpe.encode_ordinary(text)
                }))
                .map_err(|_| Failure::Backend)?;
                if tokens.len() > MAX_TOKENS {
                    return Err(Failure::Limit("Tokens", MAX_TOKENS));
                }
                Ok(if operation == Op::Count {
                    Value::Integer(tokens.len() as i64)
                } else {
                    Value::Array(
                        tokens
                            .into_iter()
                            .map(|id| Value::Integer(i64::from(id)))
                            .collect(),
                    )
                })
            } else {
                let ids = token_ids(input.2.unwrap_or_default())?;
                decode(self.hosted.tokenizers.get(index)?, &ids).map(|text| Value::Str(text.into()))
            }
        })();
        let value = match result {
            Ok(value) => Value::result_ok(value),
            Err(failure) => {
                let (variant, fields) = match failure {
                    Failure::Encoding(name) => (0, vec![Value::Str(name.into())]),
                    Failure::Token(index, value) => {
                        (1, vec![Value::Integer(index as i64), Value::Integer(value)])
                    }
                    Failure::Utf8 => (2, vec![]),
                    Failure::Limit(name, limit) => (
                        3,
                        vec![Value::Str(name.into()), Value::Integer(limit as i64)],
                    ),
                    Failure::Backend => (4, vec![]),
                };
                Value::result_error(crate::stdlib::native_errors::value(
                    self.layouts.as_ref(),
                    fpas_builtins::native_errors::NativeErrorFamily::Tokenizer,
                    variant,
                    fields,
                    location,
                )?)
            }
        };
        Ok(Some(Some(value)))
    }
}
