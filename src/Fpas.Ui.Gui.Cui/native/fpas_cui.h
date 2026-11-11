#ifndef FPAS_CUI_H
#define FPAS_CUI_H

#include <cui.h>
#include "host_widgets.h"
#include "widgets/button.h"
#include "widgets/button_group.h"
#include "widgets/sidebar.h"
#include "widgets/sheet.h"
#include "widgets/icon_button.h"
#include "widgets/copy_button.h"
#include "widgets/hold_button.h"
#include "widgets/confirm_button.h"
#include "widgets/status_button.h"
#include "widgets/split_button.h"

/** Presents the completed current frame without waiting for more input.
 * @param app Live UI-thread application with a frame open.
 * @return Nonzero when a nonempty framebuffer was presented. */
CUI_API int fpas_cui_present(cui_app *app);

/** Pumps a frame, optionally keeping a close-requested window alive for cleanup.
 * @param app The owned application.
 * @param defer_close Nonzero to return a final drawable frame after close.
 * @return Zero on close, one for a frame, two for a deferred close request. */
CUI_API int fpas_cui_frame(cui_app *app, int defer_close);

/** Reads the current frame's last widget node.
 * @param ui Current UI-thread context.
 * @return The frame-local node index. */
CUI_API int fpas_cui_last_node(cui_ctx *ui);

/** Reads finalized layout for a tracked node.
 * @param ui Current UI-thread context after presentation.
 * @param node A node from that frame.
 * @return Its logical rectangle, or an empty rectangle for an invalid node. */
CUI_API cui_rect fpas_cui_node_rect(cui_ctx *ui, int node);

/** Makes the last widget fill the available horizontal extent.
 * @param ui Current UI-thread context inside a vertical layout. */
CUI_API void fpas_cui_fill_last_width(cui_ctx *ui);

/** Finishes deferred paint without presenting before resource and window destruction.
 * @param app Live UI-thread application. Destroy it without opening or presenting another frame. */
CUI_API void fpas_cui_finish_frame_for_destroy(cui_app *app);

/** Begins a full-width column. Close it with cui_box_end.
 * @param ui Current UI-thread context.
 * @param padding Nonnegative logical padding.
 * @param gap Nonnegative logical child spacing. */
CUI_API void fpas_cui_column_begin(cui_ctx *ui, float padding, float gap);

/** Begins a full-width row. Close it with cui_box_end.
 * @param ui Current UI-thread context.
 * @param padding Nonnegative logical padding.
 * @param gap Nonnegative logical child spacing. */
CUI_API void fpas_cui_row_begin(cui_ctx *ui, float padding, float gap);

/** Requests focus for the next matching focusable control.
 * @param ui Current UI-thread context.
 * @param id Borrowed control identity, resolved in the current ID scope. */
CUI_API void fpas_cui_request_focus(cui_ctx *ui, const char *id);

/** Moves the synthetic pointer and refreshes hit testing in the open frame.
 * @param ui Current UI-thread context before drawing widgets.
 * @param x Logical horizontal coordinate.
 * @param y Logical vertical coordinate. */
CUI_API void fpas_cui_move_pointer(cui_ctx *ui, float x, float y);

/** Draws an editor with host-supplied selection. A new document version clears undo and scroll state.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable UTF-8 control identity.
 * @param buffer Writable, zero-terminated UTF-8 text.
 * @param capacity Buffer size in bytes including the terminator.
 * @param height Zero for a single-line editor, positive finite logical height for a multiline editor.
 * @param document_version Nonzero token that changes on document replacement.
 * @param anchor Input and output byte offset of the selection anchor, at a UTF-8 scalar boundary.
 * @param caret Input and output byte offset of the caret, at a UTF-8 scalar boundary.
 * @param placeholder Borrowed hint copied into frame-owned memory.
 * @return CUI text-input event flags. */
CUI_API int fpas_cui_text_edit(cui_ctx *ui, const char *id, char *buffer, size_t capacity, float height,
                               int64_t document_version, int *anchor, int *caret, const char *placeholder);

/** Begins a scroll area. Close it with cui_scroll_end.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable identity.
 * @param height Positive logical height, or zero to fill available space.
 * @param horizontal Nonzero to enable horizontal scrolling. */
CUI_API void fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal);

/** Draws a controlled list item.
 * @param ui Current UI-thread context.
 * @param label Borrowed UTF-8 label with a stable identity.
 * @param selected Nonzero when selected.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_list_item(cui_ctx *ui, const char *label, int selected);

/** Draws a controlled tab item, usually inside a horizontal row.
 * @param ui Current UI-thread context.
 * @param label Borrowed UTF-8 label with a stable identity.
 * @param selected Nonzero when selected.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_tab_item(cui_ctx *ui, const char *label, int selected);

/** Begins a selectable table with frame-owned column descriptors.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable identity.
 * @param titles Borrowed UTF-8 column titles copied into the frame.
 * @param count Positive number of titles.
 * @return Nonzero if opened. Close an open table with cui_table_end. */
CUI_API int fpas_cui_table_begin(cui_ctx *ui, const char *id, const char *const *titles, int count);

/** Begins a table row and makes it the target of cui_last_rect.
 * @param ui Current UI-thread context inside a table.
 * @param selected Nonzero when selected.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_table_row(cui_ctx *ui, int selected);

/** Begins a modal dialog and writes back its controlled open flag.
 * @param ui Current UI-thread context.
 * @param title Borrowed UTF-8 title with a stable identity.
 * @param open Input and output open flag, represented as zero or one.
 * @return Nonzero when dialog contents should be drawn. Pair with cui_dialog_end. */
CUI_API int fpas_cui_dialog_begin(cui_ctx *ui, const char *title, int *open);

/** Uploads an RGBA8 image and verifies OpenGL texture storage.
 * @param app Live UI-thread application with its renderer current.
 * @param width Positive width in pixels.
 * @param height Positive height in pixels.
 * @param rgba Borrowed pixel array with four packed bytes per pixel.
 * @return An owned renderer texture, or zero on upload failure. Release it with cui_image_destroy. */
CUI_API cui_texture fpas_cui_image_create(cui_app *app, int width, int height, const void *rgba);

/** Queues a texture for rendering in the current view.
 * @param ui Current UI-thread context.
 * @param texture Live texture belonging to this context. Keep it alive while the frame can be rendered.
 * @param width Positive finite display width in logical units.
 * @param height Positive finite display height in logical units. */
CUI_API void fpas_cui_image(cui_ctx *ui, cui_texture texture, float width, float height);

/** Sets the Win32 live-resize renderer. The callback draws on the UI thread.
 * @param app UI-thread application with no callback currently in flight.
 * @param render Renderer receiving the open CUI frame and user state, or NULL to detach.
 * @param user Borrowed state valid until detachment or application destruction. */
CUI_API void fpas_cui_set_resize_renderer(cui_app *app, void (*render)(cui_ctx *, void *), void *user);

/** Acquires the platform event sender on the UI thread.
 * @param app Live UI-thread application.
 * @return Owned wake target, or NULL if it could not be opened. */
CUI_API void *fpas_cui_wake_open(cui_app *app);

/** Posts an event to a live window without accessing its CUI state.
 * @param target Wake target whose lifetime is protected by the caller.
 * @return Nonzero if the event was posted. */
CUI_API int fpas_cui_wake_post(void *target);

/** Releases the sender before destroying its window. Serialize with posting.
 * @param target Owned wake target. */
CUI_API void fpas_cui_wake_close(void *target);

#endif
