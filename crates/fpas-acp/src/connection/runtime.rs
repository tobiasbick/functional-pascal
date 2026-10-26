//! One current-thread native executor per host VM, with bounded connection ownership.

use super::{
    Connection, Fault, Limits, driver,
    shared::{CONNECTION_TAG, SERIAL_MASK, Shared},
};
use crate::{
    schema::Version,
    transport::{self, MessageEndpoint, Transport},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
};

const CONNECTIONS: usize = 16;
static NEXT: AtomicU64 = AtomicU64::new(1);

struct State {
    connections: Mutex<BTreeMap<u64, Connection>>,
    stopping: AtomicBool,
    changed: tokio::sync::Notify,
}

/// Native SDK executor. Shut it down only after host workers have stopped polling.
pub struct Runtime {
    state: Arc<State>,
    opens: tokio::sync::mpsc::Sender<(Arc<Shared>, Arc<dyn Transport>)>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl Runtime {
    /// Allocate the runtime without launching an agent or granting host access.
    pub fn new() -> Result<Self, Fault> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(|_| Fault::Host)?;
        let (opens, mut incoming) =
            tokio::sync::mpsc::channel::<(Arc<Shared>, Arc<dyn Transport>)>(CONNECTIONS);
        let state = Arc::new(State {
            connections: Mutex::new(BTreeMap::new()),
            stopping: AtomicBool::new(false),
            changed: tokio::sync::Notify::new(),
        });
        let owned = state.clone();
        let thread = std::thread::Builder::new()
            .name("fpas-acp".into())
            .spawn(move || {
                runtime.block_on(async {
                    let mut tasks = tokio::task::JoinSet::new();
                    loop {
                        let changed = owned.changed.notified();
                        tokio::pin!(changed);
                        changed.as_mut().enable();
                        if owned.stopping.load(Ordering::Acquire) {
                            break;
                        }
                        tokio::select! {
                            _ = &mut changed => {},
                            open = incoming.recv() => {
                                let Some((shared, transport)) = open else { break; };
                                tasks.spawn(driver::run(shared, transport));
                            }
                            _ = tasks.join_next(), if !tasks.is_empty() => {},
                        }
                    }
                    incoming.close();
                    while let Some((shared, transport)) = incoming.recv().await {
                        tasks.spawn(driver::run(shared, transport));
                    }
                    while tasks.join_next().await.is_some() {}
                });
            })
            .map_err(|_| Fault::Host)?;
        Ok(Self {
            state,
            opens,
            thread: Mutex::new(Some(thread)),
        })
    }

    /// Open the production message endpoint using the same SDK path as processes.
    pub fn connect_messages(
        &self,
        version: Version,
        limits: Limits,
    ) -> Result<(Connection, MessageEndpoint), Fault> {
        self.connect_messages_configured(super::Settings::for_version(version), limits)
    }

    /// Open an endpoint with the initialization offer fixed before admission.
    pub fn connect_messages_configured(
        &self,
        settings: super::Settings,
        limits: Limits,
    ) -> Result<(Connection, MessageEndpoint), Fault> {
        let (endpoint, transport) = transport::messages(&limits)?;
        let connection = self.connect_configured(settings, limits, Arc::new(transport))?;
        Ok((connection, endpoint))
    }

    /// Attach an owned physical transport; the host retains no second child owner.
    pub fn connect(
        &self,
        version: Version,
        limits: Limits,
        transport: Arc<dyn Transport>,
    ) -> Result<Connection, Fault> {
        self.connect_configured(super::Settings::for_version(version), limits, transport)
    }

    /// Attach a transport with its initialization offer validated before effects.
    pub fn connect_configured(
        &self,
        settings: super::Settings,
        limits: Limits,
        transport: Arc<dyn Transport>,
    ) -> Result<Connection, Fault> {
        limits.validate()?;
        settings.requested_version()?;
        if self.state.stopping.load(Ordering::Acquire) {
            return Err(Fault::Closed);
        }
        let mut connections = self
            .state
            .connections
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if self.state.stopping.load(Ordering::Acquire) {
            return Err(Fault::Closed);
        }
        connections.retain(|_, connection| !connection.shared.lock().released);
        if connections.len() == CONNECTIONS {
            return Err(Fault::LimitExceeded("Connections", CONNECTIONS));
        }
        let serial = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                (next <= SERIAL_MASK).then_some(next + 1)
            })
            .map_err(|_| Fault::LimitExceeded("Connections", CONNECTIONS))?;
        let shared = Shared::new(CONNECTION_TAG | serial, settings, limits)?;
        let connection = Connection {
            shared: shared.clone(),
        };
        self.opens
            .try_send((shared, transport))
            .map_err(|_| Fault::Busy("Connect"))?;
        connections.insert(connection.id(), connection.clone());
        Ok(connection)
    }

    /// Reject new connections and wake every SDK task without joining a VM worker.
    pub fn request_shutdown(&self) {
        self.state.stopping.store(true, Ordering::Release);
        let connections: Vec<_> = self
            .state
            .connections
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect();
        for connection in connections {
            let _ = connection.close();
        }
        self.state.changed.notify_waiters();
    }

    /// Join the sole executor after all owned connections have cleaned up.
    pub fn shutdown(&self) {
        self.request_shutdown();
        if let Some(thread) = self.thread.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = thread.join();
        }
        self.state
            .connections
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
