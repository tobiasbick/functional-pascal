//! Tests for `Std.Result.{Map,AndThen,OrElse}` and `Std.Option.{Map,AndThen,OrElse}`.
//!
//! Spec: [`docs/pascal/std/result.md`](../../../../../../docs/pascal/std/result.md),
//! [`docs/pascal/std/option.md`](../../../../../../docs/pascal/std/option.md).

mod chaining;
mod closures_capture_enclosing_variables;
mod combinator_with_named_function;
mod combined_result_option_combinators;
mod compile_time_error_wrong_value_type;
mod deeper_chaining_3_steps_error_short_circuits;
mod identity_map_callback_returns_input_unchanged;
mod map_with_type_transformation_different_input_output_types;
mod nested_result_option;
mod option_and_then;
mod option_map;
mod option_or_else;
mod or_else_callback_returns_error_none;
mod qualified_calls;
mod result_and_then;
mod result_map;
mod result_or_else;
