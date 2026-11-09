# Desktop GUI implementation

Implement a CUI-backed desktop GUI for FPAS, then share MVU and compatible
widget contracts with the TUI. FPAS programs use ordinary CLR libraries and the
current language syntax. The initial targets are `linux-x64` and `win-x64`.

The [native build](../../../src/Fpas.Gui.Cui/native/README.md) provides shared
and static libraries with headless consumer link/render checks. The
[distribution SDK](../../build/distribution.md#native-cui-assets) carries both
library variants and their notices for the host RID. The initial
[C# binding](../../../src/Fpas.Gui.Cui/README.md) provides window/control interop,
UTF-8 buffers and an automatic local native build. The
[packaged FPAS fixture](../../../src/Fpas.Gui.Cui/README.md#packaged-publishing-fixture)
passes in all five publish modes with software and OpenGL rendering on Windows
x64 and Ubuntu x64 under WSL 2. The remaining phases build the public FPAS host
and share its MVU and widget contracts with the TUI.

## Execution order

Follow this order and verify the acceptance criteria in each file.

| Phase | Work | Prerequisites |
| --- | --- | --- |
| 4 | [Shared MVU core](shared-mvu.md) | [FPAS integration fixture](../../../src/Fpas.Gui.Cui/README.md#fpas-integration-fixture) |
| 5 | [Shared widget contracts](shared-widgets.md) | MVU boundaries |
| 6 | [GUI host](gui-host.md) | MVU core and widget contracts |
| 7 | [TUI alignment](tui-alignment.md) | Shared contracts and GUI host |
| 8 | [Verification and documentation](verification-and-documentation.md) | All preceding work |

Establish each stage on Linux first. Record Windows tasks separately and close
them with Windows evidence. A verified Linux slice can feed the next stage while
Windows checks remain open; overall completion requires both platforms.

## Library boundaries

| Library | Responsibility | Dependencies within this design |
| --- | --- | --- |
| `Fpas.Gui.Cui` | C# native binding, C bridge, native assets and build targets | No MVU or widget library |
| `Fpas.Mvu` | FPAS model/update transitions, messages and effect lifetimes | No GUI, terminal or widget types |
| `Fpas.Ui` | FPAS declarative widget data and event contracts | No host or native CUI dependency |
| `Fpas.Gui` | FPAS GUI host and CUI adapter | Binding, MVU core and widget contracts |
| `Fpas.Tui` | Terminal host, cell layout and renderer | MVU core and widget contracts |

Upstream files belong under the repository-root `third-party/<library>/` tree.
For CUI, `third-party/cui/` holds the release files, license notices and
`UPSTREAM.md` with the version, commit and checksums. The C# binding
stays under `src/Fpas.Gui.Cui/`, with the FPAS C bridge and CMake files in `native/`.

The public application path is MVU plus declarative widgets. The low-level C#
binding supports the host and integration fixtures; do not develop a competing
FPAS application API around imperative native calls. Portable views use the
same widget constructors for both hosts. GUI and terminal extensions are explicit.

## Agreed scope

- Build from the [pinned `cui.c` and `cui.h` release files](../../../third-party/cui/UPSTREAM.md).
  Keep upstream files byte-identical and maintain the FPAS bridge and build
  integration locally.
- Build native assets with FPAS and deliver prebuilt assets to GUI consumers.
- Support the existing `fpas publish` modes. NativeAOT links CUI statically from
  the start; ordinary single-file publishing can embed/extract the shared library.
- Keep terminal-only applications independent of CUI and native GUI tooling.
- Provide an interactive FPAS GUI widget gallery under `examples/gui/gallery/`,
  comparable to the TUI gallery; its scope belongs to the [GUI host plan](gui-host.md#widget-gallery-example).
- Upstream security corrections are outside this implementation plan by user
  decision. They are not tasks or completion gates for these stages.
- macOS, other architectures, direct Wayland support, accessibility implementation
  and complete coverage of CUI's widget catalog are outside the initial scope.

Keep these files limited to open implementation work. Move implemented contracts
and examples into the owning library/build/handbook references, remove completed
plan content and delete the folder when the implementation is complete.
