use fpas_bytecode::{Op, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

#[test]
fn spawned_task_global_is_visible_to_main_after_wait() {
    let writer = "WriteGlobal";
    let chunk = build_zero_arg_function_chunk(
        writer,
        |main| {
            let idx = main
                .add_constant(Value::Str(("Shared".to_string()).into()))
                .expect("constant");
            emit_constant(main, Value::Integer(99));
            main.emit(Op::SetGlobal(idx), loc());
            main.emit(Op::Pop, loc());

            emit_constant(main, Value::function(writer.to_string(), vec![], false));
            main.emit(Op::SpawnTask(0), loc());
            main.emit(
                Op::Intrinsic(u16::from(fpas_bytecode::Intrinsic::Task(
                    fpas_bytecode::TaskIntrinsic::Wait,
                ))),
                loc(),
            );
            main.emit(Op::Pop, loc());
            main.emit(Op::GetGlobal(idx), loc());
            main.emit(Op::PrintLn, loc());
            main.emit(Op::Halt, loc());
        },
        |body| {
            let idx = body
                .add_constant(Value::Str(("Shared".to_string()).into()))
                .expect("constant");
            emit_constant(body, Value::Integer(42));
            body.emit(Op::SetGlobal(idx), loc());
            body.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}
