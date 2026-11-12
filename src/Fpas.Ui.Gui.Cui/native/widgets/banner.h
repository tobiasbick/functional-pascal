#ifndef FPAS_BANNER_H
#define FPAS_BANNER_H

/** Draws CUI's banner.
 * @param ui Current drawing context.
 * @param id Stable widget identity.
 * @param text Literal message.
 * @param shown Model-owned visibility.
 * @param action Optional action caption.
 * @param icon Optional icon glyph.
 * @param tone Native semantic tone.
 * @param dismissible Whether to show the close button.
 * @param enabled Whether to accept input.
 * @param focus 1 for action, 2 for dismiss, otherwise 0.
 * @param node Receives the banner root, or -1 when fully collapsed.
 * @param action_node Receives the native action node, or -1.
 * @param dismiss_node Receives the native dismiss node, or -1.
 * @param key Receives the root identity even when collapsed.
 * @return Action bit 1 and dismissal bit 2 */
CUI_API int fpas_cui_banner(cui_ctx *ui, const char *id, const char *text, int shown,
    const char *action, const char *icon, int tone, int dismissible, int enabled, int focus,
    int *node, int *action_node, int *dismiss_node, unsigned int *key);
#endif
