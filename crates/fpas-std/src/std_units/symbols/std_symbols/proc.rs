//! `Std.Proc` symbol names and registry group.

std_symbol!(STD_PROC_ENVIRONMENT_VARIABLE = std_proc!("EnvironmentVariable"));
std_symbol!(STD_PROC_PROCESS_OPTIONS = std_proc!("ProcessOptions"));
std_symbol!(STD_PROC_RUN_WITH_OPTIONS = std_proc!("RunWithOptions"));
std_symbol!(STD_PROC_RUN_CAPTURE_WITH_OPTIONS = std_proc!("RunCaptureWithOptions"));
std_symbol!(STD_PROC_DEFAULT_OPTIONS = std_proc!("DefaultOptions"));
std_symbol!(STD_PROC_PROCESS = std_proc!("Process"));
std_symbol!(STD_PROC_START_OPTIONS = std_proc!("ProcessStartOptions"));
std_symbol!(STD_PROC_DEFAULT_START_OPTIONS = std_proc!("DefaultStartOptions"));
std_symbol!(STD_PROC_START = std_proc!("Start"));
std_symbol!(STD_PROC_READ_STDOUT = std_proc!("ReadStdout"));
std_symbol!(STD_PROC_READ_STDERR = std_proc!("ReadStderr"));
std_symbol!(STD_PROC_WRITE_STDIN = std_proc!("WriteStdin"));
std_symbol!(STD_PROC_CLOSE_STDIN = std_proc!("CloseStdin"));
std_symbol!(STD_PROC_TRY_WAIT = std_proc!("TryWait"));
std_symbol!(STD_PROC_TERMINATE = std_proc!("Terminate"));
std_symbol!(STD_PROC_CLOSE = std_proc!("Close"));

/// Qualified name of the `Std.Proc.ProcessOutput` record.
pub const STD_PROC_PROCESS_OUTPUT: &str = std_proc!("ProcessOutput");
/// Qualified name of `Std.Proc.CurrentExecutable`.
pub const STD_PROC_CURRENT_EXECUTABLE: &str = std_proc!("CurrentExecutable");
/// Qualified name of `Std.Proc.Run`.
pub const STD_PROC_RUN: &str = std_proc!("Run");
/// Qualified name of `Std.Proc.RunCapture`.
pub const STD_PROC_RUN_CAPTURE: &str = std_proc!("RunCapture");

pub(in crate::std_units) const STD_PROC_SYMBOLS: &[&str] = &[
    STD_PROC_PROCESS,
    STD_PROC_START_OPTIONS,
    STD_PROC_DEFAULT_START_OPTIONS,
    STD_PROC_START,
    STD_PROC_READ_STDOUT,
    STD_PROC_READ_STDERR,
    STD_PROC_WRITE_STDIN,
    STD_PROC_CLOSE_STDIN,
    STD_PROC_TRY_WAIT,
    STD_PROC_TERMINATE,
    STD_PROC_CLOSE,
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
