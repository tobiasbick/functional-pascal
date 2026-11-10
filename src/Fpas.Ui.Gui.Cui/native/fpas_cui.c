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

#include "bridge.c"
#include "controls.c"
#include "image.c"
#include "text_edit.c"
#include "widget_nodes.c"
#include "widgets/button.c"
#include "widgets/button_group.c"
#include "host_widgets.c"
#include "widgets/sidebar.c"
#include "widgets/sheet.c"
#include "widgets/icon_button.c"
#include "widgets/copy_button.c"
#include "widgets/hold_button.c"
#include "widgets/split_button.c"
#include "host_theme.c"
