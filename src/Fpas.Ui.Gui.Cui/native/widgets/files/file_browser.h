#ifndef FPAS_CUI_FILE_BROWSER_H
#define FPAS_CUI_FILE_BROWSER_H
/** Draws a file browser with model-owned folder and a synchronous source.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param folder Writable NUL-terminated UTF-8 folder buffer.
 * @param capacity Folder buffer bytes including the terminator.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param height_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param height Logical height; zero selects the native default where supported.
 * @param filter Optional semicolon-separated file patterns.
 * @param places Borrowed sidebar paths; used during this call.
 * @param place_count Number of sidebar paths.
 * @param source Optional synchronous folder-list callback; NULL uses the native filesystem.
 * @param user Borrowed callback context, valid throughout this call.
 * @param maximum Maximum entries, or zero for the native default.
 * @param revision Changing this value requests a reload.
 * @param flags Enabled bit one, folders-only two, show-hidden four, hide-places eight.
 * @param focus Whether to request keyboard focus.
 * @param selected Receives selected path or NULL; copy before the frame is released.
 * @param picked Receives activated path or NULL; copy before the frame is released.
 * @param parts Seven output nodes: list, back, forward, up, path, filter, reload; minus one when absent.
 * @return Navigated bit one and source-error bit two.
 */
CUI_API int fpas_cui_file_browser(cui_ctx *ui, const char *id, char *folder, int capacity,
    int width_kind, float width, int height_kind, float height, const char *filter,
    const char *const *places, int place_count, cui_file_list_fn source, void *user, int maximum,
    int revision, int flags, int focus, const char **selected, const char **picked, int *parts);
#endif
