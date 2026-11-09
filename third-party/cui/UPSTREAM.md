# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Release | [v1.2.0](https://github.com/yappologistic/CUI/releases/tag/v1.2.0) |
| Commit | `7bb08ff7c56eb912fe7133985f8b7fa65098ea2e` |
| Header | [cui_single.h](cui_single.h) |
| Release asset | <https://github.com/yappologistic/CUI/releases/download/v1.2.0/cui_single.h> |
| Header SHA-256 | `c16ae25c41d4cf8164f8ca111c5d7d8ba5fd2e27bb7dd323bd53288f2bff274e` |
| Local source modifications | None |

The header is byte-identical to the pinned release asset and to the output of
`tools/amalgamate.py` at the pinned commit. It includes the implementation,
platform backends, embedded fonts/icons and the original source license blocks.
Regular builds consume this checked-in file without downloading or regenerating it.

Project-owned integration belongs under `src/Fpas.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## License inventory

The paths in the middle column refer to the pinned upstream repository.
Standalone notices make the applicable terms available for binary distribution
without requiring users to inspect the implementation header.

| Local notice | Upstream source | Terms |
| --- | --- | --- |
| [LICENSE-CUI.txt](LICENSE-CUI.txt) | `LICENSE` | MIT |
| [LICENSE-stb.txt](LICENSE-stb.txt) | License blocks in `third_party/stb_image.h`, `stb_image_write.h`, `stb_truetype.h` | MIT or public domain, at the recipient's option |
| [LICENSE-Unicode.txt](LICENSE-Unicode.txt) | `third_party/LICENSE-Unicode.txt` | Unicode License V3 |
| [LICENSE-Inter.txt](LICENSE-Inter.txt) | `assets/fonts/LICENSE-Inter.txt` | SIL Open Font License 1.1 |
| [LICENSE-Geist.txt](LICENSE-Geist.txt) | `assets/fonts/LICENSE-Geist.txt` | SIL Open Font License 1.1 |
| [LICENSE-Lucide.txt](LICENSE-Lucide.txt) | `assets/fonts/LICENSE-Lucide.txt` | ISC, including Feather-derived icons under MIT |

The CUI, Unicode and font/icon notice files are copied unchanged. The stb notice
contains the identical dual-license text shared by the three embedded stb
headers, extracted without its enclosing C comment delimiters. Their original
license blocks also remain inside `cui_single.h`.

Keep these notices when redistributing the dependency, including native,
single-file and NativeAOT application distributions.

## Updating the dependency

1. Choose an upstream release deliberately and resolve its tag to a commit.
2. Obtain that release's `cui_single.h` asset and the CUI, Unicode and font/icon
   notices from that same commit. Review changes in source and licensing.
3. Verify the asset checksum against the published digest. When checking the
   generated distribution, run `python3 tools/amalgamate.py` in a separate
   upstream checkout and compare its header byte for byte with the release asset.
4. Replace the header and notices together, preserving upstream attribution.
   Re-extract the applicable stb license text and check every embedded component;
   do not assume future releases have the same license inventory.
5. Update the version, resolved commit, checksum and inventory in this file.
   Run `sha256sum third-party/cui/cui_single.h` from the repository root and
   compare it with the recorded value.
6. Run the affected native, binding and FPAS-consumer checks as those integration
   layers become available. Source integrity alone does not establish that a
   replacement version preserves the integration's behavior.
