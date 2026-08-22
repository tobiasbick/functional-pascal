//! `Std.Path` symbol names and registry group.

std_symbol!(STD_PATH_JOIN = std_path!("Join"));
std_symbol!(STD_PATH_BASE_NAME = std_path!("BaseName"));
std_symbol!(STD_PATH_DIR_NAME = std_path!("DirName"));
std_symbol!(STD_PATH_EXTENSION = std_path!("Extension"));
std_symbol!(STD_PATH_NORMALIZE = std_path!("Normalize"));

pub(in crate::std_units) const STD_PATH_SYMBOLS: &[&str] = &[
    STD_PATH_JOIN,
    STD_PATH_BASE_NAME,
    STD_PATH_DIR_NAME,
    STD_PATH_EXTENSION,
    STD_PATH_NORMALIZE,
];
