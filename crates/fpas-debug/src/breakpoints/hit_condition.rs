//! Positive-decimal exact-hit condition parsing.

use crate::evaluation::EvaluationParseError;

pub(super) fn parse(source: &str) -> Result<u64, EvaluationParseError> {
    if source.is_empty()
        || source.trim() != source
        || !source.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid(source));
    }
    let value = source.parse::<u64>().map_err(|_| invalid(source))?;
    if value == 0 {
        return Err(invalid(source));
    }
    Ok(value)
}

fn invalid(source: &str) -> EvaluationParseError {
    EvaluationParseError {
        code: "invalid_hit_condition",
        message: format!(
            "hit condition `{source}` is invalid; expected one positive decimal integer"
        ),
        hint: "Use `3` to match exactly the third physical hit.".to_string(),
        offset: 0,
        length: source.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn accepts_positive_decimal_and_rejects_every_other_first_contract_form() {
        assert_eq!(parse("1").expect("first hit"), 1);
        assert_eq!(
            parse("18446744073709551615").expect("maximum hit"),
            u64::MAX
        );
        for source in [
            "",
            "0",
            "-1",
            "1.5",
            ">= 3",
            " 3",
            "3 ",
            "٣",
            "18446744073709551616",
        ] {
            let error = parse(source).expect_err("invalid hit condition");
            assert_eq!(error.code, "invalid_hit_condition");
        }
    }
}
