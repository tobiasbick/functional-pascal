//! Shared resource limits for register executable construction and validation.

/// Maximum cumulative UTF-8 bytes in the register string table.
pub const MAX_STRING_BYTES: usize = 64 * 1024 * 1024;
/// Maximum strings in one executable.
pub const MAX_STRINGS: usize = 1_000_000;
/// Maximum persistent constants in one executable.
pub const MAX_CONSTANTS: usize = 1_000_000;
/// Maximum global slots in one executable.
pub const MAX_GLOBALS: usize = 1_000_000;
/// Maximum functions addressable by a 16-bit [`crate::FunctionId`].
pub const MAX_FUNCTIONS: usize = 65_536;
/// Maximum record layouts addressable by a 16-bit [`crate::RecordTypeId`].
pub const MAX_RECORD_LAYOUTS: usize = 65_536;
/// Maximum enum layouts addressable by a 16-bit [`crate::EnumTypeId`].
pub const MAX_ENUM_LAYOUTS: usize = 65_536;
/// Maximum executable-wide enum variants.
pub const MAX_ENUM_VARIANTS: usize = 65_536;
/// Maximum fields in one record or enum variant.
pub const MAX_LAYOUT_FIELDS: usize = 65_535;
/// Maximum instructions in one executable.
pub const MAX_INSTRUCTIONS: usize = 16_000_000;
/// Maximum instructions in one function.
pub const MAX_FUNCTION_INSTRUCTIONS: usize = 4_000_000;
/// Maximum sparse source-map runs.
pub const MAX_SOURCE_RUNS: usize = 4_000_000;
/// Maximum source paths in one executable.
pub const MAX_SOURCE_PATHS: usize = 1_000_000;
/// Maximum debugger sequence points in one executable.
pub const MAX_DEBUG_SEQUENCE_POINTS: usize = 4_000_000;
/// Maximum source-visible debugger bindings in one executable.
pub const MAX_DEBUG_BINDINGS: usize = 1_000_000;
/// Maximum lexical debugger scopes in one executable.
pub const MAX_DEBUG_SCOPES: usize = 1_000_000;
/// Maximum addressable registers in one function; the final `u16` value is a sentinel.
pub const MAX_REGISTERS_PER_FUNCTION: usize = 65_535;
/// Maximum arguments in an instruction's auxiliary count field.
pub const MAX_CALL_ARGUMENTS: usize = 255;
/// Maximum captures in the ABC auxiliary count used by closure construction.
pub const MAX_CLOSURE_CAPTURES: usize = 255;
/// Maximum linked units reserved for the later persistent executable codec.
pub const MAX_LINKED_UNITS: usize = 1_000_000;
/// Maximum bytes in an identity string reserved for the later persistent executable codec.
pub const MAX_IDENTITY_STRING_BYTES: usize = 1024 * 1024;
/// Maximum sections reserved for the later persistent executable codec.
pub const MAX_SECTIONS: usize = 64;
/// Maximum payload bytes reserved for the later persistent executable codec.
pub const MAX_PAYLOAD_BYTES: usize = 512 * 1024 * 1024;
