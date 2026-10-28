//! Strict byte codecs with bounded binary inputs and outputs.
//!
//! **Documentation:** `docs/pascal/std/text/encoding.md`

use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::{IntrinsicCall, expect_array, expect_str, pop_int, pop_value};
use crate::{EncodingIntrinsic, Intrinsic, SourceLocation, Value};

mod base64;
mod error;
mod hex;

#[cfg(test)]
mod base64_tests;
#[cfg(test)]
mod tests;

use error::CodecFailure;

const MAX_BYTES: usize = 1024 * 1024;

/// Execute one codec without changing its input array or text.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Encoding(operation) = intrinsic else {
        return Ok(None);
    };
    let argument = pop_value(call, location)?;
    let result = match operation {
        EncodingIntrinsic::Base64Encode
        | EncodingIntrinsic::Base64UrlEncode
        | EncodingIntrinsic::HexEncode => {
            let values = expect_array(argument, location)?;
            match read_bytes(values, location)? {
                Ok(bytes) => Ok(Value::Str(
                    if operation == EncodingIntrinsic::HexEncode {
                        hex::encode(&bytes)
                    } else {
                        base64::encode(&bytes, operation == EncodingIntrinsic::Base64UrlEncode)
                    }
                    .into(),
                )),
                Err(error) => Err(error),
            }
        }
        EncodingIntrinsic::Base64Decode
        | EncodingIntrinsic::Base64UrlDecode
        | EncodingIntrinsic::HexDecode => {
            let text = expect_str(argument, location)?;
            let bytes = if operation == EncodingIntrinsic::HexDecode {
                hex::decode(text)
            } else {
                base64::decode(text, operation == EncodingIntrinsic::Base64UrlDecode)
            };
            bytes.map(|bytes| {
                Value::Array(
                    bytes
                        .into_iter()
                        .map(|byte| Value::Integer(i64::from(byte)))
                        .collect::<Vec<_>>()
                        .into(),
                )
            })
        }
    };
    call.push(match result {
        Ok(value) => Value::result_ok(value),
        Err(error) => Value::result_error(error.into_value(call.factory(), location)?),
    });
    Ok(Some(()))
}

fn read_bytes(
    values: &[Value],
    location: SourceLocation,
) -> Result<Result<Vec<u8>, CodecFailure>, StdError> {
    if values.len() > MAX_BYTES {
        return Ok(Err(CodecFailure::LimitExceeded));
    }
    for (offset, value) in values.iter().enumerate() {
        let value = pop_int(value, location)?;
        if !(0..=255).contains(&value) {
            return Ok(Err(CodecFailure::InvalidByte { offset, value }));
        }
    }
    // Validate the complete byte domain before allocating the conversion buffer.
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        bytes.push(pop_int(value, location)? as u8);
    }
    Ok(Ok(bytes))
}
