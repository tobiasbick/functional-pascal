use super::*;
use fpas_std::last_headless_graph_frame_for_tests;

#[test]
fn graph_clear_put_pixel_and_present_render_runtime_backbuffer() {
    with_headless(|| {
        let mut chunk = Chunk::new();
        emit_constant(&mut chunk, Value::Integer(2));
        emit_constant(&mut chunk, Value::Integer(1));
        emit_constant(&mut chunk, Value::Str(("Graph draw".to_string()).into()));
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
        emit_constant(&mut chunk, Value::Str(("Graph line".to_string()).into()));
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
        emit_constant(&mut chunk, Value::Str(("Graph rect".to_string()).into()));
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
        emit_constant(&mut chunk, Value::Str(("Graph fill".to_string()).into()));
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
        emit_constant(&mut chunk, Value::Str(("Graph circle".to_string()).into()));
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
        emit_constant(&mut chunk, Value::Str(("Graph text".to_string()).into()));
        emit_graph_intrinsic(&mut chunk, GraphIntrinsic::ApplicationOpen);

        chunk.emit(Op::Dup, loc());
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Integer(0));
        emit_constant(&mut chunk, Value::Str(("A".to_string()).into()));
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
