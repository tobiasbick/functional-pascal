//! `Std.Graph` backend selection, thread-local runtime ownership, and test hooks.
//!
//! **Documentation:** `docs/pascal/std/graph.md` (from the repository root).

mod headless;
mod native;

use super::UploadedFrame;
use crate::error::{StdError, std_runtime_error};
use crate::ui::UiEvent;
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
    Native(Box<native::NativeGraphBackend>),
}

thread_local! {
    static GRAPH_BACKEND: RefCell<Option<GraphBackend>> = const { RefCell::new(None) };
    static BACKEND_MODE: Cell<BackendMode> = const { Cell::new(BackendMode::Native) };
    static HEADLESS_TEST_DEPTH: Cell<u32> = const { Cell::new(0) };
    static HEADLESS_TEST_GUARD: RefCell<Option<HeadlessTestGuard>> = const { RefCell::new(None) };
}

struct HeadlessTestGuard {
    previous: BackendMode,
}

impl Drop for HeadlessTestGuard {
    fn drop(&mut self) {
        GRAPH_BACKEND.with(|slot| {
            slot.borrow_mut().take();
        });
        headless::reset_last_presented_frame_for_tests();
        BACKEND_MODE.with(|mode| mode.set(self.previous));
    }
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
            BackendMode::Native => Ok(Self::Native(Box::new(native::NativeGraphBackend::open(
                width, height, title, location,
            )?))),
        }
    }

    fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        match self {
            Self::Headless(backend) => backend.close(location),
            Self::Native(backend) => backend.close(location),
        }
    }

    fn read_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        match self {
            Self::Headless(backend) => backend.read_event_timeout(timeout_ms, location),
            Self::Native(backend) => backend.read_event_timeout(timeout_ms, location),
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

/// Waits up to `timeout_ms` milliseconds for one shared UI event from the active backend.
pub(crate) fn read_graph_event_timeout(
    timeout_ms: i64,
    location: SourceLocation,
) -> Result<Option<UiEvent>, StdError> {
    with_backend(location, |backend| {
        backend.read_event_timeout(timeout_ms, location)
    })
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
    push_headless_graph_test_mode();
    let result = f();
    pop_headless_graph_test_mode();
    result
}

/// Enables the headless graph backend for one native test session (`Application.OpenForTest`).
#[doc(hidden)]
pub fn push_headless_graph_test_mode() {
    HEADLESS_TEST_DEPTH.with(|depth| {
        if depth.get() == 0 {
            let previous = BACKEND_MODE.with(|mode| mode.replace(BackendMode::Headless));
            GRAPH_BACKEND.with(|slot| {
                slot.borrow_mut().take();
            });
            headless::reset_last_presented_frame_for_tests();
            HEADLESS_TEST_GUARD.with(|guard| {
                *guard.borrow_mut() = Some(HeadlessTestGuard { previous });
            });
        }
        depth.set(depth.get().saturating_add(1));
    });
}

/// Restores the graph backend after a native test session opened with [`push_headless_graph_test_mode`].
#[doc(hidden)]
pub fn pop_headless_graph_test_mode() {
    HEADLESS_TEST_DEPTH.with(|depth| {
        if depth.get() == 0 {
            return;
        }
        depth.set(depth.get() - 1);
        if depth.get() == 0 {
            HEADLESS_TEST_GUARD.with(|guard| {
                guard.borrow_mut().take();
            });
        }
    });
}

/// Returns the last frame presented by the headless graph backend on the current thread.
#[doc(hidden)]
pub fn last_headless_graph_frame_for_tests() -> Option<UploadedFrame> {
    headless::last_presented_frame_for_tests()
}

/// Overrides the headless backend surface size on the current thread.
#[doc(hidden)]
#[cfg(test)]
pub fn set_headless_graph_surface_size_for_tests(
    width: i64,
    height: i64,
) -> Result<(), &'static str> {
    GRAPH_BACKEND.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(backend) = slot.as_mut() else {
            return Err("headless graph backend must be open before overriding its size for tests");
        };
        match backend {
            GraphBackend::Headless(backend) => {
                backend.set_size_for_tests(width, height);
                Ok(())
            }
            GraphBackend::Native(_) => {
                Err("headless graph surface override is only available in headless tests")
            }
        }
    })
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
