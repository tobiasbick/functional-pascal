//! Plain and encrypted byte transports stored behind one FPAS handle.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use rustls::{ClientConnection, ServerConnection, StreamOwned};

/// One connected TCP or TLS byte stream.
pub(super) enum Transport {
    Tcp(TcpStream),
    TlsClient(Box<StreamOwned<ClientConnection, TcpStream>>),
    TlsServer(Box<StreamOwned<ServerConnection, TcpStream>>),
}

impl Transport {
    /// Wrap a connected plain TCP stream.
    pub(super) fn tcp(stream: TcpStream) -> Self {
        Self::Tcp(stream)
    }

    /// Wrap a connected and verified TLS stream.
    pub(super) fn tls_client(stream: StreamOwned<ClientConnection, TcpStream>) -> Self {
        Self::TlsClient(Box::new(stream))
    }

    /// Wrap an accepted TLS server stream.
    pub(super) fn tls_server(stream: StreamOwned<ServerConnection, TcpStream>) -> Self {
        Self::TlsServer(Box::new(stream))
    }

    /// Return the protocol name used in diagnostics.
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Tcp(_) => "TCP",
            Self::TlsClient(_) | Self::TlsServer(_) => "TLS",
        }
    }

    /// Set read and write timeouts on the underlying socket.
    pub(super) fn set_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        let socket = self.socket();
        socket
            .set_read_timeout(timeout)
            .and_then(|()| socket.set_write_timeout(timeout))
    }

    /// Return the configured read timeout of the transport socket.
    pub(super) fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.socket().read_timeout()
    }

    /// Return the configured write timeout of the transport socket.
    pub(super) fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.socket().write_timeout()
    }

    /// Switch the transport socket between blocking and polling I/O.
    pub(super) fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket().set_nonblocking(nonblocking)
    }

    /// Clone the underlying socket for out-of-band shutdown.
    pub(super) fn try_clone_socket(&self) -> io::Result<TcpStream> {
        self.socket().try_clone()
    }

    /// Send a TLS close notification when applicable, then close the socket.
    pub(super) fn shutdown(&mut self) -> io::Result<()> {
        match self {
            Self::TlsClient(stream) => {
                stream.conn.send_close_notify();
                stream.flush()?;
            }
            Self::TlsServer(stream) => {
                stream.conn.send_close_notify();
                stream.flush()?;
            }
            Self::Tcp(_) => {}
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
            Self::TlsClient(stream) => &stream.sock,
            Self::TlsServer(stream) => &stream.sock,
        }
    }
}

impl Read for Transport {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(bytes),
            Self::TlsClient(stream) => stream.read(bytes),
            Self::TlsServer(stream) => stream.read(bytes),
        }
    }
}

impl Write for Transport {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(bytes),
            Self::TlsClient(stream) => stream.write(bytes),
            Self::TlsServer(stream) => stream.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::TlsClient(stream) => stream.flush(),
            Self::TlsServer(stream) => stream.flush(),
        }
    }
}
