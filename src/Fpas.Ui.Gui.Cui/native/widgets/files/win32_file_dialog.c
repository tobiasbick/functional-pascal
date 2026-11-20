// [Documentation](../../../../../docs/ui/gui/cui.md#desktop-services)
#include <shobjidl.h>

typedef struct fpas__win32_chooser {
    HWND owner;
    const cui_file_dialog_opts *options;
    char *path;
    size_t capacity;
    bool selected;
} fpas__win32_chooser;

static HRESULT fpas__win32_save_filter(IFileDialog *dialog, const cui_file_dialog_opts *options) {
    if (!options->filter || !options->filter[0]) return S_OK;
    WCHAR pattern[512], label[256];
    int used = 0;
    for (const char *part = options->filter; *part; ) {
        const char *end = strchr(part, ';');
        int length = end ? (int)(end - part) : (int)strlen(part);
        if (!length) { part = end + 1; continue; }
        if (used > (int)(sizeof pattern / sizeof *pattern) - 5) return S_OK;
        if (used) pattern[used++] = L';';
        pattern[used++] = L'*';
        pattern[used++] = L'.';
        int written = MultiByteToWideChar(CP_UTF8, 0, part, length, pattern + used,
            (int)(sizeof pattern / sizeof *pattern) - used - 1);
        if (!written) return S_OK;
        used += written;
        if (!end) break;
        part = end + 1;
    }
    if (!used) return S_OK;
    pattern[used] = 0;
    if (!MultiByteToWideChar(CP_UTF8, 0, options->filter_name ? options->filter_name : "Files",
            -1, label, (int)(sizeof label / sizeof *label))) return S_OK;
    const COMDLG_FILTERSPEC filters[] = { { label, pattern }, { L"All files", L"*.*" } };
    return dialog->lpVtbl->SetFileTypes(dialog, 2, filters);
}

static HRESULT fpas__win32_save_name(IFileDialog *dialog, const char *suggestion) {
    if (!suggestion || !suggestion[0]) return S_OK;
    WCHAR text[8192];
    if (!MultiByteToWideChar(CP_UTF8, 0, suggestion, -1, text, (int)(sizeof text / sizeof *text)))
        return E_INVALIDARG;
    WCHAR *separator = NULL;
    for (WCHAR *cursor = text; *cursor; ++cursor) {
        if (*cursor == L'\\' || *cursor == L'/') separator = cursor;
    }
    if (!separator) return dialog->lpVtbl->SetFileName(dialog, text);

    HRESULT hr = dialog->lpVtbl->SetFileName(dialog, separator + 1);
    if (FAILED(hr)) return hr;
    separator[1] = 0;
    WCHAR initial_path[8192];
    DWORD length = GetFullPathNameW(text, (DWORD)(sizeof initial_path / sizeof *initial_path), initial_path, NULL);
    if (!length || length >= sizeof initial_path / sizeof *initial_path) return S_OK;
    IShellItem *folder = NULL;
    hr = SHCreateItemFromParsingName(initial_path, NULL, &IID_IShellItem, (void **)&folder);
    if (SUCCEEDED(hr)) {
        dialog->lpVtbl->SetFolder(dialog, folder);
        folder->lpVtbl->Release(folder);
    }
    return S_OK;
}

static void fpas__win32_show_chooser(fpas__win32_chooser *request) {
    const cui_file_dialog_opts *options = request->options;
    bool save = options->kind == CUI_FILE_SAVE;
    IFileDialog *dialog = NULL;
    HRESULT hr = CoCreateInstance(save ? &CLSID_FileSaveDialog : &CLSID_FileOpenDialog,
        NULL, CLSCTX_INPROC_SERVER, &IID_IFileDialog, (void **)&dialog);
    if (FAILED(hr)) return;

    FILEOPENDIALOGOPTIONS flags;
    hr = dialog->lpVtbl->GetOptions(dialog, &flags);
    if (SUCCEEDED(hr)) {
        flags |= FOS_FORCEFILESYSTEM | FOS_NOCHANGEDIR | FOS_PATHMUSTEXIST;
        if (save) flags |= FOS_NOTESTFILECREATE;
        else flags = (flags & ~FOS_FILEMUSTEXIST) | FOS_PICKFOLDERS;
        hr = dialog->lpVtbl->SetOptions(dialog, flags);
    }
    if (SUCCEEDED(hr) && options->title && options->title[0]) {
        WCHAR title[256];
        if (MultiByteToWideChar(CP_UTF8, 0, options->title, -1, title, (int)(sizeof title / sizeof *title)))
            hr = dialog->lpVtbl->SetTitle(dialog, title);
    }
    if (SUCCEEDED(hr) && save) hr = fpas__win32_save_filter(dialog, options);
    if (SUCCEEDED(hr) && save) hr = fpas__win32_save_name(dialog, options->default_name);
    if (SUCCEEDED(hr)) hr = dialog->lpVtbl->Show(dialog, request->owner);

    IShellItem *item = NULL;
    if (SUCCEEDED(hr) && SUCCEEDED(dialog->lpVtbl->GetResult(dialog, &item))) {
        WCHAR *path = NULL;
        if (SUCCEEDED(item->lpVtbl->GetDisplayName(item, SIGDN_FILESYSPATH, &path))) {
            request->selected = WideCharToMultiByte(CP_UTF8, 0, path, -1, request->path,
                (int)request->capacity, NULL, NULL) > 0;
            CoTaskMemFree(path);
        }
        item->lpVtbl->Release(item);
    }
    dialog->lpVtbl->Release(dialog);
}

static DWORD WINAPI fpas__win32_chooser_thread(void *data) {
    if (SUCCEEDED(CoInitializeEx(NULL, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE))) {
        fpas__win32_show_chooser(data);
        CoUninitialize();
    }
    return 0;
}

/* CUI-046 and CUI-047: see docs/future/cui-upstream-bugs.md. */
static bool fpas__win32_file_dialog(HWND owner, const cui_file_dialog_opts *options,
    char *path, size_t capacity) {
    if (!owner || !path || !capacity || capacity > INT_MAX) return false;
    fpas__win32_chooser request = { owner, options, path, capacity, false };
    HRESULT hr = CoInitializeEx(NULL, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
    if (SUCCEEDED(hr)) {
        fpas__win32_show_chooser(&request);
        CoUninitialize();
    } else if (hr == RPC_E_CHANGED_MODE) {
        HANDLE thread = CreateThread(NULL, 0, fpas__win32_chooser_thread, &request, 0, NULL);
        if (!thread) return false;
        /* The shell sends synchronous messages to the owner while its dialog is open. */
        while (MsgWaitForMultipleObjects(1, &thread, FALSE, INFINITE, QS_SENDMESSAGE) == WAIT_OBJECT_0 + 1) {
            MSG message;
            PeekMessageW(&message, NULL, 0, 0, PM_NOREMOVE | PM_QS_SENDMESSAGE);
        }
        WaitForSingleObject(thread, INFINITE);
        CloseHandle(thread);
    }
    return request.selected;
}
