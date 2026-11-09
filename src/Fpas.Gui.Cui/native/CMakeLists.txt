cmake_minimum_required(VERSION 3.20)
project(fpas_cui LANGUAGES C)

option(BUILD_SHARED_LIBS "Build the shared CUI library instead of its static archive" ON)
option(FPAS_CUI_BUILD_TESTS "Build the native consumer link/render smoke test" OFF)

if(NOT WIN32 AND NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "FPAS CUI currently targets Windows and Linux.")
endif()

get_filename_component(FPAS_CUI_UPSTREAM_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../third-party/cui" ABSOLUTE)
if(NOT EXISTS "${FPAS_CUI_UPSTREAM_DIR}/cui_single.h")
    message(FATAL_ERROR "Missing vendored CUI header: third-party/cui/cui_single.h")
endif()

add_library(fpas_cui fpas_cui.c bridge.c)
add_library(fpas::cui ALIAS fpas_cui)
set_target_properties(fpas_cui PROPERTIES
    C_STANDARD 99 C_STANDARD_REQUIRED YES C_EXTENSIONS OFF
    C_VISIBILITY_PRESET hidden POSITION_INDEPENDENT_CODE YES)
target_include_directories(fpas_cui PUBLIC "${FPAS_CUI_UPSTREAM_DIR}")
if(BUILD_SHARED_LIBS)
    target_compile_definitions(fpas_cui PUBLIC CUI_SHARED PRIVATE CUI_BUILDING)
endif()

if(MSVC)
    target_compile_options(fpas_cui PRIVATE /W4 /utf-8 /Gy /Gw
        /wd4100 /wd4201 /wd4204 /wd4221 /wd4244 /wd4267 /wd4305 /wd4996)
    target_compile_definitions(fpas_cui PRIVATE _CRT_SECURE_NO_WARNINGS)
else()
    target_compile_options(fpas_cui PRIVATE -Wall -Wextra -Wno-unused-parameter
        -Wno-missing-field-initializers -ffunction-sections -fdata-sections)
endif()

if(WIN32)
    target_link_libraries(fpas_cui PUBLIC
        user32 gdi32 opengl32 shell32 imm32 dwmapi comdlg32 ole32)
else()
    find_package(X11 REQUIRED)
    find_package(Threads REQUIRED)
    target_include_directories(fpas_cui PRIVATE ${X11_INCLUDE_DIR})
    target_link_libraries(fpas_cui PUBLIC ${X11_LIBRARIES} ${CMAKE_DL_LIBS} m Threads::Threads)
endif()

if(FPAS_CUI_BUILD_TESTS)
    enable_testing()
    add_executable(fpas_cui_smoke tests/link_render.c)
    set_target_properties(fpas_cui_smoke PROPERTIES
        C_STANDARD 99 C_STANDARD_REQUIRED YES C_EXTENSIONS OFF)
    target_link_libraries(fpas_cui_smoke PRIVATE fpas::cui)
    add_test(NAME fpas_cui_link_render COMMAND fpas_cui_smoke)
    set_tests_properties(fpas_cui_link_render PROPERTIES TIMEOUT 30)
endif()
