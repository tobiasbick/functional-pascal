// [Documentation](../../../../../docs/ui/gui/more-developer-tools/regex-tester.md)
#include "../../fpas_cui.h"

static void fpas_regex_tester_retire(cui_ctx *ui, cui_id key) {
    const cui_id pattern = cui__hash_str("pattern", key);
    const cui_id sample = cui__hash_str("text", key);
    fpas_forget_state(ui, fpas_dev_text_slot(pattern));
    fpas_forget_state(ui, pattern);
    fpas_forget_state(ui, sample ^ CUI__ED_SALT);
    fpas_forget_state(ui, sample ^ 0xED15C011u);
    fpas_forget_state(ui, sample ^ 0xED1B0D1u);
    fpas_forget_state(ui, cui__hash_str("find", sample));
    fpas_forget_state(ui, cui__hash_str("goto", sample));
}

/** Repairs the text of an existing match chip while preserving its native layout. */
static void fpas_regex_chip(cui_ctx *ui, int chip, const char *text, int start, int end) {
    if (((unsigned char)text[start] & 0xC0) == 0x80 || ((unsigned char)text[end] & 0xC0) == 0x80) return;
    char label[44];
    int length = 0, offset = start;
    while (offset < end) {
        const int next = cui__utf8_next(text, end, offset);
        const int size = text[offset] == '\n' ? 2 : next - offset;
        if (length + size > 40) break;
        if (text[offset] == '\n') memcpy(label + length, "\\n", 2);
        else if (text[offset] == '\t') label[length] = ' ';
        else memcpy(label + length, text + offset, (size_t)size);
        length += size;
        offset = next;
    }
    if (offset < end) {
        memcpy(label + length, "\xe2\x80\xa6", 3);
        length += 3;
    }
    if (length == 0) return;
    cui_node *node = &ui->nodes[ui->nodes[chip].first];
    if (node->text_len == length && memcmp(node->text, label, (size_t)length) == 0) return;
    node->text = cui__frame_strdup(ui, label, length);
    node->text_len = length;
    node->font = CUI_FONT_MONO;
    node->text_color = CUI_C(ui, FOREGROUND);
}

/** CUI-140 (docs/future/cui-upstream-bugs.md#cui-140): preserve whole UTF-8 code points in match chips. */
static void fpas_regex_chips(cui_ctx *ui, int root, const cui__rt_state *state, const char *text) {
    if (!state || !state->nhits) return;
    const int controls = ui->nodes[root].first;
    const int editor = ui->nodes[controls].next;
    const int summary = ui->nodes[editor].next;
    int row = ui->nodes[summary].next;
    for (int index = 0; index < state->nhits && index < CUI__RT_LIST; ++index) {
        const cui__rt_hit *hit = &state->hits[index];
        const int number = ui->nodes[row].first;
        const int match = ui->nodes[number].next;
        fpas_regex_chip(ui, match, text, hit->start, hit->end);
        const int range = ui->nodes[match].next;
        int name = ui->nodes[range].next;
        for (int group = 1; group <= state->groups && group < CUI_REGEX_GROUPS; ++group) {
            const int chip = ui->nodes[name].next;
            if (hit->gs[group] >= 0) fpas_regex_chip(ui, chip, text, hit->gs[group], hit->ge[group]);
            name = ui->nodes[chip].next;
        }
        row = ui->nodes[row].next;
    }
}

/** Draws the regex tester and returns the native diagnostic for the current pattern and sample. */
const char *fpas_cui_regex_tester(cui_ctx *ui, const char *id, char *pattern, size_t pattern_cap, char *text, size_t text_cap, int options, float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    const cui_id state_key = key ^ 0x2E6Eu;
    const bool retained = options == -1;
    const cui__rt_state *state = cui__state_lazy(ui, state_key, sizeof *state, false);
    const int previous = retained ? (state ? state->own_flags : 0) : options;
    const cui_id pattern_field = cui__hash_str("pattern", key);
    const cui_id sample_field = cui__hash_str("text", key);
    *out = fpas_dev_empty();
    fpas_own(ui, key, state_key, fpas_dev_text_slot(sample_field), 0);
    fpas_on_forget(ui, key, fpas_regex_tester_retire);
    fpas_text_binding *pattern_binding = NULL, *sample_binding = NULL;
    if (pattern && pattern_cap && text && text_cap) {
        pattern_binding = fpas_dev_bind_text(ui, pattern_field, pattern, pattern_cap, false);
        sample_binding = fpas_dev_bind_text(ui, sample_field, text, text_cap, true);
    }
    cui_push_disabled(ui,!enabled);
    const cui_regex_tester_opts o = {
        .width = fpas_extent(width_kind, width), .height = height, .flags = retained ? NULL : &options
    };
    const cui_regex_tester_result r = cui_regex_tester_ex(ui,id,pattern,pattern_cap,text,text_cap,&o);
    cui_pop_disabled(ui);
    state = cui__state_lazy(ui, state_key, sizeof *state, false);
    out->index = retained ? (state ? state->own_flags : 0) : options;
    out->action = out->index != previous;
    out->count = r.matches;
    out->flags = r.valid;
    if (pattern_binding) {
        fpas_regex_chips(ui, root, state, text);
        fpas_text_store(ui, pattern_binding, pattern, strlen(pattern), pattern_cap);
        fpas_text_store(ui, sample_binding, text, strlen(text), text_cap);
    }
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
    return r.error;
}
