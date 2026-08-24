//! Plain and encrypted byte transports stored behind one FPAS handle.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use rustls::{ClientConnection, StreamOwned};

/// One connected TCP or TLS byte stream.
pub(super) enum Transport {
    Tcp(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Transport {
    /// Wrap a connected plain TCP stream.
    pub(super) fn tcp(stream: TcpStream) -> Self {
        Self::Tcp(stream)
    }

    /// Wrap a connected and verified TLS stream.
    pub(super) fn tls(stream: StreamOwned<ClientConnection, TcpStream>) -> Self {
        Self::Tls(Box::new(stream))
    }

    /// Return the protocol name used in diagnostics.
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Tcp(_) => "TCP",
            Self::Tls(_) => "TLS",
        }
    }

    /// Set read and write timeouts on the underlying socket.
    pub(super) fn set_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        let socket = self.socket();
        socket
            .set_read_timeout(timeout)
            .and_then(|()| socket.set_write_timeout(timeout))
    }

    /// Shut down the underlying socket.
    pub(super) fn shutdown(&mut self) -> io::Result<()> {
        if let Self::Tls(stream) = self {
            stream.conn.send_close_notify();
        }
        self.socket().shutdown(Shutdown::Both).or_else(|error| {
            if error.kind() == io::ErrorKind::NotConnected {
                Ok(())
            } else {
                Err(error)
            }
        })
    }

    fn socket(&self) -> &TcpStream {
        match self {
            Self::Tcp(stream) => stream,
            Self::Tls(stream) => &stream.sock,
        }
    }
}

impl Read for Transport {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(bytes),
            Self::Tls(stream) => stream.read(bytes),
        }
    }
}

impl Write for Transport {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(bytes),
            Self::Tls(stream) => stream.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}
