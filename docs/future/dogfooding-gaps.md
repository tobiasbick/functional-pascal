# Dogfooding gaps

This document records practical limits encountered when using FPAS for repository
maintenance. Required baseline work and deferred proposals belong in
[language gaps](language-gaps.md). Supported language/library usage belongs in the
[handbook](../pascal/README.md), rather than a list of completed milestones here.

## Maintenance workflow

Write filesystem, text, and data-maintenance helpers in FPAS using actual .NET
APIs. Directory traversal, collections, regex, delegates, generic routines/data,
contracts, nested routines, and Task-based async are available. The
[directory inventory](../../examples/directory-inventory/README.md) demonstrates
static LINQ calls and captured state; [local functions](../../examples/local-functions/README.md)
demonstrates lexical recursion and an escaping delegate.

Use the [development workflow](../build/development.md) for compiler/application
bootstrap and explicit references. Source imports do not acquire dependencies.
Helpers use the current CLR compiler and language; the former VM/compiler or a
different Pascal dialect cannot validate them.

## Current boundaries

- Nested CLR type annotations support arguments on every enclosing/nested
  segment. Generic arithmetic and static abstract/virtual calls use imported
  interface constraints; see [generic types](../pascal/language/functions/generic-routines.md).
- The [time recipe](../pascal/libraries/environment-and-time.md) cannot subtract
  two concrete DateTimeOffset values with `Deadline - Stamp`: binding
  `const Duration: Sys.TimeSpan := Deadline - Stamp;` reports FP3002. Concrete
  imported arithmetic operators are outside the current lookup; use the actual
  instance `Deadline.Subtract(Stamp)` API. CLR comparison operators and operators
  through generic interface constraints retain their documented support.
- Byref-like/span APIs, pointers, ref returns, custom awaiters, expression-tree
  conversion, and async enumeration are outside the supported interoperability
  contract. Choose supported .NET overloads/types where available; record a
  concrete blocked maintenance task before introducing a fallback or extension.
- The CLI provides compilation. Automatic IDE/project loading, LSP/source-query,
  and formatter tooling have separate scope. Portable symbols and XML API
  documentation are available; use compatible .NET tools for debugging/profiling.

Keep scratch sources and reports under .temp-data. A newly demonstrated blocker
needs a small reproducer, the required API/behavior, and its effect on maintenance.
Source-syntax changes still need the user's approval.
