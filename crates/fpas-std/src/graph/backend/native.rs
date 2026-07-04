//! Native `Std.Graph` backend built on `winit` and `softbuffer`.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::super::UploadedFrame;
use crate::error::{StdError, std_runtime_error};
use crate::ui::{UiEvent, UiModifiers, UiMouse, UiResize, UiWheel};
use crate::{mouse_action_index, mouse_button_index};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use softbuffer::{Context, Surface};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::keyboard::ModifiersState;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowId};

mod input;
mod surface;

use input::{map_winit_key, map_winit_mouse_button, map_winit_wheel_delta};
use surface::normalized_surface_size;

/// Native graph backend state that owns the Winit event loop and Softbuffer surface.
pub(crate) struct NativeGraphBackend {
    event_loop: EventLoop<()>,
    app: NativeGraphApp,
}

impl NativeGraphBackend {
    /// Opens one native graph backend instance.
    pub(crate) fn open(
        width: i64,
        height: i64,
        title: &str,
        location: SourceLocation,
    ) -> Result<Self, StdError> {
        let event_loop = EventLoop::new().map_err(|error| {
            backend_error(
                format!("Std.Graph could not create a native event loop: {error}"),
                "Ensure the program runs on a supported desktop target with window-system access.",
                location,
            )
        })?;
        let context = Context::new(event_loop.owned_display_handle()).map_err(|error| {
            backend_error(
                format!("Std.Graph could not create a softbuffer display context: {error}"),
                "Ensure the desktop window system is available before opening Std.Graph.",
                location,
            )
        })?;

        let mut backend = Self {
            event_loop,
            app: NativeGraphApp::new(context, width, height, title),
        };
        backend.pump(Some(Duration::ZERO), location)?;
        let _ = backend.size(location)?;
        Ok(backend)
    }

    /// Closes the native backend and drops its window resources.
    pub(crate) fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.app.window = None;
        self.app.surface = None;
        self.app.pending_frame = None;
        self.app.pending_events.clear();
        self.app.window_id = None;
        self.app.last_error = None;
        let _ = location;
        Ok(())
    }

    /// Waits up to `timeout_ms` milliseconds for one queued native event.
    pub(crate) fn read_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        if let Some(event) = self.app.pending_events.pop_front() {
            return Ok(Some(event));
        }

        let timeout = Duration::from_millis(timeout_ms.max(0) as u64);
        self.pump(Some(timeout), location)?;
        Ok(self.app.pending_events.pop_front())
    }

    /// Presents one validated frame into the native window.
    pub(crate) fn present_frame(
        &mut self,
        frame: &UploadedFrame,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.app.pending_frame = Some(frame.clone());
        let Some(window) = self.app.window.as_ref() else {
            return Err(backend_error(
                "Std.Graph has no native window to redraw.",
                "Open the application before uploading a frame.",
                location,
            ));
        };
        window.request_redraw();

        for _ in 0..16 {
            self.pump(Some(Duration::ZERO), location)?;
            if self.app.pending_frame.is_none() {
                return Ok(());
            }
        }

        Err(backend_error(
            "Std.Graph did not receive a redraw callback after `Application.UploadFrame(...)`.",
            "Keep polling window events and ensure the window stays visible while presenting frames.",
            location,
        ))
    }

    /// Returns the current native client-area size.
    pub(crate) fn size(&mut self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        self.pump(Some(Duration::ZERO), location)?;
        Ok(self.app.current_size())
    }

    fn pump(
        &mut self,
        timeout: Option<Duration>,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        match self.event_loop.pump_app_events(timeout, &mut self.app) {
            PumpStatus::Continue => self.app.take_error(location),
            PumpStatus::Exit(_exit_code) => Err(backend_error(
                "Std.Graph native event loop exited unexpectedly.",
                "Open a fresh graphics session before issuing more Std.Graph calls.",
                location,
            )),
        }
    }
}

struct NativeGraphApp {
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    window_id: Option<WindowId>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    title: String,
    initial_width: i64,
    initial_height: i64,
    width: i64,
    height: i64,
    cursor_x: i64,
    cursor_y: i64,
    left_button_down: bool,
    right_button_down: bool,
    middle_button_down: bool,
    modifiers: ModifiersState,
    pending_events: VecDeque<UiEvent>,
    pending_frame: Option<UploadedFrame>,
    last_error: Option<String>,
}

impl NativeGraphApp {
    fn new(context: Context<OwnedDisplayHandle>, width: i64, height: i64, title: &str) -> Self {
        Self {
            context,
            window: None,
            window_id: None,
            surface: None,
            title: title.to_string(),
            initial_width: width,
            initial_height: height,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            left_button_down: false,
            right_button_down: false,
            middle_button_down: false,
            modifiers: ModifiersState::default(),
            pending_events: VecDeque::new(),
            pending_frame: None,
            last_error: None,
        }
    }

    fn current_size(&self) -> (i64, i64) {
        (self.width, self.height)
    }

    fn take_error(&mut self, location: SourceLocation) -> Result<(), StdError> {
        match self.last_error.take() {
            Some(message) => Err(backend_error(
                message,
                "Retry the operation after the native window system is available again.",
                location,
            )),
            None => Ok(()),
        }
    }

    fn handle_resize(&mut self, width: u32, height: u32) {
        let Some((width, height)) = normalized_surface_size(width, height) else {
            return;
        };
        self.width = width;
        self.height = height;
        self.pending_events
            .push_back(UiEvent::Resize(UiResize::new(None, None, width, height)));
    }

    fn push_mouse_event(&mut self, action: usize, button: usize) {
        let shift = self.modifiers.shift_key();
        let ctrl = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();
        let meta = self.modifiers.super_key();
        self.pending_events.push_back(UiEvent::Mouse(UiMouse::new(
            action,
            button,
            self.cursor_x,
            self.cursor_y,
            UiModifiers::new(shift, ctrl, alt, meta),
        )));
    }

    fn push_wheel_event(&mut self, delta_x: i64, delta_y: i64) {
        let shift = self.modifiers.shift_key();
        let ctrl = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();
        let meta = self.modifiers.super_key();
        self.pending_events.push_back(UiEvent::Wheel(UiWheel::new(
            delta_x,
            delta_y,
            self.cursor_x,
            self.cursor_y,
            UiModifiers::new(shift, ctrl, alt, meta),
        )));
    }

    fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.cursor_x = x.floor() as i64;
        self.cursor_y = y.floor() as i64;
    }

    fn set_mouse_button_down(&mut self, button: usize, down: bool) {
        if button == mouse_button_index("Left") {
            self.left_button_down = down;
        } else if button == mouse_button_index("Right") {
            self.right_button_down = down;
        } else if button == mouse_button_index("Middle") {
            self.middle_button_down = down;
        }
    }

    fn active_mouse_button(&self) -> usize {
        if self.left_button_down {
            mouse_button_index("Left")
        } else if self.right_button_down {
            mouse_button_index("Right")
        } else if self.middle_button_down {
            mouse_button_index("Middle")
        } else {
            mouse_button_index("None")
        }
    }
}

impl ApplicationHandler for NativeGraphApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(
                self.initial_width as f64,
                self.initial_height as f64,
            ));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Rc::new(window),
            Err(error) => {
                self.last_error = Some(format!(
                    "Std.Graph could not create a native window: {error}"
                ));
                return;
            }
        };
        let surface = match Surface::new(&self.context, Rc::clone(&window)) {
            Ok(surface) => surface,
            Err(error) => {
                self.last_error = Some(format!(
                    "Std.Graph could not create a softbuffer surface for the native window: {error}"
                ));
                return;
            }
        };

        let size = window.inner_size();
        if let Some((width, height)) = normalized_surface_size(size.width, size.height) {
            self.width = width;
            self.height = height;
        }
        self.window_id = Some(window.id());
        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.pending_events.push_back(UiEvent::CloseRequested);
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.inner_size();
                    self.handle_resize(size.width, size.height);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.set_cursor_position(position.x, position.y);
                let button = self.active_mouse_button();
                let action = if button == mouse_button_index("None") {
                    mouse_action_index("Move")
                } else {
                    mouse_action_index("Drag")
                };
                self.push_mouse_event(action, button);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) = map_winit_mouse_button(button) {
                    let action = if state == ElementState::Pressed {
                        mouse_action_index("Down")
                    } else {
                        mouse_action_index("Up")
                    };
                    self.set_mouse_button_down(button, state == ElementState::Pressed);
                    self.push_mouse_event(action, button);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = map_winit_wheel_delta(delta);
                if delta_x != 0 || delta_y != 0 {
                    self.push_wheel_event(delta_x, delta_y);
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if let Some(key) = map_winit_key(&event, self.modifiers) {
                    self.pending_events.push_back(UiEvent::Key(key));
                }
            }
            WindowEvent::RedrawRequested => self.handle_redraw_requested(),
            _ => {}
        }
    }
}

fn backend_error(
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        message.into(),
        help.into(),
        location,
    )
}
