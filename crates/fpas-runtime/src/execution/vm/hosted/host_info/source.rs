//! A dedicated owner thread also accommodates non-Send platform collectors.
use super::collector::Collector;
use crate::SystemInfoIntrinsic;
use serde_json::Value;
use std::sync::mpsc;

type Outcome = Result<Value, String>;
struct Command {
    operation: SystemInfoIntrinsic,
    reply: mpsc::Sender<Outcome>,
}
pub(super) struct Source {
    sender: Option<mpsc::Sender<Command>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Source {
    pub(super) fn new() -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel::<Command>();
        let thread = std::thread::Builder::new()
            .name("fpas-host-info".into())
            .spawn(move || {
                let mut collector = Collector::default();
                for command in receiver {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        collector.query(command.operation)
                    }))
                    .unwrap_or_else(|_| {
                        collector = Collector::default();
                        Err("Host information collector failed".into())
                    });
                    let _ = command.reply.send(result);
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            sender: Some(sender),
            thread: Some(thread),
        })
    }
    pub(super) fn submit(
        &self,
        operation: SystemInfoIntrinsic,
    ) -> Result<mpsc::Receiver<Outcome>, String> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .as_ref()
            .ok_or("Host information collector is closed")?
            .send(Command { operation, reply })
            .map_err(|_| "Host information collector exited".to_owned())?;
        Ok(receiver)
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
