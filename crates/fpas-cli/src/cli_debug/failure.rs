//! Preserve compiler and project diagnostic records across every debugger frontend.

use crate::cli_output::CliFailure;

#[derive(Debug)]
pub(crate) enum Failure {
    Message(String),
    Diagnostics(CliFailure),
}
impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
impl From<CliFailure> for Failure {
    fn from(records: CliFailure) -> Self {
        Self::Diagnostics(records)
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Diagnostics(error) => error.fmt(f),
        }
    }
}
impl Failure {
    pub(crate) fn report(&self) -> serde_json::Value {
        let mut report = fpas_debug::batch::empty_report("error", 1);
        report["diagnostics"] = match self {
            Self::Message(message) => serde_json::json!([fpas_debug::batch::diagnostic(
                "target_setup",
                "setup",
                message
            )]),
            Self::Diagnostics(failure) => serde_json::Value::Array(
                failure
                    .records()
                    .iter()
                    .take(64)
                    .map(|record| {
                        let path = record.path.as_ref().map(|path| path.to_string_lossy());
                        fpas_debug::batch::fp_diagnostic(
                            path.as_deref(),
                            record.source.as_deref(),
                            &record.diagnostic,
                        )
                    })
                    .collect(),
            ),
        };
        if let Self::Diagnostics(failure) = self
            && failure.records().len() > 64
        {
            report["capture"]["incomplete"] = serde_json::json!(true);
            report["capture"]["reasons"] = serde_json::json!(["diagnostic_limit"]);
        }
        report
    }
}
