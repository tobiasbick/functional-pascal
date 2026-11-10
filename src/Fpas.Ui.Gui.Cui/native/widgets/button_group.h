#ifndef FPAS_BUTTON_GROUP_H
#define FPAS_BUTTON_GROUP_H

/** Opens a native CUI group of connected buttons.
 * @param ui Current UI-thread context.
 * @return Frame-local group node. */
CUI_API int fpas_cui_button_group_begin(cui_ctx *ui);

/** Closes the native group and restores its node for shared sizing and styling.
 * @param ui Current UI-thread context.
 * @param node Frame-local node returned when the group opened. */
CUI_API void fpas_cui_button_group_end(cui_ctx *ui, int node);

#endif
