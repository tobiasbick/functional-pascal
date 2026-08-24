//! `Std.Net` symbol names and registry group.

/// Qualified name of the opaque `Std.Net.Connection` record.
pub const STD_NET_CONNECTION: &str = std_net!("Connection");
std_symbol!(STD_NET_CONNECT = std_net!("Connect"));
std_symbol!(STD_NET_CONNECT_TLS = std_net!("ConnectTls"));
std_symbol!(STD_NET_SET_TIMEOUT = std_net!("SetTimeout"));
std_symbol!(STD_NET_READ = std_net!("Read"));
std_symbol!(STD_NET_WRITE = std_net!("Write"));
std_symbol!(STD_NET_CLOSE = std_net!("Close"));

pub(in crate::std_units) const STD_NET_SYMBOLS: &[&str] = &[
    STD_NET_CONNECTION,
    STD_NET_CONNECT,
    STD_NET_CONNECT_TLS,
    STD_NET_SET_TIMEOUT,
    STD_NET_READ,
    STD_NET_WRITE,
    STD_NET_CLOSE,
];
