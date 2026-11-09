#ifndef FPAS_CUI_H
#define FPAS_CUI_H

#include <cui/cui_app.h>

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

/** Draws a single-line editor and claims the pending text if it handled input.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable UTF-8 control identity.
 * @param buffer Writable, zero-terminated UTF-8 text.
 * @param capacity Buffer size in bytes including the terminator.
 * @return CUI text-input event flags. */
CUI_API int fpas_cui_text_input(cui_ctx *ui, const char *id, char *buffer, size_t capacity);

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
