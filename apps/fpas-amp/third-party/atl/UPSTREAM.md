# ATL source snapshot

- Upstream: <https://github.com/Zeugma440/atldotnet>
- Release: `7.20.1`
- Commit: `db4d1efa833939f51dc5d8c6a3d718ec5c3716db`
- License: [MIT](LICENSE.txt).
- Scope: the upstream `ATL/` library directory, with local playlist and ID3 corrections.
- Checksums: [SHA256SUMS](SHA256SUMS) records the retained source/license bytes.
- Build: [FpasAmp.Atl.csproj](../../src/FpasAmp.Atl/FpasAmp.Atl.csproj) builds ATL for
  FPas-Amp and preserves ATL's namespace and assembly name. The upstream project
  file is reference material.

## Local playlist changes

`PlaylistIOFactory` and reader constructors accept `readTrackMetadata` (default
true). The media library passes false to skip media reads and tag serialization
of unloaded tracks. That mode limits parsed lists to 100000 entries. `LoadError`
exposes failures and `OriginalLocations` retains references before host resolution.
New export paths are not parsed before being written.

M3U advances across blank/malformed metadata lines, preserves punctuation in labels
and resets labels between entries. PLS keeps equals signs in locations and sorts
numeric keys independently of INI field order. DPL continues after blank lines
and preserves asterisks inside values. FPL retains source order across URI schemes.
ASX accepts lower-case entry/attribute names; B4S retains self-closing entries.
XSPF handles namespace prefixes, and XML text readers retain CDATA content.
URI targets are retained, file URIs keep their native path data, relative export
paths use `Path.GetRelativePath`, and literal percent characters in host-local
paths are not URI-decoded. Foreign Windows paths remain unresolved on Unix.

Three .NET baseline build adjustments are included: `TagData.cs` enables nullable
annotations without nullable-flow diagnostics on upstream's partially annotated
code; `VorbisTag.SetMetaField` omits a redundant `new`; and file-structure byte
padding writes explicit bytes to avoid the newer `BitConverter.GetBytes` overload
ambiguity (and to retain one-byte padding).

These changes are local; no upstream issue or pull request has been published.
The MIDI source's contact-email line is omitted; its author attribution and
license notice are retained.

## Local ID3 changes

[ID3v2.cs](ATL/AudioData/IO/ID3v2.cs) retains the owner and binary payload of
`UFID` and `PRIV` frames. ATL carries the payload as Base64 in additional
fields under `UFID.<owner>` and `PRIV.<owner>` and decodes it when writing.
This preserves embedded and trailing zero bytes when another tag changes.

## Update

Copy the new release's library directory and license, record its exact commit,
review/reapply the documented playlist and ID3 changes and regenerate the source checksum
manifest. From the FPas-Amp directory, build `src/FpasAmp.Atl/FpasAmp.Atl.csproj`.
Run the [media library tests](../../tests/README.md#media-libraries) and review
the source diff before committing a dependency update.
