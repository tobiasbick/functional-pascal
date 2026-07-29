#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep source-state assertions focused"
)]

mod support;

use std::sync::Arc;

use fpas_language_service::{
    DocumentStore, LanguageServiceError, LineIndex, SourceVersion, TextPosition,
};
use support::TempDirectory;

#[test]
fn line_index_handles_empty_crlf_unicode_and_trailing_line() {
    let empty = LineIndex::new("");
    assert_eq!(empty.line_count(), 1);
    assert_eq!(empty.line_range("", 0), Some(0..0));

    let source = "α\r\n🙂x\n";
    let index = LineIndex::new(source);
    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_range(source, 0), Some(0.."α".len()));
    assert_eq!(index.line_range(source, 1), Some(4..9));
    assert_eq!(
        index.line_range(source, 2),
        Some(source.len()..source.len())
    );

    let emoji_offset = source.find('🙂').expect("emoji offset");
    assert_eq!(
        index.position(source, emoji_offset),
        Some(TextPosition {
            line: 1,
            byte_column: 0
        })
    );
    assert_eq!(
        index.offset(
            source,
            TextPosition {
                line: 1,
                byte_column: "🙂".len()
            }
        ),
        Some(emoji_offset + "🙂".len())
    );
    assert_eq!(
        index.offset(
            source,
            TextPosition {
                line: 1,
                byte_column: 1
            }
        ),
        None
    );
}

#[test]
fn open_apply_close_and_reopen_enforce_versions_and_disk_fallback() {
    let temp = TempDirectory::new("document-versions");
    let path = temp.write("source.fpas", "program Disk;\nbegin\nend.\n");
    let mut store = DocumentStore::new();

    let disk = store.snapshot(&path).expect("disk snapshot");
    let disk_again = store.snapshot(&path).expect("cached disk snapshot");
    assert!(Arc::ptr_eq(&disk, &disk_again));
    assert!(matches!(disk.version(), SourceVersion::Disk(_)));

    let opened = store
        .open_document(&path, 7, "program Open;\nbegin\nend.\n")
        .expect("open snapshot");
    assert_eq!(opened.version(), SourceVersion::Editor(7));
    assert!(store.is_open(&path));
    assert_eq!(
        store.snapshot(&path).expect("overlay snapshot").source(),
        "program Open;\nbegin\nend.\n"
    );

    let stale = store
        .apply_full_text(&path, 7, "program Stale;\nbegin\nend.\n")
        .expect_err("same version must be rejected");
    assert!(matches!(
        stale,
        LanguageServiceError::StaleDocumentVersion {
            current: 7,
            received: 7,
            ..
        }
    ));

    let changed = store
        .apply_full_text(&path, 8, "program Changed;\nbegin\nend.\n")
        .expect("newer version");
    assert!(!Arc::ptr_eq(&opened, &changed));
    store.close_document(&path).expect("open snapshot removed");
    assert_eq!(
        store.snapshot(&path).expect("disk restored").source(),
        disk.source()
    );
    let reopened = store
        .open_document(&path, 9, "program Reopened;\nbegin\nend.\n")
        .expect("newer reopened version");
    assert_eq!(reopened.version(), SourceVersion::Editor(9));
    assert_eq!(reopened.source(), "program Reopened;\nbegin\nend.\n");
}

#[test]
fn deleted_disk_file_returns_error_and_recreated_file_gets_new_revision() {
    let temp = TempDirectory::new("document-delete");
    let path = temp.write("source.fpas", "program First;\nbegin\nend.\n");
    let mut store = DocumentStore::new();
    let first = store.snapshot(&path).expect("first disk snapshot");

    std::fs::remove_file(&path).expect("fixture source removed");
    let missing = store.snapshot(&path).expect_err("deleted source must fail");
    assert!(matches!(missing, LanguageServiceError::SourceRead { .. }));

    std::fs::write(&path, "program Second;\nbegin\nend.\n").expect("source recreated");
    let second = store.snapshot(&path).expect("recreated disk snapshot");
    assert_ne!(first.version(), second.version());
    assert_eq!(second.source(), "program Second;\nbegin\nend.\n");
}

#[test]
fn empty_open_document_produces_a_recoverable_snapshot() {
    let temp = TempDirectory::new("document-empty");
    let path = temp.join("empty.fpas");
    let mut store = DocumentStore::new();

    let snapshot = store
        .open_document(&path, 1, "")
        .expect("empty editor buffer snapshot");

    assert_eq!(snapshot.source(), "");
    assert_eq!(snapshot.line_index().line_count(), 1);
    assert!(snapshot.has_parse_errors());
}

#[test]
fn newly_saved_file_keeps_its_existing_editor_overlay() {
    let temp = TempDirectory::new("document-first-save");
    let path = temp.join("new.fpas");
    let overlay = "program Unsaved;\nbegin\nend.\n";
    let mut store = DocumentStore::new();
    store
        .open_document(&path, 1, overlay)
        .expect("new unsaved buffer");

    std::fs::write(&path, "program Disk;\nbegin\nend.\n").expect("first disk save");

    assert_eq!(
        store
            .snapshot(&path)
            .expect("overlay after first save")
            .source(),
        overlay
    );
}
