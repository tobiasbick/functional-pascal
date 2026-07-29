//! Protocol conversion kept separate from request and notification handling.

mod position;
mod uri;

pub use position::{PositionConversionError, byte_offset_to_position, position_to_byte_offset};
pub use uri::{FileUriError, file_uri_to_path};
