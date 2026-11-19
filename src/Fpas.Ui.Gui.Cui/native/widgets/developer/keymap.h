// [Documentation](../../../../../docs/ui/gui/developer-tools/keymap.md)
#ifndef FPAS_CUI_KEYMAP_H
#define FPAS_CUI_KEYMAP_H
/** Draws keyboard bindings and proposes modifier changes or activation.
 * Arrays and UTF-8 strings are borrowed for this call; action labels needed
 * during keycap painting are copied into frame storage.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param bindings Ordered key bindings.
 * @param bindings_count Number of bindings.
 * @param categories Category names referenced by binding category indices.
 * @param categories_count Number of category names.
 * @param modifiers Current native modifier bitmask.
 * @param hide_function_row Nonzero to omit function keys.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param enabled Nonzero to accept input outside disabled scopes.
 * @param out Receives index as proposed modifiers, action as clicked binding
 * index (or -1), and count as the number of conflicts. */
CUI_API void fpas_cui_keymap(cui_ctx *ui, const char *id, const cui_keymap_binding * bindings, int bindings_count, const char *const * categories, int categories_count, int modifiers, int hide_function_row, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
