//! Open-handle metadata, resizing and complete UTF-8 writes.

use super::*;

#[test]
fn size_and_resize_work_in_writable_modes_without_moving_the_position() {
    let fixture = Fixture::new();
    let path = fixture.path("resized");
    let files = Files::new();
    for mode in [1, 2, 3] {
        fs::write(&path, b"abcdef").expect("fixture");
        let handle = files.open(&path, mode).expect("writable handle");
        files.seek(handle, 12, 0).expect("position beyond EOF");
        files.set_length(handle, 3).expect("resize");
        assert_eq!(files.size(handle).expect("size"), 3);
        assert_eq!(files.tell(handle).expect("position preserved"), 12);
        files.set_length(handle, 8).expect("extend");
        assert_eq!(&fs::read(&path).expect("zeros")[3..], &[0; 5]);
        assert_eq!(
            files.tell(handle).expect("extension preserves position"),
            12
        );
        files.close(handle).expect("close");
        assert!(matches!(files.size(handle), Err(Failure::Closed)));
        assert!(matches!(files.set_length(handle, 0), Err(Failure::Closed)));
    }
    let handle = files.open(&path, 0).expect("read-only");
    assert_eq!(files.size(handle).expect("read-only metadata"), 8);
    assert!(matches!(
        files.set_length(handle, 0),
        Err(Failure::Argument("File", _))
    ));
}

#[test]
fn resize_discards_read_ahead_and_preserves_shared_logical_position() {
    let fixture = Fixture::new();
    let path = fixture.path("buffered");
    fs::write(&path, b"one\nABCDE").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3).expect("read-write");
    assert_eq!(
        files.read_line(handle, || false).expect("line"),
        Some("one".into())
    );
    assert_eq!(files.size(handle).expect("size without moving"), 9);
    assert_eq!(files.tell(handle).expect("logical position"), 4);
    assert!(matches!(
        files.set_length(handle, -1),
        Err(Failure::Argument("Length", _))
    ));
    assert_eq!(
        files
            .tell(handle)
            .expect("invalid length preserves position"),
        4
    );
    files
        .set_length(handle, 6)
        .expect("truncate prefetched input");
    assert_eq!(files.read(handle, 10, || false).expect("not stale"), b"AB");
    files.set_length(handle, 10).expect("extend after EOF");
    assert_eq!(files.read(handle, 10, || false).expect("new bytes"), [0; 4]);
    files
        .set_length(handle, 2)
        .expect("truncate below position");
    assert_eq!(files.tell(handle).expect("past EOF"), 10);
    assert!(files.read(handle, 1, || false).expect("EOF").is_empty());
    files
        .write_text(handle, "x", false, || false)
        .expect("write at retained position");
    assert_eq!(fs::read(&path).expect("gap"), b"on\0\0\0\0\0\0\0\0x");
}

#[test]
fn resize_discards_retained_line_prefix_and_file_length_is_not_a_transfer_limit() {
    let fixture = Fixture::new();
    let path = fixture.path("pending");
    fs::write(&path, vec![b'x'; MAX_BYTES + 1]).expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3).expect("read-write");
    assert!(matches!(
        files.read_line(handle, || false),
        Err(Failure::Limit(_))
    ));
    assert_eq!(files.tell(handle).expect("pending prefix"), 0);
    files.set_length(handle, 2).expect("discard pending prefix");
    assert_eq!(
        files.read_line(handle, || false).expect("new EOF"),
        Some("xx".into())
    );
    files
        .set_length(handle, (MAX_BYTES * 2) as i64)
        .expect("length is not capped by transfer size");
    assert_eq!(
        files.size(handle).expect("larger file"),
        (MAX_BYTES * 2) as i64
    );
    assert_eq!(files.tell(handle).expect("same position"), 2);
}

#[test]
fn size_queries_the_open_file_even_after_its_path_is_replaced() {
    let fixture = Fixture::new();
    let path = fixture.path("original");
    fs::write(&path, b"line\nrest").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3).expect("read-write");
    files.read_line(handle, || false).expect("prefetch");
    fs::rename(&path, fixture.path("moved")).expect("move original");
    fs::write(&path, b"replacement").expect("new file at the old path");
    assert_eq!(files.size(handle).expect("original handle metadata"), 9);
    assert_eq!(files.tell(handle).expect("same logical position"), 5);
    let too_long = "x".repeat(MAX_BYTES + 1);
    assert!(matches!(
        files.write_text(handle, &too_long, true, || false),
        Err(Failure::Limit(_))
    ));
    assert_eq!(
        files.read(handle, 4, || false).expect("buffer unchanged"),
        b"rest"
    );
    files.set_length(handle, 2).expect("resize original");
    assert_eq!(
        fs::read(fixture.path("moved")).expect("original resized"),
        b"li"
    );
    assert_eq!(
        fs::read(&path).expect("replacement unaffected"),
        b"replacement"
    );
}

#[test]
fn text_output_is_utf8_without_bom_and_uses_logical_position_and_lf() {
    let fixture = Fixture::new();
    let path = fixture.path("text");
    fs::write(&path, b"one\nABCDE").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3).expect("read-write");
    files.read_line(handle, || false).expect("prefetch");
    files
        .write_text(handle, "é\0", false, || false)
        .expect("UTF-8 and NUL");
    files
        .write_text(handle, "🙂", true, || false)
        .expect("LF line");
    files
        .write_text(handle, "", false, || false)
        .expect("empty text");
    assert_eq!(files.tell(handle).expect("byte position"), 12);
    files
        .write_text(handle, "", true, || false)
        .expect("blank line");
    assert_eq!(
        fs::read(&path).expect("exact bytes"),
        "one\né\0🙂\n\n".as_bytes()
    );
    files.close(handle).expect("close");
    let append = files.open(&path, 2).expect("append");
    files.seek(append, 0, 0).expect("seek append");
    files
        .write_text(append, "tail", true, || false)
        .expect("append line");
    assert_eq!(
        fs::read(&path).expect("appended"),
        "one\né\0🙂\n\ntail\n".as_bytes()
    );
}

#[test]
fn text_limits_count_utf8_content_and_allow_the_added_lf() {
    let fixture = Fixture::new();
    let path = fixture.path("limit");
    let files = Files::new();
    let handle = files.open(&path, 1).expect("create");
    let exact = "é".repeat(MAX_BYTES / 2);
    files
        .write_text(handle, &exact, true, || false)
        .expect("content boundary plus LF");
    assert_eq!(files.size(handle).expect("size"), MAX_BYTES as i64 + 1);
    for newline in [false, true] {
        assert!(matches!(
            files.write_text(handle, &(exact.clone() + "x"), newline, || false),
            Err(Failure::Limit(_))
        ));
        assert_eq!(
            files.tell(handle).expect("invalid text preserves position"),
            MAX_BYTES as i64 + 1
        );
    }
    assert_eq!(
        fs::read(&path).expect("preserved bytes"),
        (exact + "\n").as_bytes()
    );
}

#[test]
fn text_cancellation_is_checked_between_chunks_and_before_the_added_lf() {
    let fixture = Fixture::new();
    let path = fixture.path("cancel-text");
    fs::write(&path, b"line\nrest").expect("fixture");
    let files = Files::new();
    let handle = files.open(&path, 3).expect("read-write");
    files.read_line(handle, || false).expect("prefetch");
    assert!(matches!(
        files.write_text(handle, "lost", true, || true),
        Err(Failure::Cancelled)
    ));
    assert_eq!(files.tell(handle).expect("no transfer"), 5);
    assert_eq!(
        files.read(handle, 4, || false).expect("buffer retained"),
        b"rest"
    );
    files.close(handle).expect("close");
    let handle = files.open(&path, 1).expect("create");
    let checks = Cell::new(0);
    assert!(matches!(
        files.write_text(handle, &"x".repeat(20_000), true, || {
            checks.set(checks.get() + 1);
            checks.get() >= 3
        }),
        Err(Failure::Cancelled)
    ));
    assert_eq!(files.tell(handle).expect("partial progress"), 8192);
    assert_eq!(fs::read(&path).expect("partial output"), vec![b'x'; 8192]);
    files.set_length(handle, 0).expect("truncate");
    files.seek(handle, 0, 0).expect("rewind");
    checks.set(0);
    assert!(matches!(
        files.write_text(handle, "text", true, || {
            checks.set(checks.get() + 1);
            checks.get() >= 3
        }),
        Err(Failure::Cancelled)
    ));
    assert_eq!(fs::read(&path).expect("cancel before LF"), b"text");
}

#[test]
fn complete_lines_are_serialized_across_tasks_sharing_a_handle() {
    let fixture = Fixture::new();
    let path = fixture.path("serialized-lines");
    let files = Arc::new(Files::new());
    let handle = files.open(&path, 1).expect("create");
    std::thread::scope(|scope| {
        for text in ["a", "b", "c", "d"] {
            let files = &files;
            scope.spawn(move || {
                for _ in 0..4 {
                    files
                        .write_text(handle, &text.repeat(20_000), true, || false)
                        .expect("whole line");
                }
            });
        }
    });
    files.close(handle).expect("close");
    let text = fs::read_to_string(&path).expect("text");
    let lines: Vec<_> = text.split_terminator('\n').collect();
    assert_eq!(lines.len(), 16);
    for line in lines {
        assert_eq!(line.len(), 20_000);
        assert!(line.bytes().all(|byte| byte == line.as_bytes()[0]));
    }
}
