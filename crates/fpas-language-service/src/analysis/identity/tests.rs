//! Independent vectors for the portable byte identity, independent of platform JSON encoding.
use super::{encoding::CanonicalHasher, source_digest};

#[test]
fn sha256_and_framing_match_external_byte_vectors() {
    assert_eq!(
        source_digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        source_digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let mut hash = CanonicalHasher::new("a");
    hash.text("bc");
    let framed = [
        1, 0, 0, 0, 0, 0, 0, 0, b'a', 2, 0, 0, 0, 0, 0, 0, 0, b'b', b'c',
    ];
    assert_eq!(hash.finish(), source_digest(&framed));
    let mut left = CanonicalHasher::new("same-domain");
    left.text("a");
    left.text("bc");
    let mut right = CanonicalHasher::new("same-domain");
    right.text("ab");
    right.text("c");
    assert_ne!(left.finish(), right.finish());
}

#[cfg(windows)]
#[test]
fn identity_paths_normalize_verbatim_prefixes_without_changing_case() {
    use std::path::Path;
    assert_eq!(
        super::identity_path(Path::new(r"\\?\C:\Tree\Main.fpas")).expect("drive path"),
        "C:/Tree/Main.fpas"
    );
    assert_eq!(
        super::identity_path(Path::new(r"\\?\UNC\host\share\Main.fpas")).expect("UNC path"),
        "//host/share/Main.fpas"
    );
}

#[cfg(unix)]
#[test]
fn identity_paths_preserve_literal_backslashes_and_reject_lossy_encoding() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
    assert_eq!(
        super::identity_path(Path::new(r"/tmp/a\b")).expect("literal path"),
        r"/tmp/a\b"
    );
    assert!(super::identity_path(Path::new(OsStr::from_bytes(b"/tmp/\xff"))).is_err());
}
