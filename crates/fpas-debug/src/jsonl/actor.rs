//! Owned debug-session actor with asynchronous resume and cooperative pause.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::JoinHandle;

use fpas_vm::{DebugPauseHandle, DebugRunResult, DebugSession, DebugSessionError};

pub(super) enum ResumeCommand {
    Continue,
    StepInto,
    StepOver,
    StepOut,
}

pub(super) struct Completion {
    pub session: DebugSession,
    pub result: Result<DebugRunResult, DebugSessionError>,
}

pub(super) struct SessionActor {
    state: ActorState,
}

enum ActorState {
    Ready(Box<DebugSession>),
    Running {
        pause: DebugPauseHandle,
        receiver: Receiver<Completion>,
        thread: Option<JoinHandle<()>>,
    },
    Empty,
}

impl SessionActor {
    pub(super) fn new(session: DebugSession) -> Self {
        Self {
            state: ActorState::Ready(Box::new(session)),
        }
    }

    pub(super) fn session(&self) -> Option<&DebugSession> {
        match &self.state {
            ActorState::Ready(session) => Some(session),
            ActorState::Running { .. } | ActorState::Empty => None,
        }
    }

    pub(super) fn session_mut(&mut self) -> Option<&mut DebugSession> {
        match &mut self.state {
            ActorState::Ready(session) => Some(session),
            ActorState::Running { .. } | ActorState::Empty => None,
        }
    }

    pub(super) fn resume(&mut self, command: ResumeCommand) -> Result<(), DebugSessionError> {
        let ActorState::Ready(session) = std::mem::replace(&mut self.state, ActorState::Empty)
        else {
            unreachable!("protocol state permits resume only while the actor is ready")
        };
        let pause = session.pause_handle();
        let (sender, receiver) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            let mut session = *session;
            let result = match command {
                ResumeCommand::Continue => session.continue_execution(),
                ResumeCommand::StepInto => session.step_into(),
                ResumeCommand::StepOver => session.step_over(),
                ResumeCommand::StepOut => session.step_out(),
            };
            let _ = sender.send(Completion { session, result });
        });
        self.state = ActorState::Running {
            pause,
            receiver,
            thread: Some(thread),
        };
        Ok(())
    }

    pub(super) fn pause(&self) {
        if let ActorState::Running { pause, .. } = &self.state {
            pause.request_pause();
        }
    }

    pub(super) fn poll(&mut self) -> Option<Completion> {
        let ActorState::Running {
            receiver, thread, ..
        } = &mut self.state
        else {
            return None;
        };
        match receiver.try_recv() {
            Ok(completion) => {
                if let Some(thread) = thread.take() {
                    let _ = thread.join();
                }
                self.state = ActorState::Empty;
                Some(completion)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                if let Some(thread) = thread.take() {
                    let _ = thread.join();
                }
                self.state = ActorState::Empty;
                None
            }
        }
    }

    pub(super) fn wait(&mut self) -> Option<Completion> {
        let ActorState::Running {
            receiver, thread, ..
        } = &mut self.state
        else {
            return None;
        };
        let completion = receiver.recv().ok();
        if let Some(thread) = thread.take() {
            let _ = thread.join();
        }
        self.state = ActorState::Empty;
        completion
    }

    pub(super) fn restore(&mut self, session: DebugSession) {
        self.state = ActorState::Ready(Box::new(session));
    }
}
