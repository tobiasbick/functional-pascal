#ifndef FPAS_CUI_H
#define FPAS_CUI_H

#include <cui.h>
#include "host_widgets.h"
#include "widgets/button.h"
#include "widgets/button_group.h"
#include "widgets/sidebar.h"
#include "widgets/sheet.h"
#include "widgets/dialog.h"
#include "widgets/action_footer.h"
#include "widgets/icon_button.h"
#include "widgets/copy_button.h"
#include "widgets/hold_button.h"
#include "widgets/confirm_button.h"
#include "widgets/status_button.h"
#include "widgets/segmented.h"
#include "widgets/toggle_group.h"
#include "widgets/checkbox.h"
#include "widgets/switch.h"
#include "widgets/radio_button.h"
#include "widgets/radio_cards.h"
#include "widgets/slider.h"
#include "widgets/range_slider.h"
#include "widgets/chip.h"
#include "widgets/rating.h"
#include "widgets/tabs.h"
#include "widgets/breadcrumb.h"
#include "widgets/pagination.h"
#include "widgets/stepper.h"
#include "widgets/toast.h"
#include "widgets/tooltip.h"
#include "widgets/popover.h"
#include "widgets/confirm_popover.h"
#include "widgets/text_field.h"
#include "widgets/search_field.h"
#include "widgets/text_area.h"
#include "widgets/tag_input.h"
#include "widgets/one_time_code.h"
#include "widgets/password_strength.h"
#include "widgets/select.h"
#include "widgets/number_field.h"
#include "widgets/split_button.h"
#include "widgets/menu.h"

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

/** Begins a scroll area. Close it with cui_scroll_end.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable identity.
 * @param height Positive logical height, or zero to fill available space.
 * @param horizontal Nonzero to enable horizontal scrolling. */
CUI_API void fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal);

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
