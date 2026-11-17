#ifndef FPAS_CUI_TOOLBAR_H
#define FPAS_CUI_TOOLBAR_H

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */
/** Begins a toolbar with native style ordinal; no_border hides its border and enabled
 * gates input. Pair with toolbar_end in the same frame. */
CUI_API void fpas_cui_toolbar_begin(cui_ctx *ui, const char *id, int style, int no_border, int enabled);
/** Closes the matching toolbar, including native overflow construction, and makes its root
 * the last layout target. */
CUI_API void fpas_cui_toolbar_end(cui_ctx *ui);
/** Adds an ordinary or primary tool. icon/label/tooltip/shortcut are optional strings.
 * toggle is -1 for an action or an input boolean for a toggle; tone is the native ordinal.
 * flags: 1 disabled, 2 flyout marker, 4 primary. Returns -1 when idle, otherwise the
 * proposed toggle boolean (zero for an ordinary action). The icon is copied for this frame. */
CUI_API int fpas_cui_toolbar_button(cui_ctx *ui, const char *icon, const char *label, const char *tooltip,
    const char *shortcut, int toggle, int tone, int flags);
/** Adds a native exclusive tool group. icons and labels have count entries; icons are
 * copied for this frame. Supports up to 32 entries (larger counts are truncated).
 * value is the current index; returns proposed index or -1 when unchanged. */
CUI_API int fpas_cui_toolbar_radio(cui_ctx *ui, const char *id, const char *const *icons,
    const char *const *labels, int count, int value);
/** Adds a native cycling choice. labels has count entries, value is the current index,
 * tooltip is optional; returns proposed index or -1 when unchanged. */
CUI_API int fpas_cui_toolbar_cycle(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int value, const char *tooltip);
#endif
