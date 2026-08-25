//! Semantic registration for the hosted `Std.Net` network interface.

use super::super::{define_func, p};
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Net` functions made visible by `uses Std.Net`.
pub(super) fn register_std_net(checker: &mut Checker) {
    let connection =
        type_registration::register_record_type(checker, s::STD_NET_CONNECTION, Vec::new());
    let listener =
        type_registration::register_record_type(checker, s::STD_NET_LISTENER, Vec::new());
    let error = Box::new(Ty::String);

    define_func(
        checker,
        s::STD_NET_CONNECT,
        vec![
            p("Host", Ty::String, false),
            p("Port", Ty::Integer, false),
            p("TimeoutMillis", Ty::Integer, false),
        ],
        Ty::Result(Box::new(connection.clone()), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_CONNECT_TLS,
        vec![
            p("Host", Ty::String, false),
            p("Port", Ty::Integer, false),
            p("TimeoutMillis", Ty::Integer, false),
        ],
        Ty::Result(Box::new(connection.clone()), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_LISTEN,
        vec![p("Host", Ty::String, false), p("Port", Ty::Integer, false)],
        Ty::Result(Box::new(listener.clone()), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_LISTEN_TLS,
        vec![
            p("Host", Ty::String, false),
            p("Port", Ty::Integer, false),
            p("CertificatePath", Ty::String, false),
            p("PrivateKeyPath", Ty::String, false),
            p("HandshakeTimeoutMillis", Ty::Integer, false),
        ],
        Ty::Result(Box::new(listener.clone()), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_ACCEPT,
        vec![p("Listener", listener.clone(), false)],
        Ty::Result(Box::new(connection.clone()), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_CLOSE_LISTENER,
        vec![p("Listener", listener, false)],
        Ty::Result(Box::new(Ty::Boolean), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_SET_TIMEOUT,
        vec![
            p("Connection", connection.clone(), false),
            p("TimeoutMillis", Ty::Integer, false),
        ],
        Ty::Result(Box::new(Ty::Boolean), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_READ,
        vec![
            p("Connection", connection.clone(), false),
            p("MaxBytes", Ty::Integer, false),
        ],
        Ty::Result(Box::new(Ty::Array(Box::new(Ty::Integer))), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_WRITE,
        vec![
            p("Connection", connection.clone(), false),
            p("Data", Ty::Array(Box::new(Ty::Integer)), false),
        ],
        Ty::Result(Box::new(Ty::Integer), error.clone()),
    );
    define_func(
        checker,
        s::STD_NET_CLOSE,
        vec![p("Connection", connection, false)],
        Ty::Result(Box::new(Ty::Boolean), error),
    );
}
