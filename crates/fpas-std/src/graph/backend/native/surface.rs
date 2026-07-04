//! Native graph surface sizing and redraw presentation.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`

use std::num::NonZeroU32;

impl super::NativeGraphApp {
    pub(super) fn handle_redraw_requested(&mut self) {
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

pub(super) fn normalized_surface_size(width: u32, height: u32) -> Option<(i64, i64)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((i64::from(width), i64::from(height)))
    }
}

#[cfg(test)]
mod tests {
    use super::normalized_surface_size;

    #[test]
    fn normalized_surface_size_accepts_positive_extents() {
        assert_eq!(normalized_surface_size(274, 196), Some((274, 196)));
    }

    #[test]
    fn normalized_surface_size_rejects_zero_extents() {
        assert_eq!(normalized_surface_size(0, 196), None);
        assert_eq!(normalized_surface_size(274, 0), None);
        assert_eq!(normalized_surface_size(0, 0), None);
    }
}
