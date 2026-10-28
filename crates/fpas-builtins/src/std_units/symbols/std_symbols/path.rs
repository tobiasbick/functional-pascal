//! `Std.Path` symbol names and registry group.

std_symbol!(STD_PATH_JOIN = std_path!("Join"));
std_symbol!(STD_PATH_BASE_NAME = std_path!("BaseName"));
std_symbol!(STD_PATH_DIR_NAME = std_path!("DirName"));
std_symbol!(STD_PATH_EXTENSION = std_path!("Extension"));
std_symbol!(STD_PATH_NORMALIZE = std_path!("Normalize"));
std_symbol!(STD_PATH_IS_ABSOLUTE = std_path!("IsAbsolute"));
std_symbol!(STD_PATH_RELATIVE_TO = std_path!("RelativeTo"));
std_symbol!(STD_PATH_CHANGE_EXTENSION = std_path!("ChangeExtension"));

pub(in crate::std_units) const STD_PATH_SYMBOLS: &[&str] = &[
    STD_PATH_JOIN,
    STD_PATH_BASE_NAME,
    STD_PATH_DIR_NAME,
    STD_PATH_EXTENSION,
    STD_PATH_NORMALIZE,
    STD_PATH_IS_ABSOLUTE,
    STD_PATH_RELATIVE_TO,
    STD_PATH_CHANGE_EXTENSION,
];
