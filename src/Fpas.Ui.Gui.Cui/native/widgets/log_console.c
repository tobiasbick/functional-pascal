#include "../fpas_cui.h"

typedef struct { cui_log_buffer log; } fpas_log_console_state;

static void fpas_log_console_free(cui_ctx *ui, void *data) {
    fpas_log_console_state *state = data;
    cui__free(ui, state->log.lines);
    cui__free(ui, state->log.text);
}

static bool fpas_log_matches(const cui_log_line *line, const char *stored, const fpas_log_entry *entry) {
    if (line->level != entry->level || line->time != entry->time) return false;
    size_t length = strlen(entry->text);
    while (length && (entry->text[length - 1] == '\n' || entry->text[length - 1] == '\r')) --length;
    if (length != line->len) return false;
    for (size_t i = 0; i < length; ++i) {
        const unsigned char value = (unsigned char)entry->text[i];
        if ((value < 0x20 || value == 0x7f ? ' ' : value) != (unsigned char)stored[i]) return false;
    }
    return true;
}

static void fpas_log_console_update(cui_ctx *ui, cui_log_buffer *log, const fpas_log_entry *entries, int count) {
    bool append = count >= log->count;
    for (int i = 0; append && i < log->count; ++i) {
        const char *stored;
        const cui_log_line *line = cui_log_get(log, i, &stored);
        append = fpas_log_matches(line, stored, &entries[i]);
    }
    int first = append ? log->count : 0;
    if (append && first == count) return;

    size_t bytes = append ? log->text_end : 0;
    for (int i = first; i < count; ++i) bytes += strlen(entries[i].text) + 1;
    if (count > log->line_cap || bytes > log->text_cap) {
        const uint64_t sequence = append ? log->total - (uint64_t)log->count : log->total;
        const int slots = CUI_MAX(count, CUI_MAX(8, log->line_cap + log->line_cap / 2));
        const size_t capacity = CUI_MAX(bytes, CUI_MAX((size_t)256, (size_t)log->text_cap * 2));
        cui_log_line *lines = cui__realloc(ui, log->lines, (size_t)slots * sizeof *lines);
        char *text = cui__realloc(ui, log->text, capacity);
        cui_log_init(log, lines, slots, text, capacity);
        log->total = sequence;
        first = 0;
    } else if (!append) cui_log_clear(log);

    for (int i = first; i < count; ++i)
        cui_log_append_text(log, (cui_log_level)entries[i].level, entries[i].time, entries[i].text, -1);
}

static void fpas_log_console_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, key ^ 0x5C2011u);
    const cui_id input_scope = cui__hash_str("search", key);
    fpas_forget_state(ui, input_scope);
}

int fpas_cui_log_console(cui_ctx *ui, const char *id, const fpas_log_entry *entries, int count, int width_kind, float width, float height, float size, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0x106C01u;
    fpas_own(ui, key, slot, key ^ 0x106Bu, 0);
    fpas_on_forget(ui, key, fpas_log_console_retire);
    fpas_log_console_state *state = cui_state(ui, slot, sizeof *state);
    cui__state_set_dtor(ui, slot, fpas_log_console_free);
    cui_log_buffer *log = &state->log;
    fpas_log_console_update(ui, log, entries, count);

    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_log_view_opts options = { .width = fpas_extent(width_kind, width), .height = height, .size = size,
        .timestamps = (flags & 2) != 0, .no_toolbar = (flags & 4) != 0 };
    const int previous = log->count;
    cui_push_disabled(ui, !enabled);
    cui_log_view_ex(ui, id, log, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    return enabled && previous > 0 && log->count == 0;
}
