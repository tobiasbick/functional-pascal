//! Native `Std.Graph` backend built on `winit` and `softbuffer`.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/05-backend-selection.md` (from the repository root).

use super::super::{GraphEvent, UploadedFrame};
use crate::error::{StdError, std_runtime_error};
use crate::{ConsoleKeyEvent, key_event::key_kind_index};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use softbuffer::{Context, Surface};
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowId};

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

    /// Polls one queued native event after pumping the platform event loop.
    pub(crate) fn poll_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<Option<GraphEvent>, StdError> {
        self.pump(Some(Duration::ZERO), location)?;
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
    modifiers: ModifiersState,
    pending_events: VecDeque<GraphEvent>,
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
        self.width = i64::from(width);
        self.height = i64::from(height);
        self.pending_events.push_back(GraphEvent::Resize {
            width: self.width,
            height: self.height,
        });
    }

    fn handle_redraw_requested(&mut self) {
        let Some(frame) = self.pending_frame.take() else {
            return;
        };
        let Some(surface) = self.surface.as_mut() else {
            self.last_error = Some(
                "Std.Graph has no softbuffer surface while trying to redraw the native window."
                    .to_string(),
            );
            return;
        };

        let Ok(width) = u32::try_from(frame.width()) else {
            self.last_error = Some("Std.Graph surface width does not fit into u32.".to_string());
            return;
        };
        let Ok(height) = u32::try_from(frame.height()) else {
            self.last_error = Some("Std.Graph surface height does not fit into u32.".to_string());
            return;
        };
        let Some(width) = NonZeroU32::new(width) else {
            self.last_error =
                Some("Std.Graph cannot resize a native surface to width 0.".to_string());
            return;
        };
        let Some(height) = NonZeroU32::new(height) else {
            self.last_error =
                Some("Std.Graph cannot resize a native surface to height 0.".to_string());
            return;
        };

        if let Err(error) = surface.resize(width, height) {
            self.last_error = Some(format!(
                "Std.Graph could not resize the softbuffer surface: {error}"
            ));
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(error) => {
                self.last_error = Some(format!(
                    "Std.Graph could not lock the softbuffer frame buffer: {error}"
                ));
                return;
            }
        };

        for (slot, pixel) in buffer.iter_mut().zip(frame.pixels().iter().copied()) {
            *slot = pixel;
        }
        if let Err(error) = buffer.present() {
            self.last_error = Some(format!("Std.Graph could not present the frame: {error}"));
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
        self.width = i64::from(size.width);
        self.height = i64::from(size.height);
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
                self.pending_events.push_back(GraphEvent::CloseRequested);
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
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if let Some(key) = map_winit_key(&event, self.modifiers) {
                    self.pending_events.push_back(GraphEvent::Key(key));
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

fn map_winit_key(event: &KeyEvent, modifiers: ModifiersState) -> Option<ConsoleKeyEvent> {
    if event.state != ElementState::Pressed || event.repeat {
        return None;
    }

    let shift = modifiers.shift_key();
    let ctrl = modifiers.control_key();
    let alt = modifiers.alt_key();
    let meta = modifiers.super_key();

    match &event.logical_key {
        Key::Named(NamedKey::Escape) => Some(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Tab) => Some(ConsoleKeyEvent::new(
            key_kind_index("Tab"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Enter) => Some(ConsoleKeyEvent::new(
            key_kind_index("Enter"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Backspace) => Some(ConsoleKeyEvent::new(
            key_kind_index("Backspace"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Space) => Some(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowUp) => Some(ConsoleKeyEvent::new(
            key_kind_index("Up"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowDown) => Some(ConsoleKeyEvent::new(
            key_kind_index("Down"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowLeft) => Some(ConsoleKeyEvent::new(
            key_kind_index("Left"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowRight) => Some(ConsoleKeyEvent::new(
            key_kind_index("Right"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Home) => Some(ConsoleKeyEvent::new(
            key_kind_index("Home"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::End) => Some(ConsoleKeyEvent::new(
            key_kind_index("End"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::PageUp) => Some(ConsoleKeyEvent::new(
            key_kind_index("PageUp"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::PageDown) => Some(ConsoleKeyEvent::new(
            key_kind_index("PageDown"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Insert) => Some(ConsoleKeyEvent::new(
            key_kind_index("Insert"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Delete) => Some(ConsoleKeyEvent::new(
            key_kind_index("Delete"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::F1) => Some(function_key_event(1, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F2) => Some(function_key_event(2, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F3) => Some(function_key_event(3, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F4) => Some(function_key_event(4, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F5) => Some(function_key_event(5, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F6) => Some(function_key_event(6, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F7) => Some(function_key_event(7, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F8) => Some(function_key_event(8, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F9) => Some(function_key_event(9, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F10) => Some(function_key_event(10, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F11) => Some(function_key_event(11, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F12) => Some(function_key_event(12, shift, ctrl, alt, meta)),
        Key::Character(text) => map_character_key(text.as_str(), shift, ctrl, alt, meta),
        _ => event
            .text
            .as_ref()
            .and_then(|text| map_character_key(text.as_str(), shift, ctrl, alt, meta))
            .or_else(|| {
                Some(ConsoleKeyEvent::new(
                    key_kind_index("Unknown"),
                    '\0',
                    shift,
                    ctrl,
                    alt,
                    meta,
                ))
            }),
    }
}

fn function_key_event(
    number: u8,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
) -> ConsoleKeyEvent {
    ConsoleKeyEvent::new(
        key_kind_index(&format!("F{number}")),
        '\0',
        shift,
        ctrl,
        alt,
        meta,
    )
}

fn map_character_key(
    text: &str,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
) -> Option<ConsoleKeyEvent> {
    let ch = text.chars().next()?;
    let kind = if ch == ' ' {
        key_kind_index("Space")
    } else {
        key_kind_index("Character")
    };
    Some(ConsoleKeyEvent::new(kind, ch, shift, ctrl, alt, meta))
}
