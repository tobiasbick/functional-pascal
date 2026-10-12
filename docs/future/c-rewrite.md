# Rewrite in C

> **⚠️ Disclaimer:** The human overlord has spoken. Everyone else is rewriting their programs in
> Rust; this hobby project intends to go the other way and rewrite its implementation in pure C.
> This is a statement of intent, not a plan: no schedule, no detailed migration plan, no promise.
> If you depend on this project, please reconsider your life choices — the human sincerely hopes
> nobody does.

## Status

Idea only. No implementation plan. C23 and CMake/Ninja with vcpkg are selected tooling for a
prospective rewrite; the Rust implementation remains the current one until something replaces it.

## Motivation

Not the syntax. The toolchain:

- **Build times.** A full Rust build and test run of the workspace takes far too long for a hobby
  project.
- **Disk usage.** The Cargo `target/` directory of this repository measured 57 GB at the time of
  writing.
- **The agent writes most of the code anyway.** With LLMs getting better at writing code, the
  safety net of the Rust compiler matters less to the human than build speed and a small, simple
  toolchain.

## Scope

- **Implementation only.** Compiler, VM, standard-library runtime, CLI, language server, and debugger
  adapter would be rewritten. The FPAS language itself — syntax, semantics, and the specification
  under `docs/pascal/` — stays as it is.
- **The FPAS test suite is the contract.** The `*_test.fpas` programs and golden files under
  `tests/` are independent of Rust and would decide whether the C implementation behaves like the
  current one. The Rust unit tests (roughly 110k of about 258k lines of Rust) would not carry over.
- **Additional FFI capability.** Plan a general C foreign-function interface for applications,
  separately from preserving the existing language during the rewrite. The intended capability
  is described below; its FPAS syntax and type rules still need their own design agreement.

## C standard and target platforms

**Target C23 for the implementation.** It is the highest published C revision available as a
compiler language mode across the target configurations below. Select it explicitly rather than
relying on compiler defaults. C2Y is still a draft and is outside the target.

This is a language-mode decision, not a claim of complete C23 compiler and standard-library
conformance. Use features supported by the selected compiler/runtime combinations. The
[GCC C status](https://gcc.gnu.org/projects/c-status.html) and
[Clang C status](https://clang.llvm.org/c_status.html) tables document the remaining gaps.

Documentation checked on **2026-10-12**. Assumptions: maintained operating-system versions and the
ability to install a recent GCC or Clang. Installed toolchains were not inspected, and no target
builds were run. For the Raspberry Pi 3, the initial target is 64-bit Linux (ARM64); this is an
explicit choice of OS architecture, not a hardware requirement for C23.

| Target | Compiler route for C23 | Constraints |
| --- | --- | --- |
| Windows 11 x64 | Recent Clang/LLVM targeting the MSVC ABI, with the Windows SDK/UCRT. Clang accepts `-std=c23` from version 18. | Native MSVC `cl.exe` documents C17 as its highest fixed C standard mode; `/std:clatest` enables experimental features. Choose Clang for the C23 target and check runtime/header availability separately. |
| Linux x64 | GCC 14 or newer, or Clang 18 or newer, with `-std=c23`. | These versions accept the mode but do not implement every C23 feature. GCC 15 adds features such as `#embed`; the chosen distribution's libc also determines library availability. |
| Linux on Raspberry Pi 3 | Raspberry Pi OS 64-bit based on Debian Trixie is a concrete route. Debian Trixie provides GCC 14 and Clang 19 for ARM64, both with C23 mode. | The Pi 3 supports this OS. Use the implemented feature set of the chosen compiler; 32-bit Linux would require a separate ARM build/dependency check. |
| FreeBSD x64 (amd64) | Base-system or packaged Clang with C23 mode. FreeBSD 15.0 ships LLVM 19.1.7. | FreeBSD 15.1 documents expanded but incomplete C23 library support. A newer compiler does not replace missing system-library APIs. |

Sources: [MSVC C standard modes](https://learn.microsoft.com/en-us/cpp/build/reference/std-specify-language-standard-version),
[Clang on Windows](https://learn.microsoft.com/en-us/cpp/build/clang-support-cmake),
[GCC 14 language modes](https://gcc.gnu.org/onlinedocs/gcc-14.3.0/gcc/C-Dialect-Options.html),
[Raspberry Pi OS compatibility](https://www.raspberrypi.com/software/operating-systems/),
[Debian Trixie GCC](https://packages.debian.org/trixie/gcc),
[Debian Trixie Clang](https://packages.debian.org/trixie/clang),
[FreeBSD 15.0 toolchain](https://www.freebsd.org/releases/15.0R/relnotes/), and
[FreeBSD 15.1 library support](https://www.freebsd.org/releases/15.1R/relnotes/).

For example, C23 `constexpr` requires Clang 19, and enhanced enums require Clang 20; GCC 14 does not
support `#embed`. Accepting `-std=c23` alone is therefore insufficient to select every C23 feature.
Before implementation, define the actual feature requirements and check their compilation and
any required library linkage on all four targets. Raise compiler requirements when a needed
feature demands it. Keep platform-dependent runtime operations in focused platform modules.

The prospective CMake build should require C23 (`C_STANDARD 23`, `C_STANDARD_REQUIRED ON`) and
request the ISO dialect rather than the GNU dialect (`C_EXTENSIONS OFF`) for project targets.
Requiring the standard prevents CMake from silently selecting an older mode; it does not verify
complete conformance.
See [CMake C_STANDARD](https://cmake.org/cmake/help/latest/prop_tgt/C_STANDARD.html).

## Build and dependency management

**Use CMake with Ninja and vcpkg in manifest mode.** vcpkg manages the implementation's external
C libraries, such as TLS, JSON, and hashing. It does not change FPAS project dependencies or the
source-adjacent `.fpascu` model.

- Install vcpkg separately outside the repository and locate it through `VCPKG_ROOT`. No Git
  submodule or vendored dependency sources are required in the FPAS repository.
- Track `vcpkg.json` with a fixed registry baseline, `CMakeLists.txt`, and `CMakePresets.json`.
  Keep machine-specific paths in environment variables or an ignored `CMakeUserPresets.json`.
  Record the selected vcpkg/toolchain versions when establishing the build.
- Ignore build output and `vcpkg_installed/`. CMake's vcpkg integration installs manifest
  dependencies during configuration and makes them available to normal CMake library discovery.
- Reuse installed dependencies during ordinary FPAS edits. Ninja rebuilds changed C sources and
  sources affected by changed headers, then relinks affected targets. Dependency packages need
  rebuilding when their inputs/configuration change and no matching binary cache entry exists.
- Keep the separate vcpkg binary cache enabled. Deleting the project build directory can remove
  its installed libraries, but matching cached packages can be restored without recompiling
  them. Temporary package build files can be removed with `--clean-after-build`; binary archives
  still consume space. Measure build time and disk usage rather than assuming a reduction.

See [vcpkg installation](https://learn.microsoft.com/en-us/vcpkg/get_started/get-started),
[manifest mode](https://learn.microsoft.com/en-us/vcpkg/concepts/manifest-mode),
[CMake integration](https://learn.microsoft.com/en-us/vcpkg/users/buildsystems/cmake-integration),
[binary caching](https://learn.microsoft.com/en-us/vcpkg/users/binarycaching),
[cleanup options](https://learn.microsoft.com/en-us/vcpkg/commands/install#--clean-after-build), and
[Ninja incremental builds](https://ninja-build.org/manual.html).

ARM Linux, Clang builds on Windows, and BSD fall under vcpkg's community support. On BSD, vcpkg
bootstraps itself from source. Availability of a package on Windows or Linux x64 does not establish
that the selected version builds on ARM64 or FreeBSD: verify the actual dependency set on each
target before adopting it. See [vcpkg supported hosts](https://learn.microsoft.com/en-us/vcpkg/concepts/supported-hosts).

## C libraries in applications through FFI

**Goal: an application can use an additional C library with an already-built FPAS compiler and
runtime.** A developer who finds functionality missing from FPAS should be able to obtain a
native library, describe the required functions in a reusable FPAS binding unit, and call them
from an ordinary FPAS program. Adding those bindings must not require rebuilding or modifying
FPAS, adding a built-in intrinsic, or waiting for a new FPAS release.

The intended workflow is:

1. Obtain the library as a Windows DLL or a Linux/FreeBSD shared object (`.so`) built for the
   application's target OS and architecture.
2. Write or reuse a binding unit describing the library, exported symbol names, and C function
   signatures. Import that unit through the existing FPAS unit/project model.
3. Compile and run the FPAS application normally. The runtime loads the native library, resolves
   its symbols, converts arguments/results, and performs calls using the platform's C ABI.
4. Deploy the application with its required native libraries, or resolve documented system
   libraries. Compiled artifacts carry binding metadata; native addresses are resolved at runtime.

**Evaluate libffi as the native-call implementation.** It handles platform calling conventions
and calls described by runtime signatures. Library loading, symbol resolution, FPAS/C value
conversion, and ownership remain FPAS responsibilities. It is available as a vcpkg port and is a
candidate for the implementation's dependencies; application authors would not need vcpkg or a
C compiler merely to use an existing shared library and its bindings. See
[libffi](https://github.com/libffi/libffi) and
[the vcpkg libffi port](https://github.com/microsoft/vcpkg/blob/master/ports/libffi/vcpkg.json).

The binding design must specify C type widths/layouts and calling conventions, string encoding,
array/buffer representation, pointer and library-handle lifetimes, and who allocates/frees each
value crossing the boundary. Records passed by value, callbacks into FPAS, variadic functions,
and interaction with FPAS tasks need explicit support decisions. Native-library discovery and
deployment rules are also open. Binding units use the existing path/workspace dependencies;
native libraries do not introduce a new FPAS package manager or compiled-unit format.

This capability is not implemented today and is independent of the C23 choice. Its acceptance
check should use the same already-built FPAS distribution with a subsequently added binding
unit and shared library on every target in the platform matrix. Cover supported argument/result
conversions, ownership/lifetime behavior, and useful errors for missing libraries or symbols.
Choose and agree on the FPAS API/syntax before implementation or changes to `docs/pascal/`.

## Open questions

- **Exact toolchain and library requirements.** Select compiler versions and the C23 features/APIs
  actually used, Windows runtime/linkage settings, and dependency versions for the target matrix.
- **Memory safety.** The Rust workspace forbids `unsafe` code; in C that guarantee moves from the
  compiler to discipline and tooling. Sanitizers (ASan, UBSan) and fuzzing of lexer, parser,
  bytecode verifier, and `.fpascu` loading would need to be part of the normal test run.
- **Dependencies.** The Rust implementation uses crates for TLS (`rustls`), terminal control
  (`crossterm`), JSON, TOML, glob patterns, hashing, and Unicode segmentation. TLS and cryptography
  must not be self-written; a C library such as mbedTLS or OpenSSL would be needed. The rest is a
  choice between small libraries and own code.
- **Migration path.** Big-bang rewrite, or component by component behind the existing file formats
  (bytecode, `.fpascu` units, program images) so the C and Rust parts can be checked against each
  other.
- **Editor tooling.** The VS Code extension talks to `fpas lsp` and the DAP adapter, so it would keep
  working as long as the C implementation speaks the same protocols.
