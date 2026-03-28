use super::super::{Checker, define_func, define_proc, p};
use crate::types::Ty;
use fpas_std::std_symbols as s;

pub fn register_std_channel(c: &mut Checker) {
    // Std.Channel.Make(): channel of T  (type inferred from context)
    define_func(
        c,
        s::STD_CHANNEL_MAKE,
        vec![],
        Ty::Channel(Box::new(Ty::Error)),
    );

    // Std.Channel.MakeBuffered(Size: integer): channel of T
    define_func(
        c,
        s::STD_CHANNEL_MAKE_BUFFERED,
        vec![p("Size", Ty::Integer, false)],
        Ty::Channel(Box::new(Ty::Error)),
    );

    // Std.Channel.Send(Ch: channel of T; Value: T)
    define_proc(
        c,
        s::STD_CHANNEL_SEND,
        vec![
            p("Ch", Ty::Channel(Box::new(Ty::Error)), false),
            p("Value", Ty::Error, false),
        ],
    );

    // Std.Channel.Receive(Ch: channel of T): T
    define_func(
        c,
        s::STD_CHANNEL_RECEIVE,
        vec![p("Ch", Ty::Channel(Box::new(Ty::Error)), false)],
        Ty::Error,
    );

    // Std.Channel.TryReceive(Ch: channel of T): Option of T
    define_func(
        c,
        s::STD_CHANNEL_TRY_RECEIVE,
        vec![p("Ch", Ty::Channel(Box::new(Ty::Error)), false)],
        Ty::Option(Box::new(Ty::Error)),
    );

    // Std.Channel.Close(Ch: channel of T)
    define_proc(
        c,
        s::STD_CHANNEL_CLOSE,
        vec![p("Ch", Ty::Channel(Box::new(Ty::Error)), false)],
    );
}

pub fn register_std_task(c: &mut Checker) {
    // Std.Task.Wait(T: task): T  (return type erased)
    define_func(
        c,
        s::STD_TASK_WAIT,
        vec![p("T", Ty::Task(Box::new(Ty::Error)), false)],
        Ty::Error,
    );

    // Std.Task.WaitAll(Tasks: array of task)
    define_proc(
        c,
        s::STD_TASK_WAIT_ALL,
        vec![p(
            "Tasks",
            Ty::Array(Box::new(Ty::Task(Box::new(Ty::Error)))),
            false,
        )],
    );
}
