//! Static geometry for Turbo Vision-style frame chrome and content viewport.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::ViewRect;

/// Implemented frame capabilities that affect static geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCapabilities {
    /// Whether the title bar reserves a close-button cell on the left.
    pub closable: bool,
    /// Whether the title bar reserves zoom/restore cells on the right.
    pub zoomable: bool,
    /// Whether content overflow may reserve frame-owned scroll bars.
    pub scrollable: bool,
}

impl FrameCapabilities {
    /// Static non-interactive frame chrome.
    #[must_use]
    pub const fn plain() -> Self {
        Self {
            closable: false,
            zoomable: false,
            scrollable: false,
        }
    }

    /// Frame chrome used for the first scrollable window/dialog geometry.
    #[must_use]
    pub const fn scrollable() -> Self {
        Self {
            closable: false,
            zoomable: false,
            scrollable: true,
        }
    }
}

/// Logical content size used to decide scroll-bar visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameContentSize {
    /// Logical content width in terminal cells.
    pub width: i64,
    /// Logical content height in terminal cells.
    pub height: i64,
}

impl FrameContentSize {
    /// Return a content size with non-negative axes.
    #[must_use]
    pub fn new(width: i64, height: i64) -> Self {
        Self {
            width: width.max(0),
            height: height.max(0),
        }
    }
}

/// Reserved title-bar button cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameButtonSlots {
    /// Close button cell (`■`) on the left.
    pub close: Option<ViewRect>,
    /// Zoom/maximize button cell (`▲`) on the right.
    pub zoom: Option<ViewRect>,
    /// Zoom-back/restore button cell (`▼`) on the right.
    pub zoom_back: Option<ViewRect>,
    /// Title text area after the close slot and before zoom slots.
    pub title: Option<ViewRect>,
}

/// Frame-owned scroll-bar slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameScrollbars {
    /// Vertical scroll bar in the frame's right chrome column.
    pub vertical: Option<ViewRect>,
    /// Horizontal scroll bar in the frame's bottom chrome row.
    pub horizontal: Option<ViewRect>,
    /// Bottom-right corner cell when both scroll bars are visible.
    pub corner: Option<ViewRect>,
}

/// Fully resolved static frame geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameGeometry {
    /// Full frame rectangle supplied by the host.
    pub outer: ViewRect,
    /// Top frame row containing border, buttons, and title text.
    pub title_bar: ViewRect,
    /// Inner content area before scroll-bar reservation.
    pub client: ViewRect,
    /// Viewport available to child content after scroll-bar reservation.
    pub view: ViewRect,
    /// Title-bar button and title slots.
    pub buttons: FrameButtonSlots,
    /// Frame-owned scroll-bar slots.
    pub scrollbars: FrameScrollbars,
}

/// Error returned when a frame rectangle cannot contain the requested chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameGeometryError {
    /// Minimum outer width required by the requested capabilities.
    pub min_width: i64,
    /// Minimum outer height required by the requested capabilities.
    pub min_height: i64,
    /// Width that was supplied.
    pub got_width: i64,
    /// Height that was supplied.
    pub got_height: i64,
}

impl FrameGeometry {
    /// Resolve static frame geometry and frame-owned scroll-bar visibility.
    ///
    /// Scroll-bar visibility is solved to a fixed point because a vertical bar can make horizontal
    /// overflow necessary and vice versa.
    pub fn resolve(
        outer: ViewRect,
        content: FrameContentSize,
        capabilities: FrameCapabilities,
    ) -> Result<Self, FrameGeometryError> {
        let (min_width, min_height) = minimum_outer_size(capabilities.scrollable);
        if outer.width < min_width || outer.height < min_height {
            return Err(FrameGeometryError {
                min_width,
                min_height,
                got_width: outer.width,
                got_height: outer.height,
            });
        }

        let title_bar = ViewRect {
            x: outer.x,
            y: outer.y,
            width: outer.width,
            height: 1,
        };
        let client = ViewRect {
            x: outer.x + 1,
            y: outer.y + 1,
            width: outer.width - 2,
            height: outer.height - 2,
        };
        let (has_vertical, has_horizontal) =
            resolve_scrollbars(client, content, capabilities.scrollable);
        let view = ViewRect {
            x: client.x,
            y: client.y,
            width: client.width - i64::from(has_vertical),
            height: client.height - i64::from(has_horizontal),
        };

        Ok(Self {
            outer,
            title_bar,
            client,
            view,
            buttons: resolve_button_slots(outer, capabilities),
            scrollbars: resolve_scrollbar_slots(client, has_vertical, has_horizontal),
        })
    }
}

fn minimum_outer_size(scrollable: bool) -> (i64, i64) {
    if scrollable { (6, 6) } else { (4, 3) }
}

fn resolve_scrollbars(
    client: ViewRect,
    content: FrameContentSize,
    scrollable: bool,
) -> (bool, bool) {
    if !scrollable {
        return (false, false);
    }

    let mut vertical = false;
    let mut horizontal = false;
    loop {
        let view_width = client.width - i64::from(vertical);
        let view_height = client.height - i64::from(horizontal);
        let next_vertical = content.height > view_height;
        let next_horizontal = content.width > view_width;
        if next_vertical == vertical && next_horizontal == horizontal {
            return (vertical, horizontal);
        }
        vertical = next_vertical;
        horizontal = next_horizontal;
    }
}

fn resolve_button_slots(outer: ViewRect, capabilities: FrameCapabilities) -> FrameButtonSlots {
    let close = capabilities.closable.then_some(ViewRect {
        x: outer.x + 2,
        y: outer.y,
        width: 1,
        height: 1,
    });
    let zoom_back = capabilities.zoomable.then_some(ViewRect {
        x: outer.x + outer.width - 3,
        y: outer.y,
        width: 1,
        height: 1,
    });
    let zoom = capabilities.zoomable.then_some(ViewRect {
        x: outer.x + outer.width - 4,
        y: outer.y,
        width: 1,
        height: 1,
    });

    let title_start = close.map_or(outer.x + 2, |slot| slot.x + 2);
    let title_end = zoom
        .map(|slot| slot.x - 1)
        .unwrap_or(outer.x + outer.width - 2);
    let title = (title_end > title_start).then_some(ViewRect {
        x: title_start,
        y: outer.y,
        width: title_end - title_start,
        height: 1,
    });

    FrameButtonSlots {
        close,
        zoom,
        zoom_back,
        title,
    }
}

fn resolve_scrollbar_slots(client: ViewRect, vertical: bool, horizontal: bool) -> FrameScrollbars {
    let horizontal_width = client.width - i64::from(vertical);
    let vertical_height = client.height - i64::from(horizontal);

    FrameScrollbars {
        vertical: vertical.then_some(ViewRect {
            x: client.x + client.width - 1,
            y: client.y,
            width: 1,
            height: vertical_height,
        }),
        horizontal: horizontal.then_some(ViewRect {
            x: client.x,
            y: client.y + client.height - 1,
            width: horizontal_width,
            height: 1,
        }),
        corner: (vertical && horizontal).then_some(ViewRect {
            x: client.x + client.width - 1,
            y: client.y + client.height - 1,
            width: 1,
            height: 1,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn rejects_too_small_plain_frame() {
        assert_eq!(
            FrameGeometry::resolve(
                rect(0, 0, 3, 3),
                FrameContentSize::new(0, 0),
                FrameCapabilities::plain(),
            ),
            Err(FrameGeometryError {
                min_width: 4,
                min_height: 3,
                got_width: 3,
                got_height: 3,
            })
        );
    }

    #[test]
    fn rejects_too_small_scrollable_frame() {
        assert_eq!(
            FrameGeometry::resolve(
                rect(0, 0, 6, 5),
                FrameContentSize::new(0, 0),
                FrameCapabilities::scrollable(),
            ),
            Err(FrameGeometryError {
                min_width: 6,
                min_height: 6,
                got_width: 6,
                got_height: 5,
            })
        );
    }

    #[test]
    fn plain_frame_reserves_title_and_client_rects() {
        let geometry = FrameGeometry::resolve(
            rect(10, 2, 20, 8),
            FrameContentSize::new(4, 2),
            FrameCapabilities::plain(),
        )
        .expect("valid frame");

        assert_eq!(geometry.title_bar, rect(10, 2, 20, 1));
        assert_eq!(geometry.client, rect(11, 3, 18, 6));
        assert_eq!(geometry.view, rect(11, 3, 18, 6));
        assert_eq!(geometry.scrollbars.vertical, None);
        assert_eq!(geometry.scrollbars.horizontal, None);
        assert_eq!(geometry.scrollbars.corner, None);
    }

    #[test]
    fn title_slots_reserve_close_and_zoom_cells() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 20, 6),
            FrameContentSize::new(0, 0),
            FrameCapabilities {
                closable: true,
                zoomable: true,
                scrollable: false,
            },
        )
        .expect("valid frame");

        assert_eq!(geometry.buttons.close, Some(rect(2, 0, 1, 1)));
        assert_eq!(geometry.buttons.zoom, Some(rect(16, 0, 1, 1)));
        assert_eq!(geometry.buttons.zoom_back, Some(rect(17, 0, 1, 1)));
        assert_eq!(geometry.buttons.title, Some(rect(4, 0, 11, 1)));
    }

    #[test]
    fn vertical_scrollbar_uses_frame_chrome_column() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 10, 8),
            FrameContentSize::new(5, 20),
            FrameCapabilities::scrollable(),
        )
        .expect("valid frame");

        assert_eq!(geometry.client, rect(1, 1, 8, 6));
        assert_eq!(geometry.view, rect(1, 1, 7, 6));
        assert_eq!(geometry.scrollbars.vertical, Some(rect(8, 1, 1, 6)));
        assert_eq!(geometry.scrollbars.horizontal, None);
        assert_eq!(geometry.scrollbars.corner, None);
    }

    #[test]
    fn horizontal_scrollbar_uses_frame_chrome_row() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 10, 8),
            FrameContentSize::new(20, 5),
            FrameCapabilities::scrollable(),
        )
        .expect("valid frame");

        assert_eq!(geometry.view, rect(1, 1, 8, 5));
        assert_eq!(geometry.scrollbars.vertical, None);
        assert_eq!(geometry.scrollbars.horizontal, Some(rect(1, 6, 8, 1)));
        assert_eq!(geometry.scrollbars.corner, None);
    }

    #[test]
    fn scrollbar_visibility_is_solved_to_fixed_point() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 10, 8),
            FrameContentSize::new(8, 7),
            FrameCapabilities::scrollable(),
        )
        .expect("valid frame");

        assert_eq!(geometry.view, rect(1, 1, 7, 5));
        assert_eq!(geometry.scrollbars.vertical, Some(rect(8, 1, 1, 5)));
        assert_eq!(geometry.scrollbars.horizontal, Some(rect(1, 6, 7, 1)));
        assert_eq!(geometry.scrollbars.corner, Some(rect(8, 6, 1, 1)));
    }

    #[test]
    fn negative_content_size_is_treated_as_empty() {
        let geometry = FrameGeometry::resolve(
            rect(0, 0, 6, 6),
            FrameContentSize::new(-10, -10),
            FrameCapabilities::scrollable(),
        )
        .expect("valid frame");

        assert_eq!(geometry.view, rect(1, 1, 4, 4));
        assert_eq!(geometry.scrollbars.vertical, None);
        assert_eq!(geometry.scrollbars.horizontal, None);
    }
}
