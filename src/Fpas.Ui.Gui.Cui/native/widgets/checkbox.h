#ifndef FPAS_CHECKBOX_H
#define FPAS_CHECKBOX_H

/** Draws a model-controlled checkbox.
 * @param ui Current UI-thread context.
 * @param id Stable row identity.
 * @param label Borrowed literal caption.
 * @param description Borrowed secondary text, or NULL.
 * @param checked_value Current model-owned value.
 * @param enabled Whether the row accepts input.
 * @param focus Whether to request keyboard focus.
 * @param indeterminate Draw a dash in place of the check mark while checked.
 * @return Nonzero when CUI proposes a change. */
CUI_API int fpas_cui_checkbox(cui_ctx *ui, const char *id, const char *label,
    const char *description, int checked_value, int enabled, int focus, int indeterminate);

#endif
