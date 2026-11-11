#ifndef FPAS_SWITCH_H
#define FPAS_SWITCH_H

/** Draws a model-controlled switch.
 * @param ui Current UI-thread context.
 * @param id Stable row identity.
 * @param label Borrowed literal caption.
 * @param description Borrowed secondary text, or NULL.
 * @param on Current model-owned value.
 * @param enabled Whether the row accepts input.
 * @param focus Whether to request keyboard focus.
 * @return Nonzero when CUI proposes a change. */
CUI_API int fpas_cui_switch(cui_ctx *ui, const char *id, const char *label,
    const char *description, int on, int enabled, int focus);

#endif
