# [Documentation](../../../docs/ui/gui/native-build.md)
cmake_minimum_required(VERSION 3.20)
project(fpas_cui LANGUAGES C)

option(BUILD_SHARED_LIBS "Build the shared CUI library instead of its static archive" ON)
option(FPAS_CUI_BUILD_TESTS "Build the native CUI consumer tests" OFF)

# Multi-config generators select the configuration at build time with --config.
if(FPAS_CUI_BUILD_CONFIGURATION AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE "${FPAS_CUI_BUILD_CONFIGURATION}")
endif()

if(NOT WIN32 AND NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "FPAS CUI currently targets Windows and Linux.")
endif()

set(FPAS_CUI_UPSTREAM_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../third-party/cui/upstream")
if(NOT EXISTS "${FPAS_CUI_UPSTREAM_DIR}/cui.c" OR NOT EXISTS "${FPAS_CUI_UPSTREAM_DIR}/cui.h")
    message(FATAL_ERROR "Missing vendored cui.c or cui.h in third-party/cui/upstream")
endif()

if(MSVC AND NOT BUILD_SHARED_LIBS)
    # Match the Windows NativeAOT runtime in both Debug and Release.
    set(CMAKE_MSVC_RUNTIME_LIBRARY MultiThreaded)
endif()

add_library(cui fpas_cui.c host/wakeup.c)
add_library(fpas::cui ALIAS cui)
target_include_directories(cui PUBLIC "${FPAS_CUI_UPSTREAM_DIR}")
# The vendored implementation includes its own bidi classifier and static text shaper.
target_compile_definitions(cui PRIVATE CUI_TEXT_SHAPING)
set_target_properties(cui PROPERTIES
    C_STANDARD 99 C_STANDARD_REQUIRED YES C_EXTENSIONS OFF
    C_VISIBILITY_PRESET hidden OUTPUT_NAME fpas_cui POSITION_INDEPENDENT_CODE YES)
if(BUILD_SHARED_LIBS)
    target_compile_definitions(cui PUBLIC CUI_SHARED PRIVATE CUI_BUILDING)
endif()
if(MSVC)
    target_compile_options(cui PRIVATE /Gy /Gw /W4 /wd4100 /wd4201 /wd4204 /wd4221 /wd4244 /wd4267 /wd4305 /wd4996 /utf-8)
    target_compile_definitions(cui PRIVATE _CRT_SECURE_NO_WARNINGS)
else()
    target_compile_options(cui PRIVATE -ffunction-sections -fdata-sections -Wall -Wextra
        -Wno-unused-parameter -Wno-missing-field-initializers)
endif()
if(WIN32)
    # Capture passes UTF-8 output paths to the bundled PNG writer.
    target_compile_definitions(cui PRIVATE STBIW_WINDOWS_UTF8)
    target_link_libraries(cui PUBLIC user32 gdi32 opengl32 shell32 imm32 dwmapi comdlg32 ole32)
else()
    find_package(X11 REQUIRED)
    target_include_directories(cui PRIVATE ${X11_INCLUDE_DIR})
    target_link_libraries(cui PUBLIC ${X11_LIBRARIES} ${CMAKE_DL_LIBS} m)
endif()
foreach(kind IN ITEMS ARCHIVE LIBRARY RUNTIME)
    if(FPAS_CUI_OUTPUT_DIRECTORY)
        # A generator expression suppresses the multi-config directory suffix.
        set_property(TARGET cui PROPERTY ${kind}_OUTPUT_DIRECTORY "$<1:${FPAS_CUI_OUTPUT_DIRECTORY}>")
    elseif(NOT DEFINED CMAKE_${kind}_OUTPUT_DIRECTORY)
        set_property(TARGET cui PROPERTY ${kind}_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}")
    endif()
endforeach()

if(FPAS_CUI_BUILD_TESTS)
    enable_testing()
    add_executable(fpas_cui_smoke tests/link_render.c)
    set_target_properties(fpas_cui_smoke PROPERTIES
        C_STANDARD 99 C_STANDARD_REQUIRED YES C_EXTENSIONS OFF)
    target_link_libraries(fpas_cui_smoke PRIVATE fpas::cui)
    add_test(NAME fpas_cui_link_render COMMAND fpas_cui_smoke)
    set_tests_properties(fpas_cui_link_render PROPERTIES TIMEOUT 30)
    foreach(widget IN ITEMS
        confirm_button status_button segmented icon_button menu disclosure feedback
        meter progress_ring timeline skeleton layout_parity tour input_regressions
        property_text_replacement calendar smart_inputs wheel_interruption charts
        visualizations more_charts collection_regressions grid_regressions grid_editing
        code_instruments instrument_display api_extensions editor_extensions painting motion creative creative_choices
        planning social media voice_ai voice_wave voice_composer voice_session voice_note ai_cards developer_tools
        canvas_bars canvas_cards canvas_format_bar
        canvas_toolbar display source_tools
        editing_tools desktop_chrome developer_graphs developer_inspectors developer_editors http_tools)
        add_executable(fpas_cui_${widget} tests/${widget}.c)
        set_target_properties(fpas_cui_${widget} PROPERTIES
            C_STANDARD 99 C_STANDARD_REQUIRED YES C_EXTENSIONS OFF)
        target_link_libraries(fpas_cui_${widget} PRIVATE fpas::cui)
        add_test(NAME fpas_cui_${widget} COMMAND fpas_cui_${widget})
        set_tests_properties(fpas_cui_${widget} PROPERTIES TIMEOUT 30)
    endforeach()
    foreach(widget IN ITEMS
        planning code_instruments voice_ai grid_regressions motion creative
        collection_regressions developer_tools social canvas_bars display editing_tools)
        add_test(NAME fpas_cui_${widget}_upstream COMMAND fpas_cui_${widget} upstream)
        set_tests_properties(fpas_cui_${widget}_upstream PROPERTIES TIMEOUT 30)
    endforeach()
    add_test(NAME fpas_cui_desktop_chrome_upstream COMMAND fpas_cui_desktop_chrome native)
    add_test(NAME fpas_cui_developer_tools_upstream_measurement
        COMMAND fpas_cui_developer_tools measurement)
    set_tests_properties(fpas_cui_desktop_chrome_upstream fpas_cui_developer_tools_upstream_measurement
        PROPERTIES TIMEOUT 30)
    if(WIN32)
        foreach(window_test IN ITEMS live_resize swipe_wheel window_transition)
            add_executable(fpas_cui_${window_test} tests/${window_test}.c)
            target_link_libraries(fpas_cui_${window_test} PRIVATE fpas::cui)
            add_test(NAME fpas_cui_${window_test} COMMAND fpas_cui_${window_test})
            set_tests_properties(fpas_cui_${window_test} PROPERTIES TIMEOUT 30)
        endforeach()
        set_tests_properties(fpas_cui_window_transition PROPERTIES SKIP_RETURN_CODE 77)
        set_tests_properties(fpas_cui_swipe_wheel PROPERTIES ENVIRONMENT "CUI_SCALE=1")
    endif()
endif()
