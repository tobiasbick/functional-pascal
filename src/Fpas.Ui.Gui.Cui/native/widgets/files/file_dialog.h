#ifndef FPAS_FILE_DIALOG_H
#define FPAS_FILE_DIALOG_H

/** Opens the platform's file dialog on the owner thread.
 * @param app Owner window.
 * @param kind CUI_FILE_OPEN, CUI_FILE_SAVE or CUI_FILE_FOLDER.
 * @param title Optional dialog title.
 * @param filter_name Optional human-readable filter name.
 * @param accept Optional semicolon-separated file extensions.
 * @param default_name Optional proposed save filename.
 * @param path Receives the UTF-8 selection.
 * @param capacity Size of the path buffer in bytes.
 * @return Nonzero for a selection; Open additionally requires a matching extension. */
CUI_API int fpas_cui_file_dialog(cui_app *app, int kind, const char *title, const char *filter_name,
    const char *accept, const char *default_name, char *path, size_t capacity);
#endif
