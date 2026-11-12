#ifndef FPAS_HOVER_CARD_H
#define FPAS_HOVER_CARD_H

/** Draws the native hover card.
 * @param ui Current drawing context, with the trigger drawn immediately before the call.
 * @param id Stable identity.
 * @param placement Preferred position relative to the trigger.
 * @param width Logical width, or zero for the native default.
 * @param delay Hover delay in seconds, or zero for the native default.
 * @param enabled Whether the trigger can open the card.
 * @param node Receives the open card root, or -1.
 * @param owner Receives the retained popup identity, or zero.
 * @return Whether the native card scope needs its body drawn. */
CUI_API int fpas_cui_hover_card_begin(cui_ctx *ui, const char *id, int placement,
    float width, float delay, int enabled, int *node, unsigned int *owner);

/** Ends a native hover-card body and restores its trigger.
 * @param ui Current drawing context.
 * @param base The trigger's frame-local layout node. */
CUI_API void fpas_cui_hover_card_end(cui_ctx *ui, int base);
#endif
