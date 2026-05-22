//! `Std.Graph` backend selection, thread-local runtime ownership, and test hooks.
//!
//! **Documentation:** `docs/future/std.graph/03-runtime-architecture.md`, `docs/future/std.graph/05-backend-selection.md` (from the repository root).

mod headless;
mod native;

use super::{GraphEvent, UploadedFrame};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BackendMode {
    #[default]
    Native,
    Headless,
}

enum GraphBackend {
    Headless(headless::HeadlessGraphBackend),
    Native(native::NativeGraphBackend),
}

thread_local! {
    static GRAPH_BACKEND: RefCell<Option<GraphBackend>> = const { RefCell::new(None) };
    static BACKEND_MODE: Cell<BackendMode> = const { Cell::new(BackendMode::Native) };
}

impl GraphBackend {
    fn open(
        width: i64,
        height: i64,
        title: &str,
        location: SourceLocation,
    ) -> Result<Self, StdError> {
        match BACKEND_MODE.with(Cell::get) {
            BackendMode::Headless => Ok(Self::Headless(headless::HeadlessGraphBackend::open(
                width, height, title,
            ))),
            BackendMode::Native => Ok(Self::Native(native::NativeGraphBackend::open(
                width, height, title, location,
            )?)),
        }
    }

    fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        match self {
            Self::Headless(backend) => backend.close(location),
            Self::Native(backend) => backend.close(location),
        }
    }

    fn poll_event(&mut self, location: SourceLocation) -> Result<Option<GraphEvent>, StdError> {
        match self {
            Self::Headless(backend) => backend.poll_event(location),
            Self::Native(backend) => backend.poll_event(location),
        }
    }

    fn present_frame(
        &mut self,
        frame: &UploadedFrame,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        match self {
            Self::Headless(backend) => backend.present_frame(frame, location),
            Self::Native(backend) => backend.present_frame(frame, location),
        }
    }

    fn size(&mut self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        match self {
            Self::Headless(backend) => backend.size(location),
            Self::Native(backend) => backend.size(location),
        }
    }
}

/// Opens the active graph backend for one `Std.Graph` session.
pub(crate) fn open_graph_backend(
    width: i64,
    height: i64,
    title: &str,
    location: SourceLocation,
) -> Result<(i64, i64), StdError> {
    GRAPH_BACKEND.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return Err(missing_backend_error(
                "Std.Graph backend state is already active for this thread.",
                "Close the current graphics session before opening another one.",
                location,
            ));
        }

        let mut backend = GraphBackend::open(width, height, title, location)?;
        let size = backend.size(location)?;
        *slot = Some(backend);
        Ok(size)
    })
}

/// Closes and drops the active graph backend for this thread.
pub(crate) fn close_graph_backend(location: SourceLocation) -> Result<(), StdError> {
    GRAPH_BACKEND.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(mut backend) = slot.take() else {
            return Err(missing_backend_error(
                "Std.Graph backend state is missing for the current thread.",
                "Open the application before closing it.",
                location,
            ));
        };

        backend.close(location)
    })
}

/// Returns the current graph surface size from the active backend.
pub(crate) fn graph_surface_size(location: SourceLocation) -> Result<(i64, i64), StdError> {
    with_backend(location, |backend| backend.size(location))
}

/// Polls one graph event from the active backend.
pub(crate) fn poll_graph_event(location: SourceLocation) -> Result<Option<GraphEvent>, StdError> {
    with_backend(location, |backend| backend.poll_event(location))
}

/// Presents one validated frame through the active backend.
pub(crate) fn present_graph_frame(
    frame: &UploadedFrame,
    location: SourceLocation,
) -> Result<(), StdError> {
    with_backend(location, |backend| backend.present_frame(frame, location))
}

/// Runs `f` with a deterministic headless graph backend on the current thread.
#[doc(hidden)]
pub fn with_headless_graph_backend_for_tests<T>(f: impl FnOnce() -> T) -> T {
    struct ResetGuard {
        previous: BackendMode,
    }

    impl Drop for ResetGuard {
        fn drop(&mut self) {
            GRAPH_BACKEND.with(|slot| {
                slot.borrow_mut().take();
            });
            headless::reset_last_presented_frame_for_tests();
            BACKEND_MODE.with(|mode| mode.set(self.previous));
        }
    }

    let previous = BACKEND_MODE.with(|mode| mode.replace(BackendMode::Headless));
    GRAPH_BACKEND.with(|slot| {
        slot.borrow_mut().take();
    });
    headless::reset_last_presented_frame_for_tests();
    let _reset = ResetGuard { previous };
    f()
}

/// Returns the last frame presented by the headless graph backend on the current thread.
#[doc(hidden)]
pub fn last_headless_graph_frame_for_tests() -> Option<UploadedFrame> {
    headless::last_presented_frame_for_tests()
}

fn with_backend<T>(
    location: SourceLocation,
    f: impl FnOnce(&mut GraphBackend) -> Result<T, StdError>,
) -> Result<T, StdError> {
    GRAPH_BACKEND.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(backend) = slot.as_mut() else {
            return Err(missing_backend_error(
                "Std.Graph backend state is missing for the current thread.",
                "Open the application before using `Std.Graph.Application.*` calls.",
                location,
            ));
        };
        f(backend)
    })
}

fn missing_backend_error(message: &str, help: &str, location: SourceLocation) -> StdError {
    std_runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, location)
}
