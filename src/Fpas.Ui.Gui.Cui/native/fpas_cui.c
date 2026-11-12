#if defined(_WIN32) && !defined(WIN32_LEAN_AND_MEAN)
/* Keep the RPC macro named small out of the UI implementation. */
#define WIN32_LEAN_AND_MEAN
#endif

#ifdef _MSC_VER
/* The release combines sources whose locals can shadow private file-scope names. */
#pragma warning(push)
#pragma warning(disable: 4459)
#endif

/* The adapters use private CUI types defined inside the release implementation. */
#include "cui.c"

#ifdef _MSC_VER
#pragma warning(pop)
#endif

static void fpas_paint_text_area(cui_ctx *ui, cui_node *area, void *data);
static void fpas_route_text_area_wheel(cui_ctx *ui);

#include "bridge.c"
#include "controls.c"
#include "widget_nodes.c"
#include "control_state.c"
#include "widgets/feedback.c"
#include "widgets/disclosure.c"
#include "widgets/button.c"
#include "widgets/button_group.c"
#include "host_widgets.c"
#include "widgets/sidebar.c"
#include "widgets/modal.c"
#include "widgets/sheet.c"
#include "widgets/dialog.c"
#include "widgets/action_footer.c"
#include "widgets/icon_button.c"
#include "widgets/copy_button.c"
#include "widgets/hold_button.c"
#include "widgets/morph_button.c"
#include "widgets/confirm_button.c"
#include "widgets/status_button.c"
#include "widgets/segmented.c"
#include "widgets/toggle_group.c"
#include "widgets/checkbox.c"
#include "widgets/switch.c"
#include "widgets/radio_button.c"
#include "widgets/radio_cards.c"
#include "widgets/slider.c"
#include "widgets/range_slider.c"
#include "widgets/chip.c"
#include "widgets/rating.c"
#include "widgets/tabs.c"
#include "widgets/breadcrumb.c"
#include "widgets/pagination.c"
#include "widgets/stepper.c"
#include "widgets/toast.c"
#include "widgets/tooltip.c"
#include "widgets/popover.c"
#include "widgets/confirm_popover.c"
#include "widgets/text_editor.c"
#include "widgets/text_field.c"
#include "widgets/search_field.c"
#include "widgets/text_area.c"
#include "widgets/tag_input.c"
#include "widgets/one_time_code.c"
#include "widgets/password_strength.c"
#include "widgets/select.c"
#include "widgets/number_field.c"
#include "widgets/split_button.c"
#include "widgets/menu.c"
#include "widgets/reactions.c"
#include "widgets/user_menu.c"
#include "host_theme.c"
