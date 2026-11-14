#ifndef FPAS_CHECKLIST_H
#define FPAS_CHECKLIST_H

/** Draws CUI's checklist.
 * @param ui Current drawing context.
 * @param id Stable widget identity.
 * @param title Optional card heading.
 * @param description Optional replacement for the native progress description.
 * @param titles Step titles.
 * @param descriptions Step descriptions or empty strings.
 * @param actions Step action captions or empty strings.
 * @param completed In-out completion flags, one per step.
 * @param count Number of steps.
 * @param enabled Whether to accept input.
 * @param focus One plus the requested slot index, or zero.
 * @param node Receives the checklist root.
 * @param parts Optional two slots per step for its circle and visible action.
 * @return Index of an activated action, or -1 */
CUI_API int fpas_cui_checklist(cui_ctx *ui, const char *id, const char *title, const char *description,
    const char *const *titles, const char *const *descriptions, const char *const *actions,
    int *completed, int count, int enabled, int focus, int *node, int *parts);
#endif
