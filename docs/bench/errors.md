# Typed error workloads

The `errors` group measures production error values with matched controls in the
same release build. New language features have no invented old-checkout baseline.
The expanded full-suite baseline must be captured on a controlled local host and
saved again when IDs or arguments change. Cloud runs establish correctness only.

| Rows | Boundary and checks | Same-build control |
| --- | --- | --- |
| `errors_enum_construction`, `errors_typed_construction` | 100000 warmed constructions of a leaf and wrapper, followed by typed payload extraction; no rendering; sum 1..N | Ordinary enum leaf/wrapper with equivalent extraction |
| `errors_generic_direct`, `errors_generic_helper`, `errors_generic_callable` | 100000 constructions alternating two distinct cause families with identical integer payloads; select callables once, use one common loop and check each concrete cause | Direct concrete constructor values versus the generic helper and its constructor callable |
| `errors_map_ok`, `errors_case_ok` | 1000000 Ok values, exact success sum; function mapper counter stays zero | Equivalent explicit case |
| `errors_map_error`, `errors_case_error`, `errors_constructor_error` | 1000000 Error values, exact typed cause/payload sum; function mapper called N times | Explicit case plus function versus the native constructor callable |
| `errors_cause_message`, `errors_cause_console` | 1000 renders at depth 16 after cause construction and an independent golden check | Message versus Console through the real in-memory VM host |
| `errors_deep_image` | Eight decoded images, each constructing, walking, rendering and dropping a depth-512 cause chain with terminal code 7 | Structural acceptance, without a fragile elapsed-time assertion |

Construction timers include the loop, calls, Result/enum handling, checksums and
payload extraction. They exclude CLI compilation and the ten-iteration warmup.
Generic results describe the helper/callable path relative to the direct callable
control, including the hidden-constructor binding ABI; they are not isolated
native-call nanoseconds. The counter records ordinary function mapper calls.
The constructor row checks the typed payload without instrumenting constructors.

Rendering timers exclude parsing, compilation, image admission, construction of
the chain and the golden check. Message consumes each rendered string's length;
Console includes its in-memory capture and screen bookkeeping. The native driver
checks every captured Console value against the independent complete golden
outside the timer. It writes no rendered values to a terminal or host writer.
Both paths use the production renderer. These rows do not isolate allocation or
terminal latency.

The deep row excludes compilation, initial encoding and one checked warmup.
Its timer includes production image decoding/admission, VM execution and dropping
the decoded VM. Optional enum debugger types are stripped; essential Cause and
identity metadata remain. Source patterns walk all 512 links and verify the
terminal code after every decode. The small render golden, full known text and
output size are also checked. This is a bounded structural test with an elapsed
value for tracking, not a wall-clock correctness gate.

```sh
cargo bench-fpas save errors-baseline --group errors
cargo bench-fpas compare errors-baseline --group errors
cargo bench-fpas native error-values 1 metadata
```

The metadata diagnostic compiles the exact construction fixture and verifies two
concrete wrapper instances and two shared constructor factories. It reports the
16-byte Value, total factory instruction bytes, serialized error identity/variant
descriptions (JSON, excluding general tables and debugger data), and the complete
portable image size. JSON size is a representation measurement, not heap usage.
There is no per-instruction error polling or automatic stack capture.
