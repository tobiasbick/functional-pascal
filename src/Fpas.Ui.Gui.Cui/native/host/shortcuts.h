// [Documentation](../../../../docs/ui/gui/cui.md#application-shortcuts)
#ifndef FPAS_CUI_SHORTCUTS_H
#define FPAS_CUI_SHORTCUTS_H
/** Reports text input, overlays and focused controls as keyboard owners (bits 1, 2 and 4). */
CUI_API int fpas_cui_shortcut_owners(cui_ctx *ui);
/** Returns the focused native editor or recorder that claimed the keyboard in the drawn frame. */
CUI_API uint32_t fpas_cui_shortcut_editor(cui_ctx *ui);
/** Consumes one matching native key combination after widget input has run. */
CUI_API int fpas_cui_take_shortcut(cui_ctx *ui, int mods, int key);
/** Copies native held-key levels for repeat suppression without per-key ABI calls. */
CUI_API void fpas_cui_shortcut_keys(cui_ctx *ui, unsigned char *keys, int count);
#endif
