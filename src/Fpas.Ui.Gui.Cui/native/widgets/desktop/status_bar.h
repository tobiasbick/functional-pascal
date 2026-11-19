// [Documentation](../../../../../docs/ui/gui/app-chrome/status-bar.md)
#ifndef FPAS_CUI_STATUS_BAR_H
#define FPAS_CUI_STATUS_BAR_H

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */

/** Begins a status row; pair with end in this frame. border controls its top border. */
CUI_API void fpas_cui_status_bar_begin(cui_ctx *ui, const char *id, int border);
/** Closes the matching row and makes it the last layout target. */
CUI_API void fpas_cui_status_bar_end(cui_ctx *ui);
/** Adds flexible space to the current status row. */
CUI_API void fpas_cui_status_bar_spacer(cui_ctx *ui);
/** Adds status text with an optional icon and native tone ordinal. */
CUI_API void fpas_cui_status_bar_text(cui_ctx *ui, const char *text, const char *icon, int tone);
/** Adds an actionable native keyed title with optional icon and tooltip. enabled gates input;
 * tone is a native tone ordinal; focus requests keyboard focus. node receives its frame-local node or -1 when absent.
 * Returns nonzero on activation. */
CUI_API int fpas_cui_status_bar_item(cui_ctx *ui, const char *title, const char *icon, int tone,
    const char *tooltip, int enabled, int focus, int *node);
/** Adds native progress with normalized value (0..1), optional label and logical width;
 * indeterminate selects animation instead of determinate progress. */
CUI_API void fpas_cui_status_bar_progress(cui_ctx *ui, float value, const char *label, float width, int indeterminate);

#endif
