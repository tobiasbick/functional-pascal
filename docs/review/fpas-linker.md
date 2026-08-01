# `fpas-linker` review follow-up

Classification: linker correctness and executable validation. No language change expected.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LINK-01 | P1 | `crates/fpas-linker/src/lib.rs:97` | The linker checks only code/location parity, not `validate_executable`. Unknown intrinsics and invalid jump targets can be returned as a successful link. | Validate the completed chunk with the authoritative executable validator and expose a precise `LinkError`. | Unknown intrinsic, out-of-range and one-past-end jumps must fail `link_objects`. |
| LINK-02 | P1 | `crates/fpas-linker/src/lib.rs:188,216` | Unit terminal `Halt` is stripped after validation, but a function entry may point to that stripped instruction. Rebasing then points the function at the next object/program. | For non-root objects require every function entry to be strictly below the retained code length before registration. | Unit with only `Halt` and a function at offset zero; multi-object case proving no rebound. |
| LINK-03 | P2 | `crates/fpas-linker/src/lib.rs:141` | A public callable definition can resolve without a matching function-table implementation, producing an executable that fails name lookup later. | Track the defining object with each definition and require a case-insensitive matching function entry for callable definitions. | Missing function entry, wrong owner/name, case-insensitive valid match, and extra local functions. |
| LINK-04 | P3 | `crates/fpas-linker/src/lib.rs:188,224` | Validated relocation lists are rebuilt into `HashMap<u32, Vec<Relocation>>`, adding hashing and allocations. Impact is unmeasured. | Consume a peekable ordered relocation iterator while copying instructions. | Behavior-equivalence tests and a benchmark before claiming improvement. |

## Implementation notes

Resolve the validation boundary with BYTECODE-04/05 and UNIT-03. The test currently named `missing_private_and_kind_mismatched_imports_are_rejected` covers only private visibility; split it and add the missing kind-mismatch case.
