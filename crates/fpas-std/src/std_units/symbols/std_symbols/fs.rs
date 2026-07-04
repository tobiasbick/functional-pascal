//! `Std.Fs` symbol names and registry group.

pub const STD_FS_READ_TEXT: &str = std_fs!("ReadText");
pub const STD_FS_WRITE_TEXT: &str = std_fs!("WriteText");
pub const STD_FS_EXISTS: &str = std_fs!("Exists");
pub const STD_FS_IS_FILE: &str = std_fs!("IsFile");
pub const STD_FS_IS_DIR: &str = std_fs!("IsDir");
pub const STD_FS_CREATE_DIR: &str = std_fs!("CreateDir");

pub(in crate::std_units) const STD_FS_SYMBOLS: &[&str] = &[
    STD_FS_READ_TEXT,
    STD_FS_WRITE_TEXT,
    STD_FS_EXISTS,
    STD_FS_IS_FILE,
    STD_FS_IS_DIR,
    STD_FS_CREATE_DIR,
];
