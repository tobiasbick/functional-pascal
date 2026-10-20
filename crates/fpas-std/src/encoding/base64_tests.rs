//! RFC 4648 vectors and strict Base64 validation without filesystem access.

use super::base64::{decode, encode};
use super::error::CodecFailure;

#[test]
fn rfc4648_vectors_have_required_standard_padding_and_unpadded_url_output() {
    for (bytes, standard, url) in [
        ("", "", ""),
        ("f", "Zg==", "Zg"),
        ("fo", "Zm8=", "Zm8"),
        ("foo", "Zm9v", "Zm9v"),
        ("foob", "Zm9vYg==", "Zm9vYg"),
        ("fooba", "Zm9vYmE=", "Zm9vYmE"),
        ("foobar", "Zm9vYmFy", "Zm9vYmFy"),
    ] {
        assert_eq!(encode(bytes.as_bytes(), false), standard);
        assert_eq!(encode(bytes.as_bytes(), true), url);
        assert_eq!(decode(standard, false), Ok(bytes.as_bytes().to_vec()));
        assert_eq!(decode(url, true), Ok(bytes.as_bytes().to_vec()));
    }
}

#[test]
fn alphabets_are_distinct_and_binary_bytes_round_trip() {
    assert_eq!(encode(&[251, 255, 255], false), "+///");
    assert_eq!(encode(&[251, 255, 255], true), "-___");
    assert_eq!(decode("+///", false), Ok(vec![251, 255, 255]));
    assert_eq!(decode("-___", true), Ok(vec![251, 255, 255]));
    for (text, url, offset) in [
        ("-___", false, 0),
        ("+///", true, 0),
        ("A+_A", false, 2),
        ("A-_/", true, 3),
    ] {
        assert_eq!(
            decode(text, url),
            Err(CodecFailure::InvalidCharacter { offset })
        );
    }
    let bytes: Vec<u8> = (0..=255).collect();
    for url in [false, true] {
        assert_eq!(decode(&encode(&bytes, url), url), Ok(bytes.clone()));
    }
}

#[test]
fn missing_padding_and_impossible_lengths_are_rejected() {
    for (text, url) in [
        ("Zg", false),
        ("Zm8", false),
        ("A", false),
        ("A", true),
        ("AAAAA", true),
    ] {
        assert_eq!(
            decode(text, url),
            Err(CodecFailure::InvalidLength { length: text.len() })
        );
    }
}

#[test]
fn misplaced_excess_or_url_padding_has_a_byte_offset() {
    for (text, url, offset) in [
        ("AA=A", false, 2),
        ("=AAA", false, 0),
        ("A===", false, 1),
        ("====", false, 0),
        ("AA======", false, 2),
        ("AA==AAAA", false, 2),
        ("Zg==", true, 2),
        ("Zm8=", true, 3),
    ] {
        assert_eq!(
            decode(text, url),
            Err(CodecFailure::InvalidPadding { offset })
        );
    }
}

#[test]
fn unused_sextet_bits_must_be_zero_in_both_variants() {
    for (text, url, offset) in [
        ("Zh==", false, 1),
        ("Zv==", false, 1),
        ("Zm9=", false, 2),
        ("Zm/=", false, 2),
        ("Zh", true, 1),
        ("Zv", true, 1),
        ("Zm9", true, 2),
        ("Zm_", true, 2),
        ("Zm9vZh==", false, 5),
        ("Zm9vZm9", true, 6),
    ] {
        assert_eq!(
            decode(text, url),
            Err(CodecFailure::NonCanonical { offset })
        );
    }
}

#[test]
fn whitespace_and_non_ascii_are_never_ignored_or_folded() {
    for url in [false, true] {
        for (text, offset) in [
            (" AAA", 0),
            ("AA A", 2),
            ("AAA\n", 3),
            ("AA\rA", 2),
            ("AA\tA", 2),
            ("AA\0A", 2),
            ("AAé", 2),
            ("AAAAéAA", 4),
            ("AAＡ", 2),
            ("AA💡AA", 2),
        ] {
            assert_eq!(
                decode(text, url),
                Err(CodecFailure::InvalidCharacter { offset })
            );
        }
    }
}
