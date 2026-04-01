# Examples

This directory contains runnable Functional Pascal examples for the current language implementation.

## Single-file programs

- `hello.fpas` - minimal program with `program`, `uses`, and `begin ... end.`
- `fibonacci.fpas` - recursion and `for` loops
- `pascal/higher-order-functions/higher_order_functions.fpas` - named first-class functions and higher-order array helpers
- `pascal/enum-data/` - enums with associated data and destructuring via `case`
- `pascal/error-handling/` - `Result`, `Option`, `panic`, and `try`
- `pascal/for/` and `pascal/for-in/` - counting loops and collection iteration
- `pascal/generics/generic_functions.fpas` - generic functions over scalars and arrays
- `pascal/pattern-matching/` - guards and exhaustiveness
- `pascal/record-methods/` - record methods and dot-call syntax
- `pascal/records/defaults_with_update.fpas` - record default fields and immutable `with` updates

## Multi-file project

- `pascal/units-basic/` - `.fpasprj`, `unit` declarations, `uses`, and qualified calls across files

## Run

Single source file:

```sh
fpas examples/hello.fpas
```

Project:

```sh
fpas examples/pascal/units-basic/units-basic.fpasprj
```