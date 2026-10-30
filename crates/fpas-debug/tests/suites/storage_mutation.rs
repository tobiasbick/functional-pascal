//! Atomic debugger mutations, storage ownership and inspection generations.

#[path = "../support/parameter_copy.rs"]
mod parameter_copy;

#[path = "../capturing_routine_assignment.rs"]
mod capturing_routine_assignment;
#[path = "../cell_capturing_routine_assignment.rs"]
mod cell_capturing_routine_assignment;
#[path = "../dictionary_mutation.rs"]
mod dictionary_mutation;
#[path = "../empty_storage_construction.rs"]
mod empty_storage_construction;
#[path = "../expression_mutation.rs"]
mod expression_mutation;
#[path = "../function_value_assignment.rs"]
mod function_value_assignment;
#[path = "../payload_mutation.rs"]
mod payload_mutation;
#[path = "../sequence_mutation.rs"]
mod sequence_mutation;
#[path = "../sequence_task_mutation.rs"]
mod sequence_task_mutation;
#[path = "../session_sequence_mutation.rs"]
mod session_sequence_mutation;
#[path = "../task_handle_assignment.rs"]
mod task_handle_assignment;
#[path = "../uninitialized_assignment.rs"]
mod uninitialized_assignment;
#[path = "../variable_mutation.rs"]
mod variable_mutation;
#[path = "../variant_construction.rs"]
mod variant_construction;
#[path = "../variant_replacement.rs"]
mod variant_replacement;
#[path = "../variant_transition.rs"]
mod variant_transition;
