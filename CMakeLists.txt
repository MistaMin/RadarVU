cmake_minimum_required(VERSION 3.22)
project(RadarVU VERSION 1.0.0)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_OSX_DEPLOYMENT_TARGET "11.0" CACHE STRING "Minimum macOS version")

# JUCE: a local checkout next to this repository (the ArcVu / audiogridder layout).
set(JUCE_DIR "${CMAKE_SOURCE_DIR}/../audiogridder/JUCE" CACHE PATH "JUCE checkout")
add_subdirectory(${JUCE_DIR} ${CMAKE_BINARY_DIR}/juce EXCLUDE_FROM_ALL)

# The loudness detector and the radar drawing list are a Rust static library.
find_program(RADARVU_CARGO cargo HINTS "$ENV{HOME}/.cargo/bin" "/opt/homebrew/bin" "/usr/local/bin")
if(NOT RADARVU_CARGO)
    message(FATAL_ERROR "cargo (Rust) is required to build the RadarVU detector")
endif()
if(WIN32)
    set(RADARVU_LIB_NAME radarvu_dsp.lib)
else()
    set(RADARVU_LIB_NAME libradarvu_dsp.a)
endif()
set(RADARVU_LIB ${CMAKE_BINARY_DIR}/${RADARVU_LIB_NAME})
set(RADARVU_RUST_SOURCES ${CMAKE_SOURCE_DIR}/dsp/src/lib.rs ${CMAKE_SOURCE_DIR}/dsp/src/loudness.rs ${CMAKE_SOURCE_DIR}/dsp/src/extra.rs ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml)
add_custom_command(
    OUTPUT ${RADARVU_LIB}
    COMMAND ${RADARVU_CARGO} build --release --manifest-path ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml
            --target-dir ${CMAKE_BINARY_DIR}/dsp-target
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_BINARY_DIR}/dsp-target/release/${RADARVU_LIB_NAME} ${RADARVU_LIB}
    DEPENDS ${RADARVU_RUST_SOURCES}
    COMMENT "Building RadarVU Rust DSP")
add_custom_target(radarvu_dsp DEPENDS ${RADARVU_LIB})

# VST3 and Standalone everywhere, AU on macOS.
set(RADARVU_FORMATS VST3 Standalone)
if(APPLE)
    list(APPEND RADARVU_FORMATS AU)
endif()
message(STATUS "RadarVU formats: ${RADARVU_FORMATS}")

juce_add_plugin(RadarVU
    COMPANY_NAME "OpenGrid"
    BUNDLE_ID com.opengrid.radarvu
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    IS_MIDI_EFFECT FALSE
    EDITOR_WANTS_KEYBOARD_FOCUS FALSE
    COPY_PLUGIN_AFTER_BUILD FALSE
    PLUGIN_MANUFACTURER_CODE OpGr
    PLUGIN_CODE RdVU
    FORMATS ${RADARVU_FORMATS}
    DESCRIPTION "BS.1770-5 loudness meter with a radar display"
    VST3_CATEGORIES Fx Analyzer
    AU_MAIN_TYPE kAudioUnitType_Effect
    MICROPHONE_PERMISSION_ENABLED TRUE
    MICROPHONE_PERMISSION_TEXT "RadarVU uses audio input to meter loudness in the standalone application."
    HARDENED_RUNTIME_ENABLED TRUE
    HARDENED_RUNTIME_OPTIONS com.apple.security.device.audio-input
    PRODUCT_NAME "RadarVU")

juce_generate_juce_header(RadarVU)
target_sources(RadarVU PRIVATE
    plugin/PluginProcessor.cpp
    plugin/PluginEditor.cpp)
target_include_directories(RadarVU PRIVATE plugin)
target_compile_definitions(RadarVU PUBLIC
    JUCE_WEB_BROWSER=0
    JUCE_USE_CURL=0
    JUCE_VST3_CAN_REPLACE_VST2=0)
target_link_libraries(RadarVU PRIVATE
    ${RADARVU_LIB}
    juce::juce_audio_utils
    juce::juce_recommended_config_flags
    juce::juce_recommended_warning_flags)
if(WIN32)
    target_link_libraries(RadarVU PRIVATE ws2_32 userenv ntdll bcrypt advapi32)
elseif(UNIX AND NOT APPLE)
    target_link_libraries(RadarVU PRIVATE pthread dl m)
endif()
add_dependencies(RadarVU radarvu_dsp)

option(RADARVU_BUILD_QA "Build the offscreen UI audit harness" OFF)
if(RADARVU_BUILD_QA)
    juce_add_console_app(RadarVU_QA PRODUCT_NAME "RadarVU QA")
    juce_generate_juce_header(RadarVU_QA)
    target_sources(RadarVU_QA PRIVATE qa/UiAudit.cpp plugin/PluginProcessor.cpp plugin/PluginEditor.cpp)
    target_include_directories(RadarVU_QA PRIVATE plugin)
    target_compile_definitions(RadarVU_QA PRIVATE JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_VST3_CAN_REPLACE_VST2=0)
    target_link_libraries(RadarVU_QA PRIVATE ${RADARVU_LIB} juce::juce_audio_utils juce::juce_recommended_config_flags)
    if(WIN32)
        target_link_libraries(RadarVU_QA PRIVATE ws2_32 userenv ntdll bcrypt advapi32)
    endif()
    add_dependencies(RadarVU_QA radarvu_dsp)
endif()
