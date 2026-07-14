# Library dependency example (path-based)

A minimal **program + library** layout without a workspace file. The program references the library via `[dependencies].projects`.

```text
library-deps/
  mylib/mylib.fpasprj      kind = library
  mylib/src/core.fpas      unit MyLib.Core
  app/app.fpasprj          kind = program, depends on ../mylib
  app/src/main.fpas
```

## Run

From the repository root:

```sh
fpas run examples/pascal/library-deps/app/app.fpasprj
fpas check examples/pascal/library-deps/mylib/mylib.fpasprj
```

The program imports `MyLib.Core` through `uses`. `MyLib.Internal` is used inside the library but omitted from `[exports].units`, so dependents cannot import it directly. Sources are merged at load time (see [Projects](../../../docs/pascal/program-structure/projects.md)).
