# Dogfooding gaps

Record concrete maintenance blockers here with a small reproducer, the required
API/behavior and its effect on the task. Use the [development workflow](../build/development.md)
for FPAS helpers; general library usage belongs in the [handbook](../pascal/README.md).
Language proposals belong in [language gaps](language-gaps.md).

## Current boundaries

If a maintenance task requires byref-like/span APIs, pointers, ref returns,
custom awaiters, expression trees or async enumeration, record the blocked task
and a reproducer before selecting a fallback or extension.
