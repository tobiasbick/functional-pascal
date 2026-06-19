use crate::tests::helpers::{
    emit_constant, graph_application_value, graph_size_value, loc, minimal_shared_state,
};
use crate::vm::Worker;
use fpas_bytecode::{Chunk, GraphIntrinsic, Intrinsic, Op, Value};
use fpas_std::{last_headless_graph_frame_for_tests, with_headless_graph_backend_for_tests};
use std::sync::Arc;

fn emit_graph_intrinsic(chunk: &mut Chunk, intrinsic: GraphIntrinsic) {
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Graph(intrinsic))), loc());
}

fn with_headless<T>(f: impl FnOnce() -> T) -> T {
    with_headless_graph_backend_for_tests(f)
}

#[test]
fn graph_open_pushes_application_record_and_opens_session() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(640));
        emit_constant(&mut chunk, Value::Integer(480));
        emit_constant(&mut chunk, Value::Str("Graph smoke".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(Arc::clone(&shared));
        worker.run().expect("graph open should succeed");

        assert_eq!(worker.stack, vec![graph_application_value()]);

        let size = shared
            .graph
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .size(loc())
            .expect("session should be open after graph open");
        assert_eq!(size, (640, 480));
    });
}

#[test]
fn graph_size_pushes_std_graph_size_record() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(320));
        emit_constant(&mut chunk, Value::Integer(200));
        emit_constant(&mut chunk, Value::Str("Graph size".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);
        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationSize);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph size should succeed");

        assert_eq!(
            worker.stack,
            vec![graph_application_value(), graph_size_value(320, 200)]
        );
    });
}

#[test]
fn graph_close_resets_the_active_session() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(128));
        emit_constant(&mut chunk, Value::Integer(96));
        emit_constant(&mut chunk, Value::Str("Graph close".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);
        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationClose);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(Arc::clone(&shared));
        worker.run().expect("graph close should succeed");

        let error = shared
            .graph
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .size(loc())
            .expect_err("session should be closed after graph close");
        assert!(error.message.contains("requires an open graphics session"));
    });
}

#[test]
fn graph_upload_frame_stages_the_validated_frame() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, graph_application_value());
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(
            &mut chunk,
            Value::Array(vec![Value::Integer(0x00102030), Value::Integer(0x00040506)]),
        );
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationUploadFrame);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        {
            let mut graph = shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            graph
                .session
                .open(2, 1, "Graph upload", loc())
                .expect("test graph session should open");
        }

        let mut worker = Worker::new_main(Arc::clone(&shared));
        worker.run().expect("graph upload should succeed");

        let graph = shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        let staged = graph
            .session
            .uploaded_frame()
            .expect("upload should stage a validated frame");
        assert_eq!(staged.width(), 2);
        assert_eq!(staged.height(), 1);
        assert_eq!(staged.pixels(), &[0x00102030, 0x00040506]);
    });
}

#[test]
fn graph_clear_put_pixel_and_present_render_runtime_backbuffer() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Str("Graph draw".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(0x00010203));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationClear);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Integer(0x00ABCDEF));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPutPixel);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker
            .run()
            .expect("graph drawing intrinsics should succeed");

        assert_eq!(worker.stack, vec![graph_application_value()]);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 1);
        assert_eq!(frame.pixels(), &[0x00010203, 0x00ABCDEF]);
    });
}

#[test]
fn graph_draw_line_renders_expected_pixels() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Str("Graph line".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(-1));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(0x00000001));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationDrawLine);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph draw line should succeed");

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 3);
        assert_eq!(frame.height(), 3);
        assert_eq!(
            frame.pixels(),
            &[0, 0, 0, 0x00000001, 0x00000001, 0x00000001, 0, 0, 0]
        );
    });
}

#[test]
fn graph_draw_rect_renders_expected_pixels() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(5));
        emit_constant(&mut chunk, Value::Integer(4));
        emit_constant(&mut chunk, Value::Str("Graph rect".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Integer(0x00000002));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationDrawRect);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph draw rect should succeed");

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 5);
        assert_eq!(frame.height(), 4);
        assert_eq!(
            frame.pixels(),
            &[
                0, 0x00000002, 0x00000002, 0x00000002, 0, 0, 0x00000002, 0, 0x00000002, 0, 0,
                0x00000002, 0x00000002, 0x00000002, 0, 0, 0, 0, 0, 0,
            ]
        );
    });
}

#[test]
fn graph_fill_rect_renders_expected_pixels() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(4));
        emit_constant(&mut chunk, Value::Integer(3));
        emit_constant(&mut chunk, Value::Str("Graph fill".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(0x00000003));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationFillRect);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph fill rect should succeed");

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 4);
        assert_eq!(frame.height(), 3);
        assert_eq!(
            frame.pixels(),
            &[
                0, 0, 0, 0, 0, 0x00000003, 0x00000003, 0, 0, 0x00000003, 0x00000003, 0
            ]
        );
    });
}

#[test]
fn graph_draw_circle_renders_expected_pixels() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(5));
        emit_constant(&mut chunk, Value::Integer(5));
        emit_constant(&mut chunk, Value::Str("Graph circle".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Integer(0x00000004));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationDrawCircle);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph draw circle should succeed");

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 5);
        assert_eq!(frame.height(), 5);
        assert_eq!(
            frame.pixels(),
            &[
                0, 0, 0, 0, 0, 0, 0, 0x00000004, 0, 0, 0, 0x00000004, 0, 0x00000004, 0, 0, 0,
                0x00000004, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
    });
}

#[test]
fn graph_draw_text_renders_expected_pixels() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(5));
        emit_constant(&mut chunk, Value::Integer(7));
        emit_constant(&mut chunk, Value::Str("Graph text".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Str("A".to_string()));
        emit_constant(&mut chunk, Value::Integer(0x00000005));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationDrawText);

        chunk.emit(Op::Dup, loc());
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationPresent);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_main(shared);
        worker.run().expect("graph draw text should succeed");

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 5);
        assert_eq!(frame.height(), 7);
        assert_eq!(
            frame.pixels(),
            &[
                0, 0x00000005, 0x00000005, 0x00000005, 0, 0x00000005, 0, 0, 0, 0x00000005,
                0x00000005, 0, 0, 0, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005,
                0x00000005, 0x00000005, 0, 0, 0, 0x00000005, 0x00000005, 0, 0, 0, 0x00000005,
                0x00000005, 0, 0, 0, 0x00000005,
            ]
        );
    });
}

#[test]
fn graph_intrinsics_fail_on_non_main_tasks() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Str("Graph go".to_string()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);
        chunk.emit(Op::Halt, loc());

        let shared = Arc::new(minimal_shared_state(chunk));
        let mut worker = Worker::new_pool(shared);
        worker.current_task_id = 1;

        let error = worker
            .run()
            .expect_err("graph open in a go task should fail");
        assert!(error.message.contains("must run on the main task"));
    });
}

#[test]
fn graph_open_for_test_opens_headless_session() {
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);

    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(32));
    emit_constant(&mut chunk, Value::Integer(24));
    emit_graph_intrinsic(&mut chunk, GraphIntrinsic::OpenForTest);
    chunk.emit(Op::Dup, loc());
    emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationClose);
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("graph open for test should succeed");

    assert_eq!(worker.stack, vec![graph_application_value()]);
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);

    let size = shared
        .graph
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .size(loc())
        .expect_err("session should be closed");
    assert!(size.message.contains("requires an open graphics session"));
}

#[test]
fn graph_open_for_test_invalid_dimensions_restore_headless_mode() {
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);

    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(24));
    emit_graph_intrinsic(&mut chunk, GraphIntrinsic::OpenForTest);
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    let error = worker
        .run()
        .expect_err("zero width should fail before opening a session");

    assert!(error.message.contains("requires positive Width"));
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);
}

#[test]
fn graph_open_for_test_second_session_error_restores_headless_nesting() {
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);

    let mut open_chunk = Chunk::new();
    emit_constant(&mut open_chunk, Value::Integer(32));
    emit_constant(&mut open_chunk, Value::Integer(24));
    emit_graph_intrinsic(&mut open_chunk, GraphIntrinsic::OpenForTest);
    open_chunk.emit(Op::Halt, loc());

    let mut shared = Arc::new(minimal_shared_state(open_chunk));
    {
        let mut worker = Worker::new_main(Arc::clone(&shared));
        worker.run().expect("first open for test should succeed");
    }
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 1);

    let mut second_open_chunk = Chunk::new();
    emit_constant(&mut second_open_chunk, Value::Integer(16));
    emit_constant(&mut second_open_chunk, Value::Integer(16));
    emit_graph_intrinsic(&mut second_open_chunk, GraphIntrinsic::OpenForTest);
    second_open_chunk.emit(Op::Halt, loc());

    Arc::get_mut(&mut shared)
        .expect("shared state should be unique in this test")
        .chunk = second_open_chunk;
    {
        let mut worker = Worker::new_main(Arc::clone(&shared));
        let error = worker
            .run()
            .expect_err("second open for test should fail while session is active");

        assert!(
            error
                .message
                .contains("cannot open a second graphics session")
        );
    }
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 1);

    let mut close_chunk = Chunk::new();
    emit_constant(&mut close_chunk, graph_application_value());
    emit_graph_intrinsic(&mut close_chunk, GraphIntrinsic::ApplicationClose);
    close_chunk.emit(Op::Halt, loc());

    Arc::get_mut(&mut shared)
        .expect("shared state should be unique in this test")
        .chunk = close_chunk;
    {
        let mut worker = Worker::new_main(shared);
        worker
            .run()
            .expect("close should tear down the first test session");
    }
    assert_eq!(fpas_std::headless_graph_test_depth_for_tests(), 0);
}
