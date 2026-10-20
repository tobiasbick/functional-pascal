//! `Std.Proc` symbol names and registry group.

std_symbol!(STD_PROC_ENVIRONMENT_VARIABLE = std_proc!("EnvironmentVariable"));
std_symbol!(STD_PROC_PROCESS_OPTIONS = std_proc!("ProcessOptions"));
std_symbol!(STD_PROC_RUN_WITH_OPTIONS = std_proc!("RunWithOptions"));
std_symbol!(STD_PROC_RUN_CAPTURE_WITH_OPTIONS = std_proc!("RunCaptureWithOptions"));
std_symbol!(STD_PROC_DEFAULT_OPTIONS = std_proc!("DefaultOptions"));

/// Qualified name of the `Std.Proc.ProcessOutput` record.
pub const STD_PROC_PROCESS_OUTPUT: &str = std_proc!("ProcessOutput");
/// Qualified name of `Std.Proc.CurrentExecutable`.
pub const STD_PROC_CURRENT_EXECUTABLE: &str = std_proc!("CurrentExecutable");
/// Qualified name of `Std.Proc.Run`.
pub const STD_PROC_RUN: &str = std_proc!("Run");
/// Qualified name of `Std.Proc.RunCapture`.
pub const STD_PROC_RUN_CAPTURE: &str = std_proc!("RunCapture");

pub(in crate::std_units) const STD_PROC_SYMBOLS: &[&str] = &[
    STD_PROC_ENVIRONMENT_VARIABLE,
    STD_PROC_PROCESS_OPTIONS,
    STD_PROC_RUN_WITH_OPTIONS,
    STD_PROC_RUN_CAPTURE_WITH_OPTIONS,
    STD_PROC_DEFAULT_OPTIONS,
    STD_PROC_PROCESS_OUTPUT,
    STD_PROC_CURRENT_EXECUTABLE,
    STD_PROC_RUN,
    STD_PROC_RUN_CAPTURE,
];
