//! Register executable metadata tables.

mod constants;
mod enums;
mod globals;
mod records;
mod source_map;
mod strings;

pub use constants::Constant;
pub use enums::{EnumLayout, EnumVariant};
pub use globals::GlobalInfo;
pub use records::{RecordField, RecordLayout};
pub use source_map::{SourceMap, SourceRun};
pub use strings::StringTable;
