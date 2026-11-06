# Dogfooding gaps

Record concrete maintenance blockers here with a small reproducer, the required
API/behavior and its effect on the task. Use the [development workflow](../build/development.md)
for FPAS helpers; general library usage belongs in the [handbook](../pascal/README.md).
Language proposals belong in [language gaps](language-gaps.md).

## Current boundaries

The [time recipe](../pascal/libraries/environment-and-time.md) cannot subtract
concrete DateTimeOffset values with `Deadline - Stamp`:
`const Duration: Sys.TimeSpan := Deadline - Stamp;` reports FP3002.
Use the actual instance API `Deadline.Subtract(Stamp)`. Concrete imported arithmetic
operator lookup is outside the selected interoperability scope; extending it
needs a separate decision.

If a maintenance task requires byref-like/span APIs, pointers, ref returns,
custom awaiters, expression trees or async enumeration, record the blocked task
and a reproducer before selecting a fallback or extension. Additional editor
services and integration remain in [editor gaps](editor-gap.md).
