//! `Std.Path` symbol names and registry group.

pub const STD_PATH_JOIN: &str = std_path!("Join");
pub const STD_PATH_BASE_NAME: &str = std_path!("BaseName");
pub const STD_PATH_DIR_NAME: &str = std_path!("DirName");
pub const STD_PATH_EXTENSION: &str = std_path!("Extension");
pub const STD_PATH_NORMALIZE: &str = std_path!("Normalize");

pub(in crate::std_units) const STD_PATH_SYMBOLS: &[&str] = &[
    STD_PATH_JOIN,
    STD_PATH_BASE_NAME,
    STD_PATH_DIR_NAME,
    STD_PATH_EXTENSION,
    STD_PATH_NORMALIZE,
];
