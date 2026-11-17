#ifndef FPAS_CUI_FILE_PICKER_H
#define FPAS_CUI_FILE_PICKER_H
/** Draws a model-controlled modal file picker.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param open Whether the model requests the modal to be visible.
 * @param enabled Whether this widget accepts input.
 * @param mode Open zero, save one, choose-folder two.
 * @param title Optional dialog title.
 * @param start Optional initial folder.
 * @param name Optional initial save name.
 * @param filter Optional semicolon-separated file patterns.
 * @param filter_label Optional human-readable filter caption.
 * @param source Optional synchronous folder-list callback; NULL uses the native filesystem.
 * @param user Borrowed callback context, valid throughout this call.
 * @param path Writable UTF-8 output path, meaningful only when the result is two.
 * @param capacity Output path buffer bytes including the terminator.
 * @param parts Five output nodes: dialog, cancel, confirm, save-name, list; minus one when absent.
 * @return Zero while unchanged, one on cancellation, two on selection; zero when disabled.
 */
CUI_API int fpas_cui_file_picker(cui_ctx *ui, const char *id, int open, int enabled, int mode, const char *title,
    const char *start, const char *name, const char *filter, const char *filter_label,
    cui_file_list_fn source, void *user, char *path, int capacity, int *parts);
#endif
