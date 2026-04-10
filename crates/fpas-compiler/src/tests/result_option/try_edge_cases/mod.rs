//! Edge cases and negative tests for the `try` operator.
//!
//! Spec: [`docs/pascal/07-error-handling.md`](../../../../../../docs/pascal/07-error-handling.md).

mod error_content_preservation_through_try;
mod multiple_try_in_one_function;
mod negative_try_on_non_result_option;
mod nested_try_expressions;
mod process_example_from_docs;
mod try_in_expression_context;
mod try_in_program_main_block;
mod try_with_option_first_positive_example_from_docs;
