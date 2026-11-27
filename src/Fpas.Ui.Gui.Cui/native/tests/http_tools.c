// [Documentation](../../../../docs/ui/gui/more-developer-tools/http-request-bar.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static int failures;
#define CHECK(test) do { if (!(test)) { fprintf(stderr, "%d: %s\n", __LINE__, #test); ++failures; } } while (0)

/** Queue one key press without holding the key into the next frame. */
static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

/** Draw one request frame and check the width and exposed focus. */
static fpas_dev_result request(cui_ctx *ui, char url[32], int method, int flags, bool loading, int width_kind, float width) {
    cui_begin_frame(ui, .016f);
    fpas_dev_result result;
    int node = -1;
    cui_column(ui, .width = CUI_FILL, .align = CUI_ALIGN_START) {
        fpas_cui_request_bar(ui, "request", method, url, 32, loading, "URL", width_kind, width, flags, &result);
        node = fpas_cui_last_node(ui);
    }
    cui_end_frame(ui);
    const float actual = fpas_cui_node_rect(ui, node).w;
    if (width_kind == 0) CHECK(actual > 0 && actual < 640);
    if (width_kind == 2) CHECK(fabsf(actual - width) < .01f);
    if (flags & 2) CHECK(fpas_cui_focused(ui, node) == ((flags & 1) != 0));
    return result;
}

/** Exercise edits, disabled input, loading and focus across action replacements. */
static void request_actions(cui_ctx *ui) {
    char url[32] = "/items";
    CHECK(request(ui, url, 2, 3, false, 0, 0).flags == 0);
    cui_input_text(ui, "x");
    CHECK(request(ui, url, 2, 1, false, 0, 0).flags == 1);
    CHECK(strcmp(url, "x/items") == 0);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 1, false, 2, 320).flags == 2);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 1, true, 2, 320).flags == 0);
    press(ui, CUI_KEY_TAB);
    CHECK(request(ui, url, 2, 1, false, 2, 320).flags == 0);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 1, false, 2, 320).flags == 2);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 1, true, 2, 530).flags == 4);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 1, true, 2, 530).flags == 4);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 2, false, 2, 530).flags == 0);
    url[0] = 0;
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 2, 3, false, 2, 530).flags == 0);
}

/** Keep method navigation and activation inside the picker. */
static void request_method(cui_ctx *ui) {
    char url[32] = "/items";
    CHECK(request(ui, url, 0, 1, false, 1, 0).flags == 0);
    press(ui, CUI_KEY_TAB);
    CHECK(request(ui, url, 0, 1, false, 1, 0).flags == 0);
    press(ui, CUI_KEY_ENTER);
    CHECK(request(ui, url, 0, 1, false, 1, 0).flags == 0);
    press(ui, CUI_KEY_DOWN);
    CHECK(request(ui, url, 0, 1, false, 1, 0).flags == 0);
    press(ui, CUI_KEY_ENTER);
    const fpas_dev_result selected = request(ui, url, 0, 1, false, 1, 0);
    CHECK(selected.index == 1 && selected.flags == 1 && strcmp(url, "/items") == 0);
}

/** Leave keyboard and text input to an open popup while the URL retains focus. */
static void request_covered(cui_ctx *ui) {
    char url[32] = "/items";
    CHECK(request(ui, url, 0, 3, false, 1, 0).flags == 0);
    press(ui, CUI_KEY_ENTER);
    cui_input_text(ui, "x");
    cui_begin_frame(ui, .016f);
    cui_button(ui, "Anchor");
    cui_popover_open(ui, "cover");
    cui_popover(ui, "cover") { cui_text(ui, "Popup"); }
    fpas_dev_result result;
    fpas_cui_request_bar(ui, "request", 0, url, sizeof url, 0, NULL, 1, 0, 1, &result);
    CHECK(result.flags == 0 && strcmp(url, "/items") == 0);
    cui_end_frame(ui);
}

/** Compare the display through the adapter with the native display. */
static void response_status(cui_ctx *ui) {
    const double values[][3] = {
        { 0, -1, -1 }, { 201, .142, 18400 }, { 299, 1.25, 0 }, { 404, 0, 512 }
    };
    for (size_t i = 0; i < sizeof values / sizeof values[0]; ++i) {
        const int status = (int)values[i][0];
        const double seconds = values[i][1], bytes = values[i][2];
        cui_begin_frame(ui, .016f);
        cui_response_status(ui, status, seconds, bytes);
        cui_end_frame(ui);
        const int count = cui_get_draw_data(ui)->vertex_count;
        CHECK(count > 0);

        cui_begin_frame(ui, .016f);
        fpas_cui_response_status(ui, "response", status, seconds, bytes, 0, 0, i % 2);
        cui_end_frame(ui);
        CHECK(cui_get_draw_data(ui)->vertex_count == count);
    }
    cui_begin_frame(ui, .016f);
    fpas_cui_response_status(ui, "response", 201, .142, 18400, 2, 115, 1);
    const int node = fpas_cui_last_node(ui);
    cui_end_frame(ui);
    const cui_rect bounds = fpas_cui_node_rect(ui, node);
    CHECK(fabsf(bounds.w - 115) < .01f && bounds.h > 20);
}

/** Run the HTTP widget cases on independent contexts. */
int main(void) {
    void (* const cases[])(cui_ctx *) = { request_actions, request_method, request_covered, response_status };
    for (size_t i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        cui_ctx *ui = cui_create(NULL);
        if (!ui) return 1;
        cui_set_viewport(ui, 640, 300, 1);
        cui_set_reduced_motion(ui, true);
        cases[i](ui);
        cui_destroy(ui);
    }
    return failures != 0;
}
