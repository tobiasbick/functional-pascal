#ifndef FPAS_LINK_H
#define FPAS_LINK_H

/** Draws the native link.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param label Literal caption.
 * @param enabled Whether to accept activation.
 * @param focus Whether to request keyboard focus.
 * @param node Receives the native link node.
 * @return Whether CUI activated the link. */
CUI_API int fpas_cui_link(cui_ctx *ui, const char *id, const char *label, int enabled, int focus, int *node);
#endif
