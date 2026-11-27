// [Documentation](../../../../../docs/ui/gui/motion/text-reveal.md)
#include "../../fpas_cui.h"

/** Keeps the original text geometry while separating words at tabs and line breaks. */
void fpas_cui_text_reveal(cui_ctx *ui, const char *id, const char *text, int replay, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_motion_replay(ui, key, replay);
    const cui_text_fx_opts options = fpas_motion_options(o);
    // CUI-098 (docs/future/cui-upstream-bugs.md#cui-098): the native word counter checks ASCII spaces only.
    if (!cui__still(ui) && text && strpbrk(text, "\t\r\n")) {
        const size_t length = strlen(text);
        char *word_text = cui__frame_strdup(ui, text, (int)length);
        for (char *ch = word_text; *ch; ++ch)
            if (*ch == '\t' || *ch == '\r' || *ch == '\n') *ch = ' ';
        cui__rv_state *state = cui_state(ui, key, sizeof *state);
        const uint32_t original_hash = cui__hash(text, length, 0x2E7u);
        if (state->init && state->hash == original_hash) state->hash = cui__hash(word_text, length, 0x2E7u);
        else state->init = false;
        cui_text_reveal_ex(ui, id, word_text, &options);
        state->hash = original_hash;
        ui->nodes[node].text = cui__frame_strdup(ui, text, (int)length);
    } else cui_text_reveal_ex(ui, id, text, &options);
    fpas_motion_node(ui, node, key);
}
