#ifndef FPAS_FILE_DIALOG_H
#define FPAS_FILE_DIALOG_H

/** Opens the platform's file dialog on the owner thread.
 * @param app Owner window.
 * @param title Optional dialog title.
 * @param filter_name Optional human-readable filter name.
 * @param accept Optional semicolon-separated file extensions.
 * @param path Receives the UTF-8 selection.
 * @param capacity Size of the path buffer in bytes.
 * @return Whether the user selected a path matching the filter. */
CUI_API int fpas_cui_open_file(cui_app *app, const char *title, const char *filter_name,
    const char *accept, char *path, size_t capacity);
#endif
