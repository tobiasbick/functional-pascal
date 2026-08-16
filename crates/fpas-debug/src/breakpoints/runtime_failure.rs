//! Stable diagnostic-code selection for inspectable runtime-failure stops.

use std::collections::HashSet;

use fpas_diagnostics::{DiagnosticCode, codes::RUNTIME_ALLOCATED_CODES};

pub(crate) const MAX_RUNTIME_FAILURE_FILTERS: usize = 64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum RuntimeFailurePolicy {
    #[default]
    All,
    Codes(HashSet<DiagnosticCode>),
}

impl RuntimeFailurePolicy {
    pub(crate) fn parse(filters: &[String]) -> Result<Self, RuntimeFailureFilterError> {
        if filters == ["all"] {
            return Ok(Self::All);
        }
        if filters.len() > MAX_RUNTIME_FAILURE_FILTERS {
            return Err(RuntimeFailureFilterError {
                message: format!(
                    "runtime failure filter count {} exceeds the limit {MAX_RUNTIME_FAILURE_FILTERS}",
                    filters.len()
                ),
                hint: "Send fewer exact runtime diagnostic codes or the single filter `all`."
                    .to_string(),
            });
        }
        let mut codes = HashSet::new();
        for filter in filters {
            if filter == "all" {
                return Err(RuntimeFailureFilterError {
                    message: "runtime failure filter `all` cannot be combined with exact codes"
                        .to_string(),
                    hint: "Send only `all`, or send only exact codes such as `F4001`.".to_string(),
                });
            }
            let code = parse_code(filter)?;
            if !RUNTIME_ALLOCATED_CODES.contains(&code) {
                return Err(RuntimeFailureFilterError {
                    message: format!(
                        "runtime failure filter `{filter}` is not an allocated runtime diagnostic code"
                    ),
                    hint: "Use `all` or a code advertised by debugger initialization.".to_string(),
                });
            }
            if !codes.insert(code) {
                return Err(RuntimeFailureFilterError {
                    message: format!("runtime failure filter `{filter}` is duplicated"),
                    hint: "List each exact diagnostic code at most once.".to_string(),
                });
            }
        }
        Ok(Self::Codes(codes))
    }

    pub(crate) fn should_stop(&self, code: DiagnosticCode) -> bool {
        match self {
            Self::All => true,
            Self::Codes(codes) => codes.contains(&code),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeFailureFilterError {
    pub(crate) message: String,
    pub(crate) hint: String,
}

fn parse_code(filter: &str) -> Result<DiagnosticCode, RuntimeFailureFilterError> {
    let Some(digits) = filter.strip_prefix('F') else {
        return Err(invalid_code(filter));
    };
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_code(filter));
    }
    let value = digits.parse::<u16>().map_err(|_| invalid_code(filter))?;
    DiagnosticCode::try_new(value).map_err(|_| invalid_code(filter))
}

fn invalid_code(filter: &str) -> RuntimeFailureFilterError {
    RuntimeFailureFilterError {
        message: format!("runtime failure filter `{filter}` is not an exact Fdddd code"),
        hint: "Use `all` or an advertised code such as `F4001`.".to_string(),
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "catalog fixtures use expect to keep allocation failures local"
)]
mod tests {
    use super::*;

    #[test]
    fn all_exact_empty_and_invalid_filters_are_deterministic() {
        assert!(
            RuntimeFailurePolicy::parse(&["all".to_string()])
                .expect("all")
                .should_stop(fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC)
        );
        assert!(
            RuntimeFailurePolicy::parse(&["F4010".to_string()])
                .expect("exact")
                .should_stop(fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC)
        );
        assert!(
            !RuntimeFailurePolicy::parse(&[])
                .expect("empty")
                .should_stop(fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC)
        );
        for invalid in ["f4010", "F410", "F4017", "F9999", "all,F4010"] {
            assert!(RuntimeFailurePolicy::parse(&[invalid.to_string()]).is_err());
        }
        assert!(RuntimeFailurePolicy::parse(&["all".to_string(), "F4010".to_string()]).is_err());
        for code in RUNTIME_ALLOCATED_CODES {
            let policy = RuntimeFailurePolicy::parse(&[code.to_string()]).expect("allocated code");
            assert!(policy.should_stop(*code));
        }
    }
}
