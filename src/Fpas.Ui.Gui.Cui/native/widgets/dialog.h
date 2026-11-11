#ifndef FPAS_DIALOG_H
#define FPAS_DIALOG_H

/** Begins a native centered dialog.
 * @param ui Drawing context.
 * @param id Stable native identity independent of the heading.
 * @param title Literal heading.
 * @param description Optional supporting text.
 * @param width Preferred logical width.
 * @param open Model-owned visibility.
 * @param enabled Whether this modal accepts input.
 * @param persistent Suppress Escape and scrim dismissal.
 * @param close_button Show the native close button.
 * @param focus Request focus for the close button.
 * @param panel Receives the panel node, or -1.
 * @return Bit 1 draws the body and bit 2 proposes dismissal. */
CUI_API int fpas_cui_dialog_begin(cui_ctx *ui, const char *id, const char *title, const char *description,
    float width, int open, int enabled, int persistent, int close_button, int focus, int *panel);

/** Ends a dialog and restores the base layout target.
 * @param ui Drawing context.
 * @param base Base view's last frame-local node. */
CUI_API void fpas_cui_dialog_end(cui_ctx *ui, int base);

#endif
