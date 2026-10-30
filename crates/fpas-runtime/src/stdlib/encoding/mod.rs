//! Strict byte codecs with bounded binary inputs and outputs.
//!
//! **Documentation:** `docs/pascal/std/text/encoding.md`

use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::{IntrinsicCall, expect_bytes, expect_str, pop_value};
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

/// Execute one codec without changing its input bytes or text.
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
            let bytes = expect_bytes(argument, location)?;
            if bytes.len() > MAX_BYTES {
                Err(CodecFailure::LimitExceeded)
            } else {
                Ok(Value::Str(
                    if operation == EncodingIntrinsic::HexEncode {
                        hex::encode(bytes)
                    } else {
                        base64::encode(bytes, operation == EncodingIntrinsic::Base64UrlEncode)
                    }
                    .into(),
                ))
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
            bytes.map(Value::native_bytes)
        }
    };
    call.push(match result {
        Ok(value) => Value::result_ok(value),
        Err(error) => Value::result_error(error.into_value(call.factory(), location)?),
    });
    Ok(Some(()))
}
