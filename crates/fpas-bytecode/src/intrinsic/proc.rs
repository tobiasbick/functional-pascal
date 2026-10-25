//! `Std.Proc` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Proc.*`.
///
/// **Documentation:** `docs/pascal/std/host/proc.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ProcIntrinsic {
    /// Construct interactive process launch options.
    DefaultStartOptions = 724,
    /// Start an owned process with separate nonblocking binary pipes.
    Start = 725,
    /// Probe the process stdout pipe.
    ReadStdout = 726,
    /// Probe the process stderr pipe.
    ReadStderr = 727,
    /// Write a bounded prefix to the process stdin pipe.
    WriteStdin = 728,
    /// Flush and close process stdin when ready.
    CloseStdin = 729,
    /// Probe the direct child's exit status.
    TryWait = 730,
    /// Terminate the owned process group.
    Terminate = 731,
    /// Close all aliases and reap the owned process.
    Close = 732,
    /// Probe the exact native exit code or terminating signal.
    TryWaitStatus = 733,
    /// Run a supervised child using the host's explicit interactive terminal.
    RunInteractive = 734,
    /// `Std.Proc.RunWithOptions` process configuration API.
    RunWithOptions = 721,
    /// `Std.Proc.RunCaptureWithOptions` process configuration API.
    RunCaptureWithOptions = 722,
    /// `Std.Proc.DefaultOptions` process configuration API.
    DefaultOptions = 723,

    /// `Std.Proc.Run(Command, Args)` - run a host process and return its exit code.
    ///
    /// **Documentation:** `docs/pascal/std/host/proc.md`
    Run = 330,
    /// `Std.Proc.CurrentExecutable()` - return the current executable path.
    ///
    /// **Documentation:** `docs/pascal/std/host/proc.md`
    CurrentExecutable = 344,
    /// `Std.Proc.RunCapture(Command, Args)` - run a host process and capture its output.
    ///
    /// **Documentation:** `docs/pascal/std/host/proc.md`
    RunCapture = 345,
}
}
