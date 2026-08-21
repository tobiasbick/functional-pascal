//! Owned debug-session actor with asynchronous resume and cooperative pause.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::JoinHandle;

use fpas_vm::{
    DebugEvaluateResult, DebugEvaluationCancelHandle, DebugEvaluationLimits, DebugExpression,
    DebugPauseHandle, DebugRunResult, DebugSession, DebugSessionError,
};

#[derive(Clone, Copy)]
/// Resume operation transferred to the owned session thread.
pub(crate) enum ResumeCommand {
    /// Resume the complete session without a selected-task step target.
    Continue,
    /// Step into the optional explicitly selected task.
    StepInto(Option<u64>),
    /// Step over in the optional explicitly selected task.
    StepOver(Option<u64>),
    /// Step out of the optional explicitly selected task.
    StepOut(Option<u64>),
}

impl ResumeCommand {
    /// Return the explicit selected task, when this is a task-specific step.
    pub(crate) const fn task_id(self) -> Option<u64> {
        match self {
            Self::Continue => None,
            Self::StepInto(task_id) | Self::StepOver(task_id) | Self::StepOut(task_id) => task_id,
        }
    }
}

pub(crate) struct ResumeCompletion {
    pub session: DebugSession,
    pub result: Result<DebugRunResult, DebugSessionError>,
}

pub(crate) struct EvaluationCompletion {
    pub session: DebugSession,
    pub result: Result<DebugEvaluateResult, DebugSessionError>,
}

pub(crate) enum ActorCompletion {
    Resume(ResumeCompletion),
    Evaluation(EvaluationCompletion),
}

pub(crate) struct SessionActor {
    state: ActorState,
}

enum ActorState {
    Ready(Box<DebugSession>),
    Running {
        pause: DebugPauseHandle,
        receiver: Receiver<ActorCompletion>,
        thread: Option<JoinHandle<()>>,
    },
    Evaluating {
        cancel: DebugEvaluationCancelHandle,
        receiver: Receiver<ActorCompletion>,
        thread: Option<JoinHandle<()>>,
    },
    Empty,
}

impl SessionActor {
    pub(crate) fn new(session: DebugSession) -> Self {
        Self {
            state: ActorState::Ready(Box::new(session)),
        }
    }

    pub(crate) fn session(&self) -> Option<&DebugSession> {
        match &self.state {
            ActorState::Ready(session) => Some(session),
            ActorState::Running { .. } | ActorState::Evaluating { .. } | ActorState::Empty => None,
        }
    }

    pub(crate) fn session_mut(&mut self) -> Option<&mut DebugSession> {
        match &mut self.state {
            ActorState::Ready(session) => Some(session),
            ActorState::Running { .. } | ActorState::Evaluating { .. } | ActorState::Empty => None,
        }
    }

    pub(crate) fn resume(&mut self, command: ResumeCommand) -> Result<(), DebugSessionError> {
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
                ResumeCommand::StepInto(Some(task_id)) => session.step_into_task(task_id),
                ResumeCommand::StepInto(None) => session.step_into(),
                ResumeCommand::StepOver(Some(task_id)) => session.step_over_task(task_id),
                ResumeCommand::StepOver(None) => session.step_over(),
                ResumeCommand::StepOut(Some(task_id)) => session.step_out_task(task_id),
                ResumeCommand::StepOut(None) => session.step_out(),
            };
            let _ = sender.send(ActorCompletion::Resume(ResumeCompletion {
                session,
                result,
            }));
        });
        self.state = ActorState::Running {
            pause,
            receiver,
            thread: Some(thread),
        };
        Ok(())
    }

    pub(crate) fn evaluate(
        &mut self,
        expression: DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) {
        let ActorState::Ready(session) = std::mem::replace(&mut self.state, ActorState::Empty)
        else {
            unreachable!("protocol state permits evaluation only while the actor is ready")
        };
        let cancel = session.evaluation_cancel_handle();
        let (sender, receiver) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            let mut session = *session;
            let result = session.evaluate_with_limits(&expression, frame_id, limits);
            let _ = sender.send(ActorCompletion::Evaluation(EvaluationCompletion {
                session,
                result,
            }));
        });
        self.state = ActorState::Evaluating {
            cancel,
            receiver,
            thread: Some(thread),
        };
    }

    pub(crate) fn pause(&self) {
        if let ActorState::Running { pause, .. } = &self.state {
            pause.request_pause();
        }
    }

    pub(crate) fn cancel_evaluation(&self) -> bool {
        if let ActorState::Evaluating { cancel, .. } = &self.state {
            cancel.cancel();
            true
        } else {
            false
        }
    }

    pub(crate) const fn is_evaluating(&self) -> bool {
        matches!(self.state, ActorState::Evaluating { .. })
    }

    pub(crate) fn poll(&mut self) -> Option<ActorCompletion> {
        let (receiver, thread) = match &mut self.state {
            ActorState::Running {
                receiver, thread, ..
            }
            | ActorState::Evaluating {
                receiver, thread, ..
            } => (receiver, thread),
            ActorState::Ready(_) | ActorState::Empty => return None,
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

    pub(crate) fn wait(&mut self) -> Option<ActorCompletion> {
        let (receiver, thread) = match &mut self.state {
            ActorState::Running {
                receiver, thread, ..
            }
            | ActorState::Evaluating {
                receiver, thread, ..
            } => (receiver, thread),
            ActorState::Ready(_) | ActorState::Empty => return None,
        };
        let completion = receiver.recv().ok();
        if let Some(thread) = thread.take() {
            let _ = thread.join();
        }
        self.state = ActorState::Empty;
        completion
    }

    pub(crate) fn restore(&mut self, session: DebugSession) {
        self.state = ActorState::Ready(Box::new(session));
    }
}

impl Drop for SessionActor {
    fn drop(&mut self) {
        match &mut self.state {
            ActorState::Running { pause, thread, .. } => {
                pause.request_pause();
                if let Some(thread) = thread.take() {
                    let _ = thread.join();
                }
            }
            ActorState::Evaluating { cancel, thread, .. } => {
                cancel.cancel();
                if let Some(thread) = thread.take() {
                    let _ = thread.join();
                }
            }
            ActorState::Ready(_) | ActorState::Empty => {}
        }
    }
}
