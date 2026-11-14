#ifndef FPAS_TOOLTIP_H
#define FPAS_TOOLTIP_H

/** Attaches native CUI hover help to the last widget's interactive anchor.
 * @param ui Current UI-thread context after drawing the target.
 * @param text Borrowed tooltip caption.
 * @param shortcut Borrowed shortcut hint, or NULL. */
CUI_API void fpas_cui_tooltip(cui_ctx *ui, const char *text, const char *shortcut);

/** Inspects the tooltip from the completed frame.
 * @param ui Current UI-thread context after presentation.
 * @param node Receives the visible tooltip node index, or -1.
 * @return The borrowed rendered caption including its shortcut, or NULL. */
CUI_API const char *fpas_cui_tooltip_inspect(cui_ctx *ui, int *node);

#endif
