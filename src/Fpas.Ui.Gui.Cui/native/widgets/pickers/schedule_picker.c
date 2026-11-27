// [Documentation](../../../../../docs/ui/gui/developer-tools/schedule-picker.md)
#include "../../fpas_cui.h"

/** Restores the result for the model expression when a proposed edit does not fit. */
static cui_schedule_result fpas_schedule_model(const char *value, const cui_schedule_opts *options) {
    cui__cron parsed;
    cui_schedule_result result = { .valid = cui__cron_parse(value, &parsed) };
    if (result.valid) {
        const int64_t now = options->now ? options->now : (int64_t)time(NULL);
        const int offset = options->utc ? 0 : cui__utc_offset(now);
        int64_t next;
        if (cui_cron_next(value, now, offset, &next)) result.next = next;
    }
    return result;
}

/** Draws the native picker and returns only edits that fit the model buffer. */
void fpas_cui_schedule_picker(cui_ctx *ui, const char *id, char *value, size_t capacity, int64_t now, int utc, int next_count, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_schedule_opts o = { .width=fpas_extent(width_kind,width), .now=now, .utc=utc, .next_count=next_count };
    // CUI-135 (docs/future/cui-upstream-bugs.md#cui-135): a clipped preset can be another valid schedule.
    char preset[sizeof "59 23 * * 1,2,3,4,5,6,0"];
    char *proposal = value;
    size_t proposal_capacity = capacity;
    if (value && capacity >= 12 && capacity < sizeof preset) {
        snprintf(preset, sizeof preset, "%s", value);
        proposal = preset;
        proposal_capacity = sizeof preset;
    }
    cui_schedule_result r = cui_schedule_picker_ex(ui, id, proposal, proposal_capacity, &o);
    if (proposal == preset) {
        const size_t length = strlen(preset);
        if (length < capacity) memcpy(value, preset, length + 1);
        else {
            r = fpas_schedule_model(value, &o);
            cui_request_redraw(ui);
        }
    }
    out->flags = (r.changed ? 1 : 0) | (r.valid ? 2 : 0);
    out->x = (double)r.next;
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
