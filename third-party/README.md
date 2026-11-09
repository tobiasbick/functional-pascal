# Third-party source dependencies

Keep each upstream dependency in its own directory. Include the required source
files, complete license notices and an `UPSTREAM.md` recording its origin,
version, commit, checksum and update procedure.

Keep project-owned bindings, native bridges and build definitions under `src/`.
Generated binaries and temporary build files do not belong in this directory.

| Dependency | Source and license inventory |
| --- | --- |
| CUI | [CUI provenance](cui/UPSTREAM.md) |
