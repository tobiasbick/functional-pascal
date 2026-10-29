//! VM-owned TCP listener registry.

use crate::execution::vm::hosted::net::failure::Failure;
use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::tls::server::TlsServer;
use super::transport::Transport;

const HANDLE_TAG: u64 = 0x4E4C_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

#[cfg(test)]
#[path = "listeners/address_tests.rs"]
mod address_tests;

/// Thread-safe TCP and TLS listeners owned by one VM.
pub(in crate::execution::vm::hosted) struct NetworkListeners {
    next_handle: AtomicU64,
    listeners: Mutex<HashMap<u64, Arc<Listener>>>,
    shutdown: AtomicBool,
}

struct Listener {
    socket: TcpListener,
    mode: ListenerMode,
    closed: AtomicBool,
}

impl Listener {
    fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

enum ListenerMode {
    Tcp,
    Tls(TlsServer),
}

impl NetworkListeners {
    /// Create an empty per-VM listener registry.
    pub(in crate::execution::vm::hosted) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            listeners: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
        }
    }

    /// Bind a TCP listener and return its opaque runtime handle.
    /// Port zero requests an OS-assigned port; see `docs/pascal/std/network/net.md`.
    pub(super) fn listen(&self, host: &str, port: i64) -> Result<u64, Failure> {
        self.listen_with_cancellation(host, port, || false)
    }

    pub(super) fn listen_with_cancellation(
        &self,
        host: &str,
        port: i64,
        cancelled: impl Fn() -> bool,
    ) -> Result<u64, Failure> {
        self.bind(host, port, ListenerMode::Tcp, cancelled)
    }

    /// Bind a TLS listener configured from PEM files.
    /// Port zero requests an OS-assigned port; see `docs/pascal/std/network/net.md`.
    pub(super) fn listen_tls(
        &self,
        host: &str,
        port: i64,
        certificate_path: &str,
        private_key_path: &str,
        handshake_timeout_millis: i64,
    ) -> Result<u64, Failure> {
        self.listen_tls_with_cancellation(
            host,
            port,
            certificate_path,
            private_key_path,
            handshake_timeout_millis,
            || false,
        )
    }

    pub(super) fn listen_tls_with_cancellation(
        &self,
        host: &str,
        port: i64,
        certificate_path: &str,
        private_key_path: &str,
        handshake_timeout_millis: i64,
        cancelled: impl Fn() -> bool,
    ) -> Result<u64, Failure> {
        self.ensure_active(&cancelled)?;
        let tls = TlsServer::from_pem_files_controlled(
            certificate_path,
            private_key_path,
            handshake_timeout_millis,
            || cancelled() || self.shutdown.load(Ordering::Acquire),
        )?;
        self.bind(host, port, ListenerMode::Tls(tls), cancelled)
    }

    /// Return the bound address of an open TCP or TLS listener.
    /// See `docs/pascal/std/network/net.md`.
    pub(super) fn local_address(&self, handle: u64) -> Result<SocketAddr, Failure> {
        self.listener(handle)?.socket.local_addr().map_err(|error| {
            Failure::io(
                &error,
                format!("Could not read network listener address: {error}"),
            )
        })
    }

    fn ensure_active(&self, cancelled: impl Fn() -> bool) -> Result<(), Failure> {
        if cancelled() {
            Err(Failure::Cancelled)
        } else if self.shutdown.load(Ordering::Acquire) {
            Err(Failure::Closed)
        } else {
            Ok(())
        }
    }

    fn bind(
        &self,
        host: &str,
        port: i64,
        mode: ListenerMode,
        cancelled: impl Fn() -> bool,
    ) -> Result<u64, Failure> {
        let port = listener_port(port)?;
        self.ensure_active(&cancelled)?;
        let addresses = super::connections::resolve_listener(host, port, || {
            cancelled() || self.shutdown.load(Ordering::Acquire)
        })?;
        self.ensure_active(&cancelled)?;
        if addresses.is_empty() {
            return Err(Failure::no_addresses(format!(
                "Listener host '{host}' resolved to no addresses"
            )));
        }
        let socket = TcpListener::bind(addresses.as_slice()).map_err(|error| {
            Failure::io(
                &error,
                format!("Could not bind network listener on {host}:{port}: {error}"),
            )
        })?;
        socket.set_nonblocking(true).map_err(|error| {
            Failure::io(
                &error,
                format!("Could not configure network listener: {error}"),
            )
        })?;
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        let listener = Arc::new(Listener {
            socket,
            mode,
            closed: AtomicBool::new(false),
        });
        let mut listeners = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.ensure_active(cancelled)?;
        listeners.insert(handle, listener);
        Ok(handle)
    }

    /// Accept one TCP or TLS connection from a listener.
    pub(super) fn accept(&self, handle: u64) -> Result<Transport, Failure> {
        self.accept_until_cancelled(handle, || false)
    }

    /// Accept one TCP or TLS connection until the supplied token is cancelled.
    pub(super) fn accept_with_cancellation(
        &self,
        handle: u64,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Transport, Failure> {
        self.accept_until_cancelled(handle, is_cancelled)
    }

    fn accept_until_cancelled(
        &self,
        handle: u64,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Transport, Failure> {
        let listener = self.listener(handle)?;
        loop {
            ensure_accept_active(&listener, &is_cancelled)?;
            let (stream, _) = match listener.socket.accept() {
                Ok(accepted) => accepted,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::park_timeout(Duration::from_millis(10));
                    continue;
                }
                Err(error) => {
                    return Err(Failure::io(
                        &error,
                        format!("Network listener accept failed: {error}"),
                    ));
                }
            };
            ensure_accept_active(&listener, &is_cancelled)?;
            // Windows accepted sockets inherit the listener's non-blocking mode; plain reads and
            // TLS handshake timeouts require a blocking socket.
            stream.set_nonblocking(false).map_err(|error| {
                Failure::io(
                    &error,
                    format!("Could not configure accepted connection: {error}"),
                )
            })?;
            stream.set_nodelay(true).map_err(|error| {
                Failure::io(
                    &error,
                    format!("Could not configure accepted connection: {error}"),
                )
            })?;
            match &listener.mode {
                ListenerMode::Tcp => {
                    ensure_accept_active(&listener, &is_cancelled)?;
                    return Ok(Transport::tcp(stream));
                }
                ListenerMode::Tls(tls) => {
                    if let Ok(stream) =
                        tls.accept(stream, || listener.is_closed() || is_cancelled())
                    {
                        ensure_accept_active(&listener, &is_cancelled)?;
                        return Ok(Transport::tls_server(stream));
                    }
                }
            }
        }
    }

    /// Close and invalidate one listener handle.
    pub(in crate::execution::vm::hosted) fn close(&self, handle: u64) -> Result<(), Failure> {
        validate_handle(handle)?;
        let listener = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or(Failure::Closed)?;
        listener.close();
        Ok(())
    }

    /// Interrupt and invalidate every listener owned by the VM.
    pub(in crate::execution::vm::hosted) fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        let listeners = self
            .listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain()
            .map(|(_, listener)| listener)
            .collect::<Vec<_>>();
        for listener in listeners {
            listener.close();
        }
    }

    /// Validate a listener before transferring it to a server lifetime.
    pub(in crate::execution::vm::hosted) fn validate(&self, handle: u64) -> Result<(), Failure> {
        self.listener(handle).map(|_| ())
    }

    fn listener(&self, handle: u64) -> Result<Arc<Listener>, Failure> {
        validate_handle(handle)?;
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or(Failure::Closed)
    }
}

fn ensure_accept_active(
    listener: &Listener,
    is_cancelled: impl Fn() -> bool,
) -> Result<(), Failure> {
    if is_cancelled() {
        return Err(Failure::Cancelled);
    }
    if listener.is_closed() {
        return Err(Failure::Closed);
    }
    Ok(())
}

fn listener_port(port: i64) -> Result<u16, Failure> {
    u16::try_from(port).map_err(|_| {
        Failure::argument(
            "Port",
            format!("Network listener port must be in 0..=65535, got {port}"),
        )
    })
}

fn validate_handle(handle: u64) -> Result<(), Failure> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err(Failure::argument(
            "Listener",
            "Value is not a network listener handle",
        ))
    }
}

#[cfg(test)]
#[path = "listeners/tests.rs"]
mod tests;
