# Formatter tests

Integration tests share three executables. The original filename is the module
prefix, so filters still select a concern without linking a separate executable.

| Target | Modules |
| --- | --- |
| `formatting` | `automatic_width`, `call_arguments`, `case_block_regressions`, `comment_regressions`, `control_block_regressions`, `declaration_closer_regressions`, `expression_closer_regressions`, `individual_declarations`, `layout_regressions`, `literal_regressions`, `source_layout`, `terminator_regressions` |
| `syntax` | `api_regressions`, `contracts`, `discard`, `logical_precedence`, `public_signatures` |
| `corpus` | `fuzz_light`, `golden_output`, `round_trip` |

Example: `cargo test -p fpas-fmt --test formatting call_arguments`.
Add new test modules to a suite; automatic test discovery is disabled.
