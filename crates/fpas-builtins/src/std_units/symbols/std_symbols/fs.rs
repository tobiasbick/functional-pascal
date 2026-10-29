//! `Std.Io.Fs` symbol names and registry group.

std_symbol!(STD_FS_FILE = std_fs!("File"));
std_symbol!(STD_FS_FILE_MODE = std_fs!("FileMode"));
std_symbol!(STD_FS_SEEK_ORIGIN = std_fs!("SeekOrigin"));
std_symbol!(STD_FS_OPEN = std_fs!("Open"));
std_symbol!(STD_FS_CLOSE = std_fs!("Close"));
std_symbol!(STD_FS_READ = std_fs!("Read"));
std_symbol!(STD_FS_WRITE = std_fs!("Write"));
std_symbol!(STD_FS_READ_LINE = std_fs!("ReadLine"));
std_symbol!(STD_FS_SEEK = std_fs!("Seek"));
std_symbol!(STD_FS_TELL = std_fs!("Tell"));
std_symbol!(STD_FS_FLUSH = std_fs!("Flush"));
std_symbol!(STD_FS_SIZE = std_fs!("Size"));
std_symbol!(STD_FS_SET_LENGTH = std_fs!("SetLength"));
std_symbol!(STD_FS_WRITE_STRING = std_fs!("WriteString"));
std_symbol!(STD_FS_WRITE_LINE = std_fs!("WriteLine"));
std_symbol!(STD_FS_WRITE_STRING_WITH_CANCELLATION = std_fs!("WriteStringWithCancellation"));
std_symbol!(STD_FS_WRITE_LINE_WITH_CANCELLATION = std_fs!("WriteLineWithCancellation"));
std_symbol!(STD_FS_READ_WITH_CANCELLATION = std_fs!("ReadWithCancellation"));
std_symbol!(STD_FS_WRITE_WITH_CANCELLATION = std_fs!("WriteWithCancellation"));
std_symbol!(STD_FS_READ_LINE_WITH_CANCELLATION = std_fs!("ReadLineWithCancellation"));

std_symbol!(STD_FS_READ_BYTES = std_fs!("ReadBytes"));
std_symbol!(STD_FS_WRITE_BYTES = std_fs!("WriteBytes"));

std_symbol!(STD_FS_READ_TEXT = std_fs!("ReadText"));
std_symbol!(STD_FS_WRITE_TEXT = std_fs!("WriteText"));
std_symbol!(STD_FS_WRITE_TEXT_ATOMIC = std_fs!("WriteTextAtomic"));
std_symbol!(STD_FS_DELETE_FILE = std_fs!("DeleteFile"));
std_symbol!(STD_FS_DELETE_DIR = std_fs!("DeleteDir"));
std_symbol!(STD_FS_DELETE_DIR_ALL = std_fs!("DeleteDirAll"));
std_symbol!(STD_FS_RENAME = std_fs!("Rename"));
std_symbol!(STD_FS_COPY_FILE = std_fs!("CopyFile"));
std_symbol!(STD_FS_APPEND_TEXT = std_fs!("AppendText"));
std_symbol!(STD_FS_FILE_SIZE = std_fs!("FileSize"));
std_symbol!(STD_FS_MODIFIED_TIME = std_fs!("ModifiedTime"));
std_symbol!(STD_FS_IS_SYM_LINK = std_fs!("IsSymLink"));
std_symbol!(STD_FS_CURRENT_DIR = std_fs!("CurrentDir"));
std_symbol!(STD_FS_SET_CURRENT_DIR = std_fs!("SetCurrentDir"));
std_symbol!(STD_FS_TEMP_DIR = std_fs!("TempDir"));
std_symbol!(STD_FS_CREATE_TEMP_DIR = std_fs!("CreateTempDir"));
std_symbol!(STD_FS_CREATE_TEMP_FILE = std_fs!("CreateTempFile"));
std_symbol!(STD_FS_READ_DIR_ALL = std_fs!("ReadDirAll"));
std_symbol!(STD_FS_COPY_DIR_ALL = std_fs!("CopyDirAll"));
std_symbol!(STD_FS_EXISTS = std_fs!("Exists"));
std_symbol!(STD_FS_IS_FILE = std_fs!("IsFile"));
std_symbol!(STD_FS_IS_DIR = std_fs!("IsDir"));
std_symbol!(STD_FS_CREATE_DIR = std_fs!("CreateDir"));
std_symbol!(STD_FS_CREATE_DIR_ALL = std_fs!("CreateDirAll"));
std_symbol!(STD_FS_READ_DIR = std_fs!("ReadDir"));
std_symbol!(STD_FS_GLOB = std_fs!("Glob"));

pub(in crate::std_units) const STD_FS_SYMBOLS: &[&str] = &[
    STD_FS_FILE,
    STD_FS_FILE_MODE,
    STD_FS_SEEK_ORIGIN,
    STD_FS_OPEN,
    STD_FS_CLOSE,
    STD_FS_READ,
    STD_FS_WRITE,
    STD_FS_READ_LINE,
    STD_FS_SEEK,
    STD_FS_TELL,
    STD_FS_FLUSH,
    STD_FS_SIZE,
    STD_FS_SET_LENGTH,
    STD_FS_WRITE_STRING,
    STD_FS_WRITE_LINE,
    STD_FS_WRITE_STRING_WITH_CANCELLATION,
    STD_FS_WRITE_LINE_WITH_CANCELLATION,
    STD_FS_READ_WITH_CANCELLATION,
    STD_FS_WRITE_WITH_CANCELLATION,
    STD_FS_READ_LINE_WITH_CANCELLATION,
    STD_FS_READ_BYTES,
    STD_FS_WRITE_BYTES,
    STD_FS_READ_TEXT,
    STD_FS_WRITE_TEXT,
    STD_FS_WRITE_TEXT_ATOMIC,
    STD_FS_DELETE_FILE,
    STD_FS_EXISTS,
    STD_FS_IS_FILE,
    STD_FS_IS_DIR,
    STD_FS_CREATE_DIR,
    STD_FS_CREATE_DIR_ALL,
    STD_FS_GLOB,
    STD_FS_READ_DIR,
];
