# CoreCLR native connection

The native module calls CoreCLR's COM-style debugging vtables directly from C#.
`ComWrappers` exports the managed callback interfaces on Windows and Linux.
No Windows COM activation or external debugger executable is involved. The
shared core and frontends are described in the [core reference](../../../docs/architecture/debugger-core.md).

## Selected dependency

The Windows x64 and Linux x64 assets are pinned to Microsoft.Diagnostics.DbgShim
10.0.750501. Their NuGet manifests identify source commit
`ad724c4b85e7ae775e42f393c6e95a7557a57954` in dotnet/diagnostics.

| Asset | SHA-256 |
| --- | --- |
| `dbgshim.dll` from `Microsoft.Diagnostics.DbgShim.win-x64` | `18730F1FE4B555AB8F062E4C14104EE64CAC774C9277554E9CC560546ABD22DF` |
| `libdbgshim.so` from `Microsoft.Diagnostics.DbgShim.linux-x64` | `661698F7BE43162F50B465576E059585713F6E715C6F2D5B9BA5E255E9A8DEBA` |

These packages supply native assets, not a managed debugger library. Runtime
startup registration discovers the launched runtime. The selected shim locates
its matching DBI beside that runtime; DBI loads the matching data-access library.
The supported target baseline is the installed .NET 10 CoreCLR layout. Single-file
targets and custom target-runtime library providers are outside this baseline. Debug launch does
not download runtime libraries. Missing or incompatible assets produce a setup
failure before managed program execution.

Native loading and passive inspection share the deployed shim selection:
`runtimes/<rid>/native/<library>`, then a flattened file beside the compiler.
The loader opens that absolute path. An incompatible selected file is a setup
failure even when PATH or LD_LIBRARY_PATH contains another valid copy.

In a [single-file compiler distribution](../../../docs/build/distribution.md),
selection materializes the embedded RID-specific shim in FPAS's verified asset
cache. Inspection reports that same path/hash and the loader opens it absolutely.
The compiler's bundle does not change the supported target-runtime layout.

Source binding uses ICorDebugFunction4 native entry breakpoints when JIT-time
rejection arrives before native code publication. Rebinding then stays within
the original source region, with bounded operand preparation and explicit
per-request rejection. It does not call DisableOptimizations. The
[core reference](../../../docs/architecture/debugger-core.md#breakpoints-and-source-control)
owns this control policy and the distinction between unavailable source points
and a failed native session.

The shim stays loaded for the debugger process lifetime. Native interfaces are
released after target exit and ICorDebug termination. Callback references remain
rooted through startup unregistration and shutdown. A fresh session worker per
target isolates DBI/DAC loading between runtime installations. The
[session reference](../../../docs/architecture/debugger-core.md) owns supervision,
private framing, process containment and shutdown bounds.

## Verification

`NativeCoreTests` runs real FPAS programs through this module. Its cases cover a
source breakpoint, changed local values after stepping, normal exit, stopped
target disposal, input validation, a host exiting before CLR startup and
cancellation after native launch/startup registration while still suspended.
On Linux a private launch broker owns the target's OS parent relationship and
collects its real `waitpid` status independently of DBI. Cancellation checks the
actual process entry is gone before fixture cleanup.

The [test instructions](../../../tests/README.md#own-coreclr-connection-tests)
describe the command. These tests establish the native module's behavior;
frontends, supervision, general values and async stepping have separate coverage.
Debug and Release tool builds are verified on native Windows x64 and WSL2 Linux
x64, with Debug and fully optimized FPAS targets. `NativeAssetTests` also runs the
extracted SDK CLI with missing, incompatible and genuine ARM64 shim files, while
a valid external copy remains discoverable. Only the deployed selected file may
be loaded; unusable assets fail before target code executes.

## Source references

- [Pinned shim implementation](https://github.com/dotnet/diagnostics/blob/ad724c4b85e7ae775e42f393c6e95a7557a57954/src/dbgshim/dbgshim.cpp).
- [Pinned C# interop examples](https://github.com/dotnet/diagnostics/tree/ad724c4b85e7ae775e42f393c6e95a7557a57954/src/tests/DbgShim.UnitTests).
- [.NET 10.0.14 debugging ABI](https://github.com/dotnet/runtime/blob/v10.0.14/src/coreclr/inc/cordebug.idl).
- [.NET 10.0.14 metadata import ABI](https://github.com/dotnet/runtime/blob/v10.0.14/src/coreclr/inc/cor.h).
- [Upstream license](THIRD-PARTY-NOTICES.txt).
