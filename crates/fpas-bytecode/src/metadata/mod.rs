//! Register executable metadata tables.

mod constants;
mod distinct_types;
mod enums;
mod globals;
mod records;
mod source_map;
mod strings;

pub use constants::Constant;
pub use distinct_types::DistinctTypeName;
pub use enums::{EnumLayout, EnumVariant};
pub use globals::{GlobalInfo, GlobalInitializer};
pub use records::{
    RecordConstructionInfo, RecordField, RecordLayout, RecordMethod, RecordTypeAlias,
};
pub use source_map::{SourceMap, SourceRun};
pub use strings::StringTable;
