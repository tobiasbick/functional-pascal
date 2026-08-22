//! `Std.Fs` symbol names and registry group.

std_symbol!(STD_FS_READ_TEXT = std_fs!("ReadText"));
std_symbol!(STD_FS_WRITE_TEXT = std_fs!("WriteText"));
std_symbol!(STD_FS_WRITE_TEXT_ATOMIC = std_fs!("WriteTextAtomic"));
std_symbol!(STD_FS_EXISTS = std_fs!("Exists"));
std_symbol!(STD_FS_IS_FILE = std_fs!("IsFile"));
std_symbol!(STD_FS_IS_DIR = std_fs!("IsDir"));
std_symbol!(STD_FS_CREATE_DIR = std_fs!("CreateDir"));
std_symbol!(STD_FS_GLOB = std_fs!("Glob"));

pub(in crate::std_units) const STD_FS_SYMBOLS: &[&str] = &[
    STD_FS_READ_TEXT,
    STD_FS_WRITE_TEXT,
    STD_FS_WRITE_TEXT_ATOMIC,
    STD_FS_EXISTS,
    STD_FS_IS_FILE,
    STD_FS_IS_DIR,
    STD_FS_CREATE_DIR,
    STD_FS_GLOB,
];
