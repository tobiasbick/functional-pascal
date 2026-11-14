#ifndef FPAS_CONFIRM_POPOVER_H
#define FPAS_CONFIRM_POPOVER_H

/** Draws CUI's anchored confirmation question and both native actions.
 * @param ui Drawing context after the trigger.
 * @param id Stable native panel identity.
 * @param title Borrowed question.
 * @param description Borrowed optional supporting text.
 * @param confirm Borrowed affirmative caption.
 * @param cancel Borrowed cancellation caption.
 * @param placement CUI placement.
 * @param danger Nonzero for a danger action.
 * @param open Model-owned visibility.
 * @param enabled Input permission.
 * @param nodes Receives the panel, cancel and confirm nodes, or -1.
 * @param owner Receives the native state owner, or zero.
 * @return Bit 1 proposes confirmation, bit 2 dismissal and bit 4 an open panel. */
CUI_API int fpas_cui_confirm_popover(cui_ctx *ui, const char *id, const char *title,
    const char *description, const char *confirm, const char *cancel, int placement,
    int danger, int open, int enabled, int *nodes, unsigned int *owner);

#endif
