cmake_minimum_required(VERSION 3.20)
project(fpas_cui LANGUAGES C)

option(BUILD_SHARED_LIBS "Build the shared CUI library instead of its static archive" ON)
option(FPAS_CUI_BUILD_TESTS "Build the native consumer link/render smoke test" OFF)

# Multi-config generators select the configuration at build time with --config.
if(FPAS_CUI_BUILD_CONFIGURATION AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE "${FPAS_CUI_BUILD_CONFIGURATION}")
endif()

if(NOT WIN32 AND NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "FPAS CUI currently targets Windows and Linux.")
endif()

set(FPAS_CUI_UPSTREAM_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../third-party/cui/upstream")
if(NOT EXISTS "${FPAS_CUI_UPSTREAM_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "Missing vendored CUI sources: third-party/cui/upstream")
endif()

set(CUI_BUILD_SHARED ${BUILD_SHARED_LIBS})
set(CUI_BUILD_EXAMPLES OFF)
set(CUI_BUILD_TESTS OFF)
add_subdirectory("${FPAS_CUI_UPSTREAM_DIR}" upstream)

target_sources(cui PRIVATE bridge.c controls.c wakeup.c)
target_include_directories(cui PRIVATE "${FPAS_CUI_UPSTREAM_DIR}/src")
add_library(fpas::cui ALIAS cui)
set_target_properties(cui PROPERTIES OUTPUT_NAME fpas_cui POSITION_INDEPENDENT_CODE YES)
# The binding distributes one unversioned library under its P/Invoke name.
set_property(TARGET cui PROPERTY VERSION)
set_property(TARGET cui PROPERTY SOVERSION)
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
    if(WIN32)
        add_executable(fpas_cui_live_resize tests/live_resize.c)
        target_link_libraries(fpas_cui_live_resize PRIVATE fpas::cui)
        add_test(NAME fpas_cui_live_resize COMMAND fpas_cui_live_resize)
        set_tests_properties(fpas_cui_live_resize PROPERTIES TIMEOUT 30)
    endif()
endif()
